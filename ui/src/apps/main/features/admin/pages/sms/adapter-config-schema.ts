import { NumberParamSchema } from '@shared/types/base-schema';
import { z } from 'zod';

export const AdapterConfigParamSchema = z.object({
  type: z.string(),
});
export type AdapterConfigParamType = z.infer<typeof AdapterConfigParamSchema>;




// ===== 华为云短信配置 =====
export const HwSmsConfigFormSchema = z.object({
  name: z.string().min(1, '配置名称不能为空'),
  url: z.string().url('请输入有效的URL'),
  app_key: z.string().min(1, 'App Key不能为空'),
  app_secret: z.string().min(1, 'App Secret不能为空'),
  callback_key: z.string().optional(),
  limit: NumberParamSchema.refine(val => val == null || val >= 0, '限制数量不能小于0'),
})

export type HwSmsConfigFormType = z.infer<typeof HwSmsConfigFormSchema>

// ===== 阿里云短信配置 =====
export const AliSmsConfigFormSchema = z.object({
  name: z.string().min(1, '配置名称不能为空'),
  access_id: z.string().min(1, 'Access ID不能为空'),
  access_secret: z.string().min(1, 'Access Secret不能为空'),
  region: z.string().min(1, '地域不能为空'),
  callback_key: z.string().optional(),
  limit: NumberParamSchema.refine(val => val == null || val >= 0, '限制数量不能小于0'),
})

export type AliSmsConfigFormType = z.infer<typeof AliSmsConfigFormSchema>

// ===== 腾讯云短信配置 =====
export const TencentSmsConfigFormSchema = z.object({
  name: z.string().min(1, '配置名称不能为空'),
  region: z.string().min(1, '地域不能为空'),
  secret_id: z.string().min(1, 'Secret ID不能为空'),
  secret_key: z.string().min(1, 'Secret Key不能为空'),
  sms_app_id: z.string().min(1, 'SMS App ID不能为空'),
  callback_key: z.string().optional(),
  limit: NumberParamSchema.refine(val => val == null || val >= 0, '限制数量不能小于0'),
})

export type TencentSmsConfigFormType = z.infer<typeof TencentSmsConfigFormSchema>

// ===== 京东云短信配置 =====
export const JdSmsConfigFormSchema = z.object({
  name: z.string().min(1, '配置名称不能为空'),
  region: z.string().min(1, '地域不能为空'),
  access_key: z.string().min(1, 'Access Key不能为空'),
  access_secret: z.string().min(1, 'Access Secret不能为空'),
  limit: NumberParamSchema.refine(val => val == null || val >= 0, '限制数量不能小于0'),
})

export type JdSmsConfigFormType = z.infer<typeof JdSmsConfigFormSchema>

// ===== 网易云短信配置 =====
export const NeteaseSmsConfigFormSchema = z.object({
  name: z.string().min(1, '配置名称不能为空'),
  access_key: z.string().min(1, 'Access Key不能为空'),
  access_secret: z.string().min(1, 'Access Secret不能为空'),
  limit: NumberParamSchema.refine(val => val == null || val >= 0, '限制数量不能小于0'),
})

export type NeteaseSmsConfigFormType = z.infer<typeof NeteaseSmsConfigFormSchema>

// ===== 容联云短信配置 =====
export const CloopenSmsConfigFormSchema = z.object({
  name: z.string().min(1, '配置名称不能为空'),
  account_sid: z.string().min(1, 'Account SID不能为空'),
  account_token: z.string().min(1, 'Account Token不能为空'),
  sms_app_id: z.string().min(1, 'SMS App ID不能为空'),
  callback_key: z.string().optional(),
  limit: NumberParamSchema.refine(val => val == null || val >= 0, '限制数量不能小于0'),
})

export type CloopenSmsConfigFormType = z.infer<typeof CloopenSmsConfigFormSchema>

