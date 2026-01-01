import { PageDataParam } from '@shared/types/base-schema';
import { NumberParamSchema } from '@shared/types/base-schema';
import { z } from 'zod';

// 基础过滤器字段 schema
const SubAppRequestFilterBaseSchema = z.object({
    id: NumberParamSchema,
    sub_app_id: NumberParamSchema,
    request_type: NumberParamSchema,
    status: NumberParamSchema,
});

// URL 参数 schema，包含分页参数
export const SubAppRequestFilterParamSchema = SubAppRequestFilterBaseSchema.extend(PageDataParam);

// 表单 schema，使用 transform 来处理从字符串到最终类型的转换
export const SubAppRequestFilterFormSchema = z.object({
    id: NumberParamSchema,
    sub_app_id: NumberParamSchema,
    request_type: NumberParamSchema,
    status: NumberParamSchema,
});

export type SubAppRequestFilterParamType = z.infer<typeof SubAppRequestFilterParamSchema>;
