import { LimitDataParam } from '@shared/types/base-schema';
import { NumberParamSchema } from '@shared/types/base-schema';
import { z } from 'zod';

// 基础过滤器字段 schema
const SmsLogFilterBaseSchema = z.object({
  tpl_key: z.string().optional(),
  status: NumberParamSchema,
  body_id: NumberParamSchema,
  snid: z.string().optional(),
  mobile: z.string().optional(),
});

// URL 参数 schema，包含分页参数
export const SmsLogFilterParamSchema = SmsLogFilterBaseSchema.extend(LimitDataParam);

// 表单过滤器 schema（不包含分页参数）
export const SmsLogFilterFormSchema = z.object({
  tpl_key: z.string().optional(),
  status: NumberParamSchema.catch(undefined),
  body_id: NumberParamSchema.catch(undefined),
  snid: z.string().optional(),
  mobile: z.string().optional(),
});

export type SmsLogFilterParamType = z.infer<typeof SmsLogFilterParamSchema>;
