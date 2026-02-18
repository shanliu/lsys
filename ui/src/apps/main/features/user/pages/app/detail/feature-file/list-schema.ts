import { LimitDataParam } from '@shared/types/base-schema';
import { z } from 'zod';

// 基础过滤器字段 schema
const FileListFilterBaseSchema = z.object({
    storage_type: z.string().optional(),
    file_md5: z.string().optional(),
    status: z.number().optional(),
});

// URL 参数 schema，包含分页参数
export const FileListFilterParamSchema = FileListFilterBaseSchema.extend(LimitDataParam);

// 表单过滤器 schema（不包含分页参数）
export const FileListFilterFormSchema = FileListFilterBaseSchema.extend({
    storage_type: z.string().optional().transform(val => val === '' ? undefined : val),
    file_md5: z.string().optional().transform(val => val === '' ? undefined : val),
    status: z.string().optional().transform(val => val === '' || val === undefined ? undefined : Number(val)),
});

export type FileListFilterParamType = z.infer<typeof FileListFilterParamSchema>;
