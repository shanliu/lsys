CREATE TABLE `yaf_file` (
	`id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `storage_type` VARCHAR(12) NOT NULL COMMENT '文件存储类型：local=本地存储, aliyun=阿里云OSS,tencent=腾讯云COS',
    `status` TINYINT NOT NULL COMMENT '状态：1=正常, 2=已删除,3=未完成,4=失败',
    `file_name` VARCHAR(255) NOT NULL  DEFAULT '' COMMENT '文件原始文件名',
    `file_md5` CHAR(32) NOT NULL DEFAULT '' COMMENT '文件md5',
    `file_size` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '文件大小',
    `modify_time` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '文件最后修改时间',
    `content_type` VARCHAR(128) NOT NULL DEFAULT '' COMMENT '文件MIME类型',
    `copy_file_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '复制来源文件ID',
    `from_user_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '上传用户ID',
    `add_time` BIGINT UNSIGNED NOT NULL COMMENT '添加时间',
    `change_time` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '记录更新时间',
    KEY `file_md5` (`file_md5`) USING BTREE,
    KEY `file_status` (`status`) USING BTREE,
    KEY `file_user_time` (`from_user_id`,`add_time`) USING BTREE,
    KEY `file_storage_type` (`storage_type`) USING BTREE,
    KEY `copy_file_id` (`copy_file_id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '文件列表';

CREATE TABLE `yaf_file_local` (
	`id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `file_id` BIGINT UNSIGNED NOT NULL COMMENT '文件ID',
    `source_type` TINYINT NOT NULL COMMENT '文件来源：1=上传, 2=URL下载, 3=本地路径,4=OSS同步',
	`source_name` VARCHAR(255) NOT NULL  DEFAULT '' COMMENT '文件原始文件名',
    `oss_file_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '从OSS同步新增文件 OSS源文件ID',
    `local_path` VARCHAR(512) NOT NULL COMMENT '文件存储路径',

    `file_chunk_total` INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '分片总数,上传或URL下载时分片总数,<=1时不分片',
	`file_chunk_succ` INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '已完成分片数',
	`file_chunk_size` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '已上传文件大小,分片总数>0',

    `last_error` VARCHAR(255) NOT NULL DEFAULT '' COMMENT '最后一次同步错误消息,方便列表查看',
    KEY `file_id` (`file_id`) USING BTREE,
    KEY `file_source_type` (`source_type`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '本地文件';


CREATE TABLE `yaf_file_local_chunk` (
	`id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
	`file_id` BIGINT UNSIGNED NOT NULL COMMENT '文件ID',
	`chunk_index` INT UNSIGNED NOT NULL COMMENT '分片索引(从0开始)',
	`start_offset` BIGINT UNSIGNED NOT NULL COMMENT '分片起始字节',
    `chunk_md5` CHAR(32) NOT NULL DEFAULT '' COMMENT '文件md5',
	`upload_md5` CHAR(32) NOT NULL DEFAULT '' COMMENT '上传时文件hash,用户提交',
    `chunk_path` VARCHAR(512) NOT NULL COMMENT '分片文件存储路径',
    `file_size` BIGINT UNSIGNED NOT NULL COMMENT '预计文件大小',
	`complete_size` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '已完成文件实际大小',
    `status` TINYINT NOT NULL COMMENT '状态：1=正常, 2=已删除,3=未完成,4=失败,5=已合并,6=已清理',
    `add_time` BIGINT UNSIGNED NOT NULL COMMENT '添加时间',
    `change_time` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '修改时间',
    UNIQUE KEY `file_chunk_index` (`file_id`,`chunk_index`) USING BTREE,
    KEY `file_chunk_status` (`file_id`,`status`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '本地文件 上传 或 URL下载分块';


CREATE TABLE `yaf_file_oss` (
  `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  `file_id` BIGINT UNSIGNED NOT NULL COMMENT '关联 yaf_file.id',
  `object_key` VARCHAR(1024) NOT NULL COMMENT '对象键（路径）',
  `local_file_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT 'local 文件上传到OSS时的本地 yaf_file.id',
  `object_url` VARCHAR(2048) NOT NULL DEFAULT '' COMMENT '对外访问 URL（若可公开访问/或 CDN 地址）',
  `object_url_md5` CHAR(32) NOT NULL DEFAULT '' COMMENT '对外访问 URL md5 方便查询',
  `bucket` VARCHAR(255) NOT NULL DEFAULT '' COMMENT '目标 bucket 如果OSS支持',
  `region` VARCHAR(128) NOT NULL DEFAULT '' COMMENT '区域/endpoint 信息（可选）如果OSS支持',
  `size` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '对象大小（字节）',
  `last_error` VARCHAR(255) NOT NULL DEFAULT '' COMMENT '最后一次同步错误消息,方便列表查看',
    KEY `oss_provider_status` (`file_id`,`bucket`,`object_key`(100)) USING BTREE,
    KEY `oss_url_md5` (`object_url_md5`) USING BTREE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='OSS远程文件';

CREATE TABLE `yaf_file_log` (
	`id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
	`file_id` BIGINT UNSIGNED NOT NULL COMMENT '文件ID',
    `file_chunk_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '分片ID',
	`message` VARCHAR(1024) NOT NULL COMMENT '日志内容',
	`user_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '上传用户ID',
    `add_time` BIGINT UNSIGNED NOT NULL  COMMENT '添加时间',
    KEY `file_log_file_time` (`file_id`,`add_time`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '文件日志';


CREATE TABLE `yaf_file_user` (
	`id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `user_id` BIGINT UNSIGNED NOT NULL COMMENT '上传用户ID',
    `app_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '应用ID,0=系统,>0=具体应用',
    `file_id` BIGINT UNSIGNED NOT NULL COMMENT '文件ID',
    `status` TINYINT NOT NULL DEFAULT 1 COMMENT '状态：1=正常, 2=已删除',
	`source_url` VARCHAR(2048) NOT NULL  DEFAULT '' COMMENT '来源URL,默认同个用户的URL下载时相同URL只记录一次,但可多次下载多个记录',
    `source_md5` CHAR(32) NOT NULL  DEFAULT '' COMMENT '来源URL hash,上传时用户提交的hash',
    `add_time` BIGINT UNSIGNED NOT NULL COMMENT '添加时间',
    `delete_time` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT  '软删除时间',
    KEY `user_source_md5` (`user_id`,`app_id`,`source_md5`) USING BTREE,
    KEY `file_user_status_time` (`user_id`,`app_id`,`file_id`,`add_time`,`status`) USING BTREE,
    KEY `app_id` (`app_id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '用户文件列表';
