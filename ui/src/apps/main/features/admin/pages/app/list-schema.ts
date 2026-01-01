import { NumberParamSchema } from '@shared/types/base-schema';
import { z } from 'zod';

// 基础过滤器字段 schema
const AdminAppListFilterBaseSchema = z.object({
  app_name: z.string().optional(),
  status: NumberParamSchema,
  user_id: NumberParamSchema,
  client_id: z.string().optional(),
  parent_app_id: NumberParamSchema,
  app_id: NumberParamSchema,
});

// URL 参数 schema，包含分页参数
export const AdminAppListFilterParamSchema = AdminAppListFilterBaseSchema.extend({
  page: z.coerce.number().optional().catch(1),
  limit: z.coerce.number().optional().catch(20),
  detail: z.coerce.number().optional(),
});

// 表单 schema，使用 coerce 来自动处理字符串到数字的转换
export const AdminAppListFilterFormSchema = z.object({
  app_name: z.string().optional(),
  status: NumberParamSchema.catch(undefined),
  user_id: NumberParamSchema.catch(undefined),
  client_id: z.string().optional(),
  parent_app_id: NumberParamSchema.catch(undefined),
  app_id: NumberParamSchema.catch(undefined),
});

export type AdminAppListFilterParamType = z.infer<typeof AdminAppListFilterParamSchema>;
