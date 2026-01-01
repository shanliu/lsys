import { type EmailDataType } from '@shared/apis/user/profile';
import { NumberParamSchema } from '@shared/types/base-schema';
import { z } from 'zod';

// 邮箱页面搜索参数验证
export const emailSearchSchema = z.object({
  action: z.enum(['add', 'verify']).optional(),
  id: NumberParamSchema,
});

export type EmailSearchType = z.infer<typeof emailSearchSchema>;

// 邮箱数据类型
export type EmailData = EmailDataType;

// 邮箱表单数据验证
export const emailFormSchema = z.object({
  email: z.string().trim().min(1, '请输入邮箱地址').email('请输入正确的邮箱地址'),
});

export type EmailFormType = z.infer<typeof emailFormSchema>;

// 邮箱验证码表单数据验证
export const emailVerifyFormSchema = z.object({
  code: z.string().trim().min(1, '请输入验证码').length(6, '验证码应为6位数字'),
});

export type EmailVerifyType = z.infer<typeof emailVerifyFormSchema>;
