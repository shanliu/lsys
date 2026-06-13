/**
 * 头像上传组件
 *
 * 流程：
 * 1. 用户选择图片文件
 * 2. 调用 /api/user/file/upload_create 创建上传任务（storage_type=local_public），响应包含 file_key
 * 3. 调用 /api/user/file/upload_data 上传文件数据
 * 4. 使用第2步返回的 file_key 调用 onKeyChange 通知父组件
 *
 * 显示规则：
 * - value 为空：显示占位符
 * - value 以 http 开头：直接作为 src 显示
 * - value 为 file_key：拼接 /api/user/file/share/{key} 显示
 */

import { useToast } from '@shared/contexts/toast-context';
import { getUserFileShareUrl } from '@shared/lib/apis/api_read';
import { cn } from '@shared/lib/utils';
import { formatServerError } from '@shared/lib/utils/format-utils';
import { userSelfFileUploadCreate, userSelfFileUploadData } from '@shared/apis/user/file';
import { Loader2, Upload, User } from 'lucide-react';
import React, { useRef, useState } from 'react';

interface AvatarUploadProps {
  /** 当前头像值：http URL 或 file_key */
  value?: string;
  /** 上传成功后返回新的 file_key */
  onKeyChange?: (key: string) => void;
  disabled?: boolean;
  className?: string;
}

export const AvatarUpload: React.FC<AvatarUploadProps> = ({
  value,
  onKeyChange,
  disabled,
  className,
}) => {
  const { error } = useToast();
  const inputRef = useRef<HTMLInputElement>(null);
  const [uploading, setUploading] = useState(false);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);

  const displayUrl = previewUrl ?? (value ? (value.startsWith('http') ? value : getUserFileShareUrl(value)) : null);
  const isDisabled = disabled || uploading;

  const handleFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    // 立即显示本地预览
    const localUrl = URL.createObjectURL(file);
    setPreviewUrl(localUrl);

    setUploading(true);
    try {      // 1. 创建上传任务（单分片），公开类型直接返回 file_key
      const createRes = await userSelfFileUploadCreate({
        file_name: file.name,
        chunks: [{ offset: 0, len: file.size }],
        storage_type: 'local_public',
      });
      if (!createRes.status || !createRes.response) {
        throw new Error(formatServerError(createRes));
      }
      const fileRefId = createRes.response.id;
      const fileKey = createRes.response.file_key;
      if (!fileKey) {
        throw new Error('创建上传任务未返回 file_key，请检查存储类型配置');
      }

      // 2. 上传数据
      const uploadRes = await userSelfFileUploadData(fileRefId, 0, file);
      if (!uploadRes.status) {
        throw new Error(formatServerError(uploadRes));
      }

      // 上传完成后切换为服务端 share URL 显示
      setPreviewUrl(getUserFileShareUrl(fileKey));
      onKeyChange?.(fileKey);
    } catch (err: any) {
      error(err?.message ?? '头像上传失败');
      // 还原预览并释放 object URL
      URL.revokeObjectURL(localUrl);
      setPreviewUrl(null);
    } finally {
      setUploading(false);
      // 清空 input，允许重新选择同一文件
      if (inputRef.current) inputRef.current.value = '';
      // 释放本地预览 object URL（成功时已切换为 share URL，释放无影响）
      URL.revokeObjectURL(localUrl);
    }
  };

  return (
    <div className={cn('flex flex-col items-center gap-2', className)}>
      {/* 外层容器不裁剪，内层负责圆形显示，input 在外层绝对定位 */}
      <div className="relative w-24 h-24">
        <div
          className={cn(
            'w-full h-full rounded-full border-2 border-dashed border-muted-foreground/40',
            'overflow-hidden flex items-center justify-center',
            'hover:border-primary hover:bg-muted/50 transition-colors',
            isDisabled ? 'opacity-60' : ''
          )}
        >
          {displayUrl ? (
            <img
              src={displayUrl}
              alt="用户头像"
              className="w-full h-full object-cover"
              onError={(e) => {
                e.currentTarget.src = '';
                e.currentTarget.style.display = 'none';
              }}
            />
          ) : (
            <User className="w-10 h-10 text-muted-foreground/50" />
          )}

          {/* 悬浮遮罩 */}
          {!uploading && (
            <div className={cn(
              'absolute inset-0 bg-black/40 flex flex-col items-center justify-center',
              'opacity-0 hover:opacity-100 transition-opacity',
              'text-white text-xs gap-1 pointer-events-none rounded-full'
            )}>
              <Upload className="w-5 h-5" />
              <span>上传</span>
            </div>
          )}

          {/* 上传中遮罩 */}
          {uploading && (
            <div className="absolute inset-0 bg-black/50 flex items-center justify-center pointer-events-none rounded-full">
              <Loader2 className="w-6 h-6 text-white animate-spin" />
            </div>
          )}
        </div>

        {/* 透明 input 在外层，不受 overflow-hidden 裁剪，覆盖整个圆形区域 */}
        {!isDisabled && (
          <input
            ref={inputRef}
            type="file"
            accept="image/*"
            onChange={handleFileChange}
            style={{
              position: 'absolute',
              inset: 0,
              width: '100%',
              height: '100%',
              opacity: 0,
              cursor: 'pointer',
              fontSize: 0,
              borderRadius: '9999px',
            }}
          />
        )}
      </div>

      <p className="text-xs text-muted-foreground">点击更换头像</p>
    </div>
  );
};
