import { LimitDataParam } from '@shared/types/base-schema';
import { NumberParamSchema } from '@shared/types/base-schema';
import { z } from 'zod';

// 基础过滤器字段 schema
const LoginLogFilterBaseSchema = z.object({
  login_type: z.string().optional(),
  login_account: z.string().optional(),
  login_ip: z.string().optional(),
  is_login: NumberParamSchema,
});

// URL 参数 schema，包含分页参数
export const LoginLogFilterParamSchema = LoginLogFilterBaseSchema.extend(LimitDataParam);

// 表单过滤器 schema（不包含分页参数）
export const LoginLogFilterFormSchema = z.object({
  login_type: z.string().optional(),
  login_account: z.string().optional(),
  login_ip: z.string().optional(),
  is_login: NumberParamSchema.catch(undefined),
});
export type LoginLogFilterFormType = z.infer<typeof LoginLogFilterFormSchema>;

export type LoginLogFilterParamType = z.infer<typeof LoginLogFilterParamSchema>
