import { LimitDataParam } from '@shared/types/base-schema'
import { z } from 'zod'

export const CONTENT_SEARCH_TYPES = [
    { value: '', label: '无' },
    { value: 'file_md5', label: '文件MD5' },
    { value: 'source_url', label: '来源URL' },
    { value: 'url', label: '本地URL' },
] as const

// 列表模式
export const LIST_MODE = {
    NORMAL: 'normal',           // 普通文件列表
    DOWNLOADING: 'downloading', // 下载中文件列表
    LINEAGE: 'lineage',        // 关联文件列表
} as const

export type ListModeType = typeof LIST_MODE[keyof typeof LIST_MODE]

// 关系类型选项
export const REL_TYPE_OPTIONS = [
    { value: '', label: '全部' },
    { value: '1', label: '拷贝' },
    { value: '2', label: '转换' },
    { value: '3', label: 'OSS同步' },
] as const

// 下载状态选项
export const DOWNLOAD_STATUS_OPTIONS = [
    { value: '', label: '全部' },
    { value: 'true', label: '下载中' },
    { value: 'false', label: '排队中' },
] as const

// 基础过滤 Schema（普通模式）
export const AdminFileListFilterBaseSchema = z.object({
    mode: z.string().optional().default('normal'),
    status: z.number().optional(),
    tag_name: z.string().optional(),
    content_type: z.string().optional(),
    content_value: z.string().optional(),
})

// 下载模式过滤 Schema
export const AdminFileDownloadingFilterBaseSchema = z.object({
    mode: z.literal('downloading'),
    is_downloading: z.boolean().optional(),
    user_id: z.number().optional(),
})

// 关联模式过滤 Schema
export const AdminFileLineageFilterBaseSchema = z.object({
    mode: z.literal('lineage'),
    source_file_id: z.number(),           // 源文件的 file_ref_id
    source_file_name: z.string().optional(), // 源文件名（用于显示）
    rel_type: z.number().optional(),      // 关系类型
    storage_type: z.string().optional(),  // 存储类型
})

// 联合 Schema
export const AdminFileListFilterParamSchema = z.discriminatedUnion('mode', [
    AdminFileListFilterBaseSchema.extend({ mode: z.literal('normal'), ...LimitDataParam }),
    AdminFileDownloadingFilterBaseSchema.extend({ ...LimitDataParam }),
    AdminFileLineageFilterBaseSchema.extend({ ...LimitDataParam }),
])

// 表单 Schema（用于表单验证）
export const AdminFileListFilterFormSchema = z.object({
    mode: z.string().optional(),
    // 普通模式字段
    status: z.string().optional().transform(val => val === '' || val === undefined ? undefined : Number(val)),
    tag_name: z.string().optional().transform(val => val === '' ? undefined : val),
    content_type: z.string().optional().transform(val => val === '' ? undefined : val),
    content_value: z.string().optional().transform(val => val === '' ? undefined : val),
    // 下载模式字段
    is_downloading: z.string().optional().transform(val => {
        if (val === '' || val === undefined) return undefined
        return val === 'true'
    }),
    user_id: z.string().optional().transform(val => val === '' || val === undefined ? undefined : Number(val)),
    // 关联模式字段
    rel_type: z.string().optional().transform(val => val === '' || val === undefined ? undefined : Number(val)),
    storage_type: z.string().optional().transform(val => val === '' ? undefined : val),
})

export type AdminFileListFilterParamType = z.infer<typeof AdminFileListFilterParamSchema>
