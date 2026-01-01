import { BoolSchema, PageDataParam } from '@shared/types/base-schema'
import { z } from 'zod'

// 基础过滤器字段 schema
const RoleListFilterBaseSchema = z.object({
  role_name: z.string().optional(),
  role_key: z.string().optional(),
  user_range: z.number().optional(),
  res_range: z.number().optional(),
  // 是否使用应用关联用户
  use_app_user: BoolSchema.optional(),
  // 用户参数（当use_app_user为false时必填）
  user_param: z.string().optional(),
})

// URL 参数 schema，包含分页参数
export const RoleListFilterParamSchema = RoleListFilterBaseSchema.extend(PageDataParam)

// 表单过滤器 schema（不包含分页参数）
export const RoleListFilterFormSchema = z.object({
  role_name: z.string().optional().transform(val => val === '' ? undefined : val),
  role_key: z.string().optional().transform(val => val === '' ? undefined : val),
  user_range: z.string().optional().transform(val => val === '' || val === undefined ? undefined : Number(val)),
  res_range: z.string().optional().transform(val => val === '' || val === undefined ? undefined : Number(val)),
})

export type RoleListFilterFormType = z.infer<typeof RoleListFilterFormSchema>
export type RoleListFilterParamType = z.infer<typeof RoleListFilterParamSchema>

// 角色表单 Schema（添加/编辑）
export const RoleFormSchema = z.object({
  role_name: z.string().min(1, '角色名称必须填写'),
  role_key: z.string(),
  user_range: z.coerce.number().min(1, '请选择用户范围'),
  res_range: z.coerce.number().min(1, '请选择资源范围'),
  // 新增时需要的用户模式字段
  use_app_user: BoolSchema.optional(),
  user_param: z.string().optional(),
})

export type RoleFormType = z.infer<typeof RoleFormSchema>

// 角色用户表单 Schema
export const RoleUserFormSchema = z.object({
  use_app_user: BoolSchema,
  user_param: z.string(),
  timeout: z.coerce.number().optional().default(0),
})

export type RoleUserFormType = z.infer<typeof RoleUserFormSchema>
