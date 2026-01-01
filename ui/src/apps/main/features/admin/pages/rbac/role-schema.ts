import { PageDataParam } from '@shared/types/base-schema'
import { z } from 'zod'

// 基础过滤器字段 schema
const RoleListFilterBaseSchema = z.object({
  role_name: z.string().optional(),
  role_key: z.string().optional(),
  user_range: z.number().optional(),
  res_range: z.number().optional(),
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
  role_key: z.string().min(1, '角色标识必须填写'),
  user_range: z.coerce.number().min(1, '请选择用户范围'),
  res_range: z.coerce.number().min(1, '请选择资源范围'),
})

export type RoleFormType = z.infer<typeof RoleFormSchema>

// 角色用户添加表单 Schema
export const RoleUserAddFormSchema = z.object({
  user_id: z.coerce.number().min(1, '请选择用户'),
  timeout: z.coerce.number().optional().default(0),
})

export type RoleUserAddFormType = z.infer<typeof RoleUserAddFormSchema>
