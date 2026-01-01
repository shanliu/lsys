import { LimitDataParam } from '@shared/types/base-schema';
import { z } from 'zod';

// 基础过滤器字段 schema
const SmsListFilterBaseSchema = z.object({
  status: z.number().optional(),
  tpl_key: z.string().optional(),
  mobile: z.string().optional(),
  snid: z.string().optional(),
});

// URL 参数 schema，包含分页参数
export const SmsListFilterParamSchema = SmsListFilterBaseSchema.extend(LimitDataParam);

// 表单过滤器 schema（不包含分页参数）
export const SmsListFilterFormSchema = SmsListFilterBaseSchema.extend({
  status: z.string().optional().transform(val => val === '' || val === undefined ? undefined : Number(val)),
  tpl_key: z.string().optional().transform(val => val === '' ? undefined : val),
  mobile: z.string().optional().transform(val => val === '' ? undefined : val),
  snid: z.string().optional().transform(val => val === '' ? undefined : val),
});

export type SmsListFilterParamType = z.infer<typeof SmsListFilterParamSchema>;
