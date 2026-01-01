import { type MobileDataType } from '@shared/apis/user/profile';

import { MobileSchema, NumberParamSchema } from '@shared/types/base-schema';
import { z } from 'zod';

// 手机号页面搜索参数验证
export const mobileSearchSchema = z.object({
  action: z.enum(['add', 'verify']).optional(),
  id: NumberParamSchema,
});

export type MobileSearchType = z.infer<typeof mobileSearchSchema>;

// 手机号数据类型
export type MobileData = MobileDataType;

// 手机号表单数据验证
export const mobileFormSchema = z.object({
  area_code: z.string().min(1, '请选择区号'),
  mobile: MobileSchema.min(1, '请输入手机号码'),
});

export type MobileFormType = z.infer<typeof mobileFormSchema>;

// 手机号验证码表单数据验证
export const mobileVerifyFormSchema = z.object({
  code: z.string().min(1, '请输入验证码').length(6, '验证码应为6位数字'),
});

export type MobileVerifyType = z.infer<typeof mobileVerifyFormSchema>;
