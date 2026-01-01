import { type AddressDataType } from '@shared/apis/user/profile';

import { MobileSchema, NumberParamSchema } from '@shared/types/base-schema';
import { z } from 'zod';

// 地址页面搜索参数验证
export const addressSearchSchema = z.object({
  action: z.enum(['add', 'edit']).optional(),
  id: NumberParamSchema,
});

export type AddressSearchParams = z.infer<typeof addressSearchSchema>;

// 地址数据类型
export type AddressData = AddressDataType;

// 地址表单数据验证
export const addressFormSchema = z.object({
  name: z.string().trim().min(1, '请输入收件人姓名'),
  mobile: MobileSchema.min(1, '请输入手机号码'),
  code: z.string().min(1, '请选择地区'),
  info: z.string().min(1, '请选择地区'),
  detail: z.string().trim().min(1, '请输入详细地址'),
});

export type AddressFormData = z.infer<typeof addressFormSchema>;
