import { PageDataParam } from '@shared/types/base-schema';
import { NumberParamSchema } from '@shared/types/base-schema';
import { z } from 'zod';

// 基础过滤器字段 schema
const AppListFilterBaseSchema = z.object({
  status: NumberParamSchema,
  client_id: z.string().optional().nullable(),
  parent_app_id: NumberParamSchema,
  app_id: NumberParamSchema,
});

// URL 参数 schema，包含分页参数
export const AppListFilterParamSchema = AppListFilterBaseSchema.extend(PageDataParam);

// 表单 schema，使用通用数字参数 Schema，避免 '' -> 0 等问题
export const AppListFilterFormSchema = z.object({
  status: NumberParamSchema,
  client_id: z.string().optional(),
  parent_app_id: NumberParamSchema,
  app_id: NumberParamSchema,
});

export type AppListFilterParamType = z.infer<typeof AppListFilterParamSchema>;
