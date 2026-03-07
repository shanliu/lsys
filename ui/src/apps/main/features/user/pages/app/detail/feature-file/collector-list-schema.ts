import { PageDataParam } from '@shared/types/base-schema';
import { z } from 'zod';

// 基础过滤器字段 schema
const CollectorListFilterBaseSchema = z.object({
    status: z.number().optional(),
});

// URL 参数 schema，包含分页参数
export const CollectorListFilterParamSchema = CollectorListFilterBaseSchema.extend(PageDataParam);

// 表单过滤器 schema（不包含分页参数）
export const CollectorListFilterFormSchema = CollectorListFilterBaseSchema.extend({
    status: z.string().optional().transform(val => val === '' || val === undefined ? undefined : Number(val)),
});

export type CollectorListFilterParamType = z.infer<typeof CollectorListFilterParamSchema>;
