import { PageDataParam } from '@shared/types/base-schema'
import { z } from 'zod'

// 资源列表过滤表单 Schema
export const ResListFilterFormSchema = z.object({
  res_name: z.string().optional().transform(val => val === '' ? undefined : val),
  res_type: z.string().optional().transform(val => val === '' ? undefined : val),
  res_data: z.string().optional().transform(val => val === '' ? undefined : val),
})

export type ResListFilterFormType = z.infer<typeof ResListFilterFormSchema>

// 资源列表URL参数 Schema
export const ResListFilterParamSchema = z.object({
  res_name: z.string().optional(),
  res_type: z.string().optional(),
  res_data: z.string().optional(),
}).extend(PageDataParam)

export type ResListFilterParamType = z.infer<typeof ResListFilterParamSchema>

// 资源表单 Schema（添加/编辑）
export const ResFormSchema = z.object({
  res_name: z.string().min(1, '资源名称必须填写'),
  res_type: z.string().min(1, '资源类型必须填写'),
  res_data: z.string().optional(),
})

export type ResFormType = z.infer<typeof ResFormSchema>

// 操作列表过滤表单 Schema
export const OpListFilterFormSchema = z.object({
  op_name: z.string().optional().transform(val => val === '' ? undefined : val),
  op_key: z.string().optional().transform(val => val === '' ? undefined : val),
})

export type OpListFilterFormType = z.infer<typeof OpListFilterFormSchema>

// 操作列表URL参数 Schema
export const OpListFilterParamSchema = z.object({
  op_name: z.string().optional(),
  op_key: z.string().optional(),
}).extend(PageDataParam)

export type OpListFilterParamType = z.infer<typeof OpListFilterParamSchema>

// 操作表单 Schema（添加/编辑）
export const OpFormSchema = z.object({
  op_name: z.string().min(1, '操作名称必须填写'),
  op_key: z.string().min(1, '操作标识必须填写'),
})

export type OpFormType = z.infer<typeof OpFormSchema>
