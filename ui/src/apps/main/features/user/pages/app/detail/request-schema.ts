import { NumberParamSchema } from '@shared/types/base-schema';
import { z } from 'zod';

// 基础过滤器字段 schema
const AppRequestListFilterBaseSchema = z.object({
    id: NumberParamSchema,
    status: NumberParamSchema,
    request_type: NumberParamSchema,
});

// URL 参数 schema，包含分页参数
export const AppRequestListFilterParamSchema = AppRequestListFilterBaseSchema.extend({
    page: z.coerce.number().optional(),
    limit: z.coerce.number().optional(),
});

// 表单过滤器 schema（不包含分页参数）
export const AppRequestListFilterFormSchema = z.object({
    id: NumberParamSchema,
    status: NumberParamSchema,
    request_type: NumberParamSchema,
});

export type AppRequestListFilterParamType = z.infer<typeof AppRequestListFilterParamSchema>;
