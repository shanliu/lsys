import { z } from 'zod'
import { BoolSchema } from '@shared/types/base-schema'

// 资源列表过滤表单 Schema
export const ResListFilterFormSchema = z.object({
  res_name: z.string().optional(),
  res_type: z.string().optional(),
  res_data: z.string().optional(),
})

export type ResListFilterFormType = z.infer<typeof ResListFilterFormSchema>

// 资源列表URL参数 Schema
export const ResListFilterParamSchema = z.object({
  res_name: z.string().optional(),
  res_type: z.string().optional(),
  res_data: z.string().optional(),
  page: z.coerce.number().optional(),
  limit: z.coerce.number().optional(),
  // 是否使用应用关联用户
  use_app_user: BoolSchema.optional(),
  // 用户参数（当use_app_user为false时必填）
  user_param: z.string().optional(),
})

export type ResListFilterParamType = z.infer<typeof ResListFilterParamSchema>

// 资源表单 Schema（添加/编辑）
export const ResFormSchema = z.object({
  res_name: z.string().min(1, '资源名称必须填写'),
  res_type: z.string().min(1, '资源类型必须填写'),
  res_data: z.string().optional(),
  // 新增时需要的用户模式字段
  use_app_user: BoolSchema.optional(),
  user_param: z.string().optional(),
})

export type ResFormType = z.infer<typeof ResFormSchema>

// 操作列表过滤表单 Schema
export const OpListFilterFormSchema = z.object({
  op_name: z.string().optional(),
  op_key: z.string().optional(),
})

export type OpListFilterFormType = z.infer<typeof OpListFilterFormSchema>

// 操作列表URL参数 Schema
export const OpListFilterParamSchema = z.object({
  op_name: z.string().optional(),
  op_key: z.string().optional(),
  page: z.coerce.number().optional(),
  limit: z.coerce.number().optional(),
  // 是否使用应用关联用户
  use_app_user: BoolSchema.optional(),
  // 用户参数（当use_app_user为false时必填）
  user_param: z.string().optional(),
})

export type OpListFilterParamType = z.infer<typeof OpListFilterParamSchema>

// 操作表单 Schema（添加/编辑）
export const OpFormSchema = z.object({
  op_name: z.string().min(1, '操作名称必须填写'),
  op_key: z.string().min(1, '操作标识必须填写'),
  // 新增时需要的用户模式字段
  use_app_user: BoolSchema.optional(),
  user_param: z.string().optional(),
})

export type OpFormType = z.infer<typeof OpFormSchema>
