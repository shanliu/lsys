import { NumberParamSchema } from '@shared/types/base-schema';
import { z } from 'zod';

// 基础过滤器字段 schema
const AdminAppRequestFilterBaseSchema = z.object({
  id: NumberParamSchema,
  app_id: NumberParamSchema,
  request_type: z.string().optional(),
  status: z.string().optional(),
});

// URL 参数 schema，包含分页参数
export const AdminAppRequestFilterParamSchema = AdminAppRequestFilterBaseSchema.extend({
  page: z.coerce.number().optional().catch(1),
  limit: z.coerce.number().optional().catch(20),
});

// 表单过滤器 schema
export const AdminAppRequestFilterFormSchema = z.object({
  id: NumberParamSchema.catch(undefined),
  app_id: NumberParamSchema.catch(undefined),
  request_type: z.string().optional(),
  status: z.string().optional(),
});

export type AdminAppRequestFilterParamType = z.infer<typeof AdminAppRequestFilterParamSchema>;
