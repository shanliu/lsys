import { LimitDataParam } from '@shared/types/base-schema';
import { z } from 'zod';

// 文件内容搜索类型
export const CONTENT_SEARCH_TYPES = [
    { value: '', label: '无' },
    { value: 'file_md5', label: '文件MD5' },
    { value: 'source_url', label: '来源URL' },
    { value: 'url', label: '本地URL' },
] as const;

// 基础过滤器字段 schema
const FileListFilterBaseSchema = z.object({
    status: z.number().optional(),
    tag_name: z.string().optional(),
    content_type: z.string().optional(),
    content_value: z.string().optional(),
});

// URL 参数 schema，包含分页参数 + 视图模式参数
export const FileListFilterParamSchema = FileListFilterBaseSchema.extend(LimitDataParam).extend({
    // 视图模式
    mode: z.enum(["normal", "downloading", "lineage"]).optional(),
    // 关联文件视图参数
    source_id: z.number().optional(),
    rel_type: z.number().nullable().optional(),
});

// 表单过滤器 schema（不包含分页/视图参数）
export const FileListFilterFormSchema = FileListFilterBaseSchema.extend({
    status: z.string().optional().transform(val => val === '' || val === undefined ? undefined : Number(val)),
    tag_name: z.string().optional().transform(val => val === '' ? undefined : val),
    content_type: z.string().optional().transform(val => val === '' ? undefined : val),
    content_value: z.string().optional().transform(val => val === '' ? undefined : val),
});

export type FileListFilterParamType = z.infer<typeof FileListFilterParamSchema>;
export type FileListViewMode = "normal" | "downloading" | "lineage";
