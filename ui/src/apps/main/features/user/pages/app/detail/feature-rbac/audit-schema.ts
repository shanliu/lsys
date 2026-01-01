import { LimitDataParam } from '@shared/types/base-schema';
import { z } from 'zod';

// 基础过滤器字段 schema
const AuditListFilterBaseSchema = z.object({
  user_ip: z.string().optional(),
  device_id: z.string().optional(),
  request_id: z.string().optional(),
  check_result: z.string().optional(),
});

// URL 参数 schema，包含分页参数
export const AuditListFilterParamSchema = AuditListFilterBaseSchema.extend(LimitDataParam);

// 表单过滤器 schema（不包含分页参数）
export const AuditListFilterFormSchema = AuditListFilterBaseSchema.extend({
  user_ip: z.string().optional().transform(val => val === '' ? undefined : val),
  device_id: z.string().optional().transform(val => val === '' ? undefined : val),
  request_id: z.string().optional().transform(val => val === '' ? undefined : val),
  check_result: z.string().optional().transform(val => val === '' || val === undefined ? undefined : val),
});

export type AuditListFilterParamType = z.infer<typeof AuditListFilterParamSchema>;
