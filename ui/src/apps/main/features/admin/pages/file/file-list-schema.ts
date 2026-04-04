import { LimitDataParam } from '@shared/types/base-schema'
import { z } from 'zod'

export const CONTENT_SEARCH_TYPES = [
    { value: '', label: '无' },
    { value: 'file_md5', label: '文件MD5' },
    { value: 'source_url', label: '来源URL' },
    { value: 'url', label: '本地URL' },
] as const



export const AdminFileListFilterBaseSchema = z.object({
    status: z.number().optional(),
    tag_name: z.string().optional(),
    content_type: z.string().optional(),
    content_value: z.string().optional(),
})

export const AdminFileListFilterParamSchema = AdminFileListFilterBaseSchema.extend({
    ...LimitDataParam,
})

export const AdminFileListFilterFormSchema = AdminFileListFilterBaseSchema.extend({
    status: z.string().optional().transform(val => val === '' || val === undefined ? undefined : Number(val)),
    tag_name: z.string().optional().transform(val => val === '' ? undefined : val),
    content_type: z.string().optional().transform(val => val === '' ? undefined : val),
    content_value: z.string().optional().transform(val => val === '' ? undefined : val),
})

export type AdminFileListFilterParamType = z.infer<typeof AdminFileListFilterParamSchema>
