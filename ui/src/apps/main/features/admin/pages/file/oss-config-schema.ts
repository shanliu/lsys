import { NumberParamSchema } from '@shared/types/base-schema';
import { z } from 'zod';

// ===== 阿里云 OSS 配置 =====
export const AliyunOssConfigSchema = z.object({
  endpoint: z.string().min(1, 'Endpoint 不能为空'),
  bucket: z.string().min(1, 'Bucket 不能为空'),
  access_key: z.string().min(1, 'Access Key 不能为空'),
  secret_key: z.string().min(1, 'Secret Key 不能为空'),
  timeout_secs: NumberParamSchema.optional(),
});

export type AliyunOssConfigType = z.infer<typeof AliyunOssConfigSchema>;

// ===== 腾讯云 COS 配置 =====
export const TencentCosConfigSchema = z.object({
  endpoint: z.string().min(1, 'Endpoint 不能为空'),
  bucket: z.string().min(1, 'Bucket 不能为空'),
  access_key: z.string().min(1, 'Access Key 不能为空'),
  secret_key: z.string().min(1, 'Secret Key 不能为空'),
  timeout_secs: NumberParamSchema.optional(),
});

export type TencentCosConfigType = z.infer<typeof TencentCosConfigSchema>;

// ===== AWS S3 / Minio 配置 =====
export const AwsS3ConfigSchema = z.object({
  endpoint: z.string().optional().or(z.literal('')),
  region: z.string().min(1, 'Region 不能为空'),
  bucket: z.string().min(1, 'Bucket 不能为空'),
  access_key: z.string().min(1, 'Access Key 不能为空'),
  secret_key: z.string().min(1, 'Secret Key 不能为空'),
  timeout_secs: NumberParamSchema.optional(),
});

export type AwsS3ConfigType = z.infer<typeof AwsS3ConfigSchema>;

// ===== 统一表单 Schema =====
export const OssConfigFormSchema = z.object({
  name: z.string().min(1, '配置名称不能为空'),
  config_key: z.string().min(1, '配置标识不能为空').max(32, '配置标识最长32字符')
    .regex(/^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$/, '只允许小写字母、数字、连字符，不能以连字符开头或结尾'),
  provider_type: z.string().min(1, '厂商类型不能为空'),
  
  // 厂商特定的配置字段，将被平铺在表单中并在提交时包装进 provider_config
  // 这里使用一个包含所有可能字段的联合类型，或者干脆在 Schema 中定义为 Record<string, any>
  // 为了更好的类型检查，我们在 Drawer 中根据 provider_type 来渲染不同的字段
  endpoint: z.string().optional(),
  bucket: z.string().optional(),
  access_key: z.string().optional(),
  secret_key: z.string().optional(),
  region: z.string().optional(),
  timeout_secs: NumberParamSchema.optional(),
});

export type OssConfigFormType = z.infer<typeof OssConfigFormSchema>;
