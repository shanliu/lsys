import { NumberParamSchema } from '@shared/types/base-schema';
import { z } from 'zod';

// 基础过滤器字段 schema
const AppNotifyListFilterBaseSchema = z.object({
    notify_method: z.string().optional(),
    notify_status: z.string().optional(),
});

// URL 参数 schema，包含分页参数
export const AppNotifyListFilterParamSchema = AppNotifyListFilterBaseSchema.extend({
    pos: NumberParamSchema,
    limit: NumberParamSchema,
    forward: z.coerce.boolean().optional(),
    eq_pos: z.coerce.boolean().optional(),
});

// 表单过滤器 schema（不包含分页参数）
export const AppNotifyListFilterFormSchema = z.object({
    notify_method: z.string().optional(),
    notify_status: z.string().optional(),
});

export type AppNotifyListFilterParamType = z.infer<typeof AppNotifyListFilterParamSchema>;
