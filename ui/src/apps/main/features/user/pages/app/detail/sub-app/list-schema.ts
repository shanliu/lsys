import { NumberParamSchema, PageDataParam } from '@shared/types/base-schema';
import { z } from 'zod';


// 基础过滤器字段 schema
const SubAppListFilterBaseSchema = z.object({
    sub_app_id: NumberParamSchema,
    status: NumberParamSchema,
});

// URL 参数 schema，包含分页参数
export const SubAppListFilterParamSchema = SubAppListFilterBaseSchema.extend(PageDataParam);

// 表单 schema，使用 transform 来处理从字符串到最终类型的转换
export const SubAppListFilterFormSchema = z.object({
    sub_app_id: NumberParamSchema,
    status: NumberParamSchema,
});

export type SubAppListFilterParamType = z.infer<typeof SubAppListFilterParamSchema>;
