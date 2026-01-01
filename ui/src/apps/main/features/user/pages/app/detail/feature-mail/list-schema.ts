import { LimitDataParam } from '@shared/types/base-schema';
import { z } from 'zod';

// 基础过滤器字段 schema
const MailListFilterBaseSchema = z.object({
  status: z.number().optional(),
  tpl_id: z.string().optional(),
  to_mail: z.string().optional(),
  snid: z.string().optional(),
});

// URL 参数 schema，包含分页参数
export const MailListFilterParamSchema = MailListFilterBaseSchema.extend(LimitDataParam);

// 表单过滤器 schema（不包含分页参数）
export const MailListFilterFormSchema = MailListFilterBaseSchema.extend({
  status: z.string().optional().transform(val => val === '' || val === undefined ? undefined : Number(val)),
  tpl_id: z.string().optional().transform(val => val === '' ? undefined : val),
  to_mail: z.string().optional().transform(val => val === '' ? undefined : val),
  snid: z.string().optional().transform(val => val === '' ? undefined : val),
});

export type MailListFilterParamType = z.infer<typeof MailListFilterParamSchema>;
