import { LimitDataParam } from '@shared/types/base-schema';
import { NumberParamSchema } from '@shared/types/base-schema';
import { z } from 'zod';

// 基础过滤器字段 schema
const RbacAuditLogFilterBaseSchema = z.object({
  user_id: NumberParamSchema,
  app_id: NumberParamSchema,
  user_ip: z.string().optional(),
  request_id: NumberParamSchema,
});

// URL 参数 schema，包含分页参数
export const RbacAuditLogFilterParamSchema = RbacAuditLogFilterBaseSchema.extend(LimitDataParam);

// 表单 schema，使用 coerce 来自动处理字符串到数字的转换
export const RbacAuditLogFilterFormSchema = z.object({
  user_id: NumberParamSchema.catch(undefined),
  app_id: NumberParamSchema.catch(undefined),
  user_ip: z.string().optional(),
  request_id: NumberParamSchema.catch(undefined),
});

export type RbacAuditLogFilterParamType = z.infer<typeof RbacAuditLogFilterParamSchema>;

// 偏移分页状态类型
export type RbacOffsetPaginationType = {
  pos: number | null
  limit: number
  forward: boolean
  more: boolean
  eq_pos: boolean
};
