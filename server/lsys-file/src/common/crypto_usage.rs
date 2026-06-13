// 这是一个使用示例文件，展示如何使用加密功能
// 此文件不会被编译，仅供参考

#[allow(dead_code)]
mod examples {
    use crate::dao::file_helpers::FileHelper;
    use crate::model::FileModel;
    use std::io::Write;

    /// 示例 1: 加密文件
    fn example_encrypt_file(file_helper: &FileHelper) -> std::io::Result<()> {
        // 加密文件
        let encrypted_info = file_helper.encrypt_file("/path/to/source/file.txt")?;
        
        println!("加密成功！");
        println!("  文件名: {}", encrypted_info.filename);
        println!("  相对路径: {}", encrypted_info.relative_path);
        println!("  完整路径: {:?}", encrypted_info.full_path);
        
        Ok(())
    }

    /// 示例 2: 解密文件到公开存储
    fn example_decrypt_to_public(file_helper: &FileHelper) -> std::io::Result<()> {
        let encrypted_path = "enc_1234567890_12345_txt.enc";
        
        // 解密到公开存储，指定原始扩展名
        let decrypted_info = file_helper.decrypt_file_to_storage(
            encrypted_path,
            FileModel::STORAGE_TYPE_LOCAL_PUBLIC,
            Some("txt"), // 指定原始文件扩展名
        )?;
        
        println!("解密到公开存储成功！");
        println!("  相对路径: {}", decrypted_info.0);
        println!("  完整路径: {:?}", decrypted_info.1);
        
        Ok(())
    }

    /// 示例 3: 解密文件到私有存储（使用默认扩展名）
    fn example_decrypt_to_private(file_helper: &FileHelper) -> std::io::Result<()> {
        let encrypted_path = "enc_1234567890_12345_txt.enc";
        
        // 解密到私有存储，使用默认扩展名 "dat"
        let decrypted_info = file_helper.decrypt_file_to_storage(
            encrypted_path,
            FileModel::STORAGE_TYPE_LOCAL_PRIVATE,
            None,  // 使用默认扩展名 "dat"
        )?;
        
        println!("解密到私有存储成功！");
        println!("  相对路径: {}", decrypted_info.0);
        println!("  完整路径: {:?}", decrypted_info.1);
        
        Ok(())
    }

    /// 示例 4: 流式解密（适合超大文件）
    fn example_stream_decrypt(file_helper: &FileHelper) -> std::io::Result<()> {
        let encrypted_path = "enc_large_file.enc";
        
        // 创建解密迭代器
        let iter = file_helper.decrypt_file_range(encrypted_path, 0, None)?;
        
        println!("开始流式解密，总共 {} 个块", iter.total_chunks());
        
        let mut output_file = std::fs::File::create("/tmp/decrypted_output.bin")?;
        
        // 逐块解密并写入
        for (idx, result) in iter.enumerate() {
            let (chunk_info, data) = result?;
            output_file.write_all(&data)?;
            
            println!("已处理块 {}/{}, 大小: {} 字节", 
                     idx + 1, 
                     chunk_info.index + 1,
                     data.len());
        }
        
        output_file.flush()?;
        println!("流式解密完成！");
        
        Ok(())
    }

    /// 示例 5: 部分解密（Range 请求）
    fn example_partial_decrypt(file_helper: &FileHelper) -> std::io::Result<()> {
        let encrypted_path = "enc_video.enc";
        
        // 从偏移 1MB 开始，读取 2MB
        let offset = 1024 * 1024;
        let length = 2 * 1024 * 1024;
        
        let iter = file_helper.decrypt_file_range(
            encrypted_path,
            offset,
            Some(length),
        )?;
        
        let (start, end, total) = iter.range_info();
        println!("读取范围: {} - {}, 总长度: {} 字节", start, end, total);
        
        let mut partial_data = Vec::new();
        for result in iter {
            let (_, data) = result?;
            partial_data.extend_from_slice(&data);
        }
        
        println!("部分解密完成，共 {} 字节", partial_data.len());
        
        Ok(())
    }

    /// 示例 6: 验证加密文件
    fn example_verify_file(file_helper: &FileHelper) -> std::io::Result<()> {
        let encrypted_path = "enc_1234567890_12345_txt.enc";
        
        // 验证文件是否有效
        let is_valid = file_helper.verify_encrypted_file(encrypted_path)?;
        
        if is_valid {
            println!("文件验证成功，可以正常解密");
        } else {
            println!("文件验证失败，可能已损坏");
        }
        
        Ok(())
    }

    /// 示例 7: 获取加密文件大小
    fn example_get_file_size(file_helper: &FileHelper) -> std::io::Result<()> {
        let encrypted_path = "enc_1234567890_12345_txt.enc";
        
        // 获取原始文件大小（解密后的大小）
        let size = file_helper.get_encrypted_file_size(encrypted_path)?;
        
        println!("原始文件大小: {} 字节 ({:.2} MB)", 
                 size, 
                 size as f64 / 1024.0 / 1024.0);
        
        Ok(())
    }

    /// 示例 8: 删除加密文件
    async fn example_delete_file(file_helper: &FileHelper) -> std::io::Result<()> {
        let encrypted_path = "enc_1234567890_12345_txt.enc";
        
        // 获取加密文件的完整路径
        let crypto_base = file_helper.config
            .get_base_path(FileModel::STORAGE_TYPE_LOCAL_CRYPTO)
            .await?;
        let full_path = crypto_base.join(encrypted_path);
        
        // 直接删除文件
        tokio::fs::remove_file(full_path).await?;
        
        println!("加密文件已删除");
        
        Ok(())
    }

    /// 示例 9: 完整工作流程
    async fn example_full_workflow(file_helper: &FileHelper) -> std::io::Result<()> {
        // 1. 加密文件
        println!("步骤 1: 加密文件");
        let encrypted_info = file_helper.encrypt_file("/path/to/source.txt").await?;
        println!("  加密完成: {}", encrypted_info.0);
        
        // 2. 验证加密文件
        println!("\n步骤 2: 验证加密文件");
        let is_valid = file_helper.verify_encrypted_file(&encrypted_info.0).await?;
        println!("  验证结果: {}", if is_valid { "有效" } else { "无效" });
        
        // 3. 获取文件大小
        println!("\n步骤 3: 获取文件大小");
        let size = file_helper.get_encrypted_file_size(&encrypted_info.0).await?;
        println!("  文件大小: {} 字节", size);
        
        // 4. 解密到公开存储
        println!("\n步骤 4: 解密到公开存储");
        let decrypted_info = file_helper.decrypt_file_to_storage(
            &encrypted_info.0,
            FileModel::STORAGE_TYPE_LOCAL_PUBLIC,
            Some("txt"), // 指定原始扩展名
        ).await?;
        println!("  解密完成: {:?}", decrypted_info.1);
        
        // 5. 删除加密文件（可选）
        println!("\n步骤 5: 删除加密文件");
        tokio::fs::remove_file(&encrypted_info.1).await?;
        println!("  删除完成");
        
        Ok(())
    }
}
