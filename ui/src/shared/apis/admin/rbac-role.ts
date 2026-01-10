import { authApi } from "@shared/lib/apis/api_auth";
import { cleanEmptyStringParams, parseResData } from "@shared/lib/apis/utils";
import { ApiResult } from "@shared/types/apis-rest";
import { BoolSchema, LimitParam, LimitResSchema, PageParam, PageResSchema, UnixTimestampSchema, UserDataResSchema } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

/**
 * RBAC 角色管理 API
 * 对应文档: docs/api/system/rbac/role/
 */

// 添加角色
export const RoleAddParamSchema = z.object({
    /** 用户范围: 1-指定用户, 2-任意用户 */
    user_range: z.coerce.number().min(1).max(2),
    /** 资源范围: 1-包含指定授权, 2-访问任意资源, 3-禁止指定授权 */
    res_range: z.coerce.number().min(1).max(3),
    /** 角色名称 */
    role_name: z.string().min(1, "角色名称不能为空"),
    /** 角色标识（会话角色时必填） */
    role_key: z.string(),
});
export type RoleAddParamType = z.infer<typeof RoleAddParamSchema>;

export const RoleAddResSchema = z.object({
    /** 新创建的角色ID */
    id: z.coerce.number(),
});
export type RoleAddResType = z.infer<typeof RoleAddResSchema>;

/**
 * 添加角色
 * @description 创建一个新的RBAC角色，包含用户范围和资源范围设置
 */
export async function roleAdd(
    param: RoleAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<RoleAddResType>> {
    const { data } = await authApi().post("/api/system/rbac/role/add", param, config);
    return parseResData(data, RoleAddResSchema);
}

// 角色列表
export const RoleListParamSchema = z.object({
    /** 角色名称 */
    role_name: z.string().optional(),
    /** 角色标识 */
    role_key: z.string().optional(),
    /** 用户数据 */
    user_data: z.string().optional(),
    /** 是否统计用户数量 */
    user_count: BoolSchema.optional().default(false),
    res_count: BoolSchema.optional().nullable(),
    res_op_count: BoolSchema.optional().nullable(),
    /** ID列表 */
    ids: z.array(z.coerce.number()).optional(),
    /** 用户范围 */
    user_range: z.coerce.number().optional().nullable(),
    /** 资源范围 */
    res_range: z.coerce.number().optional().nullable(),
    ...PageParam,
});
export type RoleListParamType = z.infer<typeof RoleListParamSchema>;

export const RoleItemSchema = z.object({
    /** 修改时间 */
    change_time: UnixTimestampSchema,
    /** 修改用户ID */
    change_user_id: z.coerce.number(),
    /** 角色ID */
    id: z.coerce.number(),
    /** 资源范围: 1-包含指定授权, 2-访问任意资源, 3-禁止指定授权 */
    res_range: z.coerce.number(),
    /** 角色标识 */
    role_key: z.string(),
    /** 角色名称 */
    role_name: z.string(),
    /** 状态 */
    status: z.coerce.number(),
    /** 用户数量 */
    user_count: z.coerce.number().optional().nullable(),
    /** 资源数量 */
    res_count: z.coerce.number().optional().nullable(),
    /** 资源数量 */
    res_op_count: z.coerce.number().optional().nullable(),
    /** 用户数据 */
    user_data: UserDataResSchema.nullable(),
    /** 用户ID（系统级角色时可能不存在） */
    user_id: z.coerce.number().optional().nullable(),
    /** 用户范围: 1-指定用户, 2-任意用户 */
    user_range: z.coerce.number(),
});
export const RoleListResSchema = z.object({
    data: z.array(RoleItemSchema),
    ...PageResSchema,
});
export type RoleItemType = z.infer<typeof RoleItemSchema>;
export type RoleListResType = z.infer<typeof RoleListResSchema>;

/**
 * 获取角色列表
 * @description 分页获取系统中的RBAC角色列表，支持多种筛选条件
 */
export async function roleList(
    param: RoleListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<RoleListResType>> {
    const cleanedParam = cleanEmptyStringParams(param, ['role_name', 'role_key', 'user_data']);
    const { data } = await authApi().post("/api/system/rbac/role/list", cleanedParam, config);
    return parseResData(data, RoleListResSchema);
}

// 编辑角色
export const RoleEditParamSchema = z.object({
    /** 角色ID */
    role_id: z.coerce.number().min(1, "角色ID必须大于0"),
    /** 角色名称 */
    role_name: z.string().min(1, "角色名称不能为空"),
    /** 角色标识 */
    role_key: z.string().min(1, "角色标识不能为空"),
});
export type RoleEditParamType = z.infer<typeof RoleEditParamSchema>;

/**
 * 编辑角色
 * @description 修改现有角色的基本信息，包括名称、标识和权限范围
 */
export async function roleEdit(
    param: RoleEditParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/rbac/role/edit", param, config);
    return data;
}

// 删除角色
export const RoleDeleteParamSchema = z.object({
    /** 角色ID */
    role_id: z.coerce.number().min(1, "角色ID必须大于0"),
});
export type RoleDeleteParamType = z.infer<typeof RoleDeleteParamSchema>;

/**
 * 删除角色
 * @description 删除指定的RBAC角色，同时会删除相关的权限和用户关联
 */
export async function roleDelete(
    param: RoleDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/rbac/role/delete", param, config);
    return data;
}

// 可用用户列表
export const RoleAvailableUserParamSchema = z.object({
    /** 搜索关键词 */
    user_data: z.string().optional(),
    ...LimitParam,
});
export type RoleAvailableUserParamType = z.infer<typeof RoleAvailableUserParamSchema>;

export const UserItemSchema = z.object({
    /** 用户ID */
    id: z.coerce.number(),
    /** 应用ID */
    app_id: z.string().optional(),
    /** 用户数据ID */
    user_data: z.string(),
    /** 用户名 */
    user_account: z.string(),
    /** 昵称 */
    user_nickname: z.string(),
});

export const RoleAvailableUserResSchema = z.object({
    data: z.array(UserItemSchema),
    ...LimitResSchema,
});
export type UserItemType = z.infer<typeof UserItemSchema>;
export type RoleAvailableUserResType = z.infer<typeof RoleAvailableUserResSchema>;

/**
 * 获取角色可用用户列表
 * @description 获取可以分配给指定角色的用户列表
 */
export async function roleAvailableUser(
    param: RoleAvailableUserParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<RoleAvailableUserResType>> {
    const cleanedParam = cleanEmptyStringParams(param, ['user_data']);
    const { data } = await authApi().post("/api/system/rbac/role/available_user", cleanedParam, config);
    return parseResData(data, RoleAvailableUserResSchema);
}

// 角色用户数据
export const RoleUserDataParamSchema = z.object({
    /** 角色ID */
    role_id: z.coerce.number().min(1, "角色ID必须大于0"),
    /** 是否获取全部数据（false=分页，true=全部） */
    all: BoolSchema.default(false),
    ...PageParam,
});
export type RoleUserDataParamType = z.infer<typeof RoleUserDataParamSchema>;

// 角色用户数据项（包含用户详细信息）
export const RoleUserDataItemSchema = z.object({
    /** 关联记录ID */
    id: z.coerce.number(),
    /** 角色ID */
    role_id: z.coerce.number(),
    /** 用户ID */
    user_id: z.coerce.number(),
    /** 状态 */
    status: z.coerce.number().optional(),
    /** 超时时间 */
    timeout: z.coerce.number().optional(),
    /** 变更时间 */
    change_time: UnixTimestampSchema.optional(),
    /** 变更用户ID */
    change_user_id: z.coerce.number().optional(),
    /** 用户详细信息 */
    user_data: UserDataResSchema,
});
export type RoleUserDataItemType = z.infer<typeof RoleUserDataItemSchema>;

export const RoleUserDataResSchema = z.object({
    data: z.array(RoleUserDataItemSchema),
    ...PageResSchema,
});
export type RoleUserDataResType = z.infer<typeof RoleUserDataResSchema>;

/**
 * 获取角色关联的用户数据
 * @description 获取指定角色下的所有用户列表
 */
export async function roleUserData(
    param: RoleUserDataParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<RoleUserDataResType>> {
    const { data } = await authApi().post("/api/system/rbac/role/user_data", param, config);
    return parseResData(data, RoleUserDataResSchema);
}

// 角色用户数据项
export const RoleUserAddDataItemSchema = z.object({
    /** 用户ID */
    user_id: z.coerce.number().min(1, "用户ID必须大于0"),
    /** 超时时间(秒) */
    timeout: z.coerce.number().min(0, "超时时间不能小于0"),
});
export type RoleUserAddDataItemType = z.infer<typeof RoleUserAddDataItemSchema>;

// 角色添加用户
export const RoleUserAddParamSchema = z.object({
    /** 角色ID */
    role_id: z.coerce.number().min(1, "角色ID必须大于0"),
    /** 用户数据列表 */
    user_data: z.array(RoleUserAddDataItemSchema).min(1, "至少选择一个用户"),
});
export type RoleUserAddParamType = z.infer<typeof RoleUserAddParamSchema>;

/**
 * 角色添加用户
 * @description 将指定用户添加到角色中，建立用户和角色的关联关系
 */
export async function roleUserAdd(
    param: RoleUserAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/rbac/role/user_add", param, config);
    return data;
}

// 角色删除用户
export const RoleUserDeleteParamSchema = z.object({
    /** 角色ID */
    role_id: z.coerce.number().min(1, "角色ID必须大于0"),
    /** 用户ID列表 */
    user_data: z.array(z.coerce.number().min(1, "用户ID必须大于0")).min(1, "至少选择一个用户"),
});
export type RoleUserDeleteParamType = z.infer<typeof RoleUserDeleteParamSchema>;

/**
 * 角色删除用户
 * @description 从角色中移除指定用户，解除用户和角色的关联关系
 */
export async function roleUserDelete(
    param: RoleUserDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/rbac/role/user_delete", param, config);
    return data;
}

// 角色权限数据
export const RolePermDataParamSchema = z.object({
    /** 角色ID */
    role_id: z.coerce.number().min(1, "角色ID必须大于0"),
    ...PageParam,
});
export type RolePermDataParamType = z.infer<typeof RolePermDataParamSchema>;

export const PermissionItemSchema = z.object({
    /** 资源ID */
    res_id: z.coerce.number(),
    /** 操作ID */
    op_id: z.coerce.number(),
    /** 用户ID */
    user_id: z.coerce.number().optional(),
    /** 资源类型 */
    res_type: z.string(),
    /** 资源数据 */
    res_data: z.string(),
    /** 资源状态 */
    res_status: z.coerce.number().optional(),
    /** 操作键 */
    op_key: z.string(),
    /** 操作状态 */
    op_status: z.coerce.number().optional(),
    /** 资源名称 */
    res_name: z.string().optional(),
    /** 操作名称 */
    op_name: z.string().optional(),
    /** 变更时间 */
    change_time: UnixTimestampSchema.optional(),
    /** 变更用户ID */
    change_user_id: z.coerce.number().optional(),
});

export const RolePermDataResSchema = z.object({
    data: z.array(PermissionItemSchema),
    ...PageResSchema,
});
export type PermissionItemType = z.infer<typeof PermissionItemSchema>;
export type RolePermDataResType = z.infer<typeof RolePermDataResSchema>;

/**
 * 获取角色权限数据
 * @description 获取指定角色的所有权限列表
 */
export async function rolePermData(
    param: RolePermDataParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<RolePermDataResType>> {
    const { data } = await authApi().post("/api/system/rbac/role/perm_data", param, config);
    return parseResData(data, RolePermDataResSchema);
}

// 角色权限数据项
export const RolePermDataItemSchema = z.object({
    /** 操作ID */
    op_id: z.coerce.number().min(1, "操作ID必须大于0"),
    /** 资源ID */
    res_id: z.coerce.number().min(1, "资源ID必须大于0"),
});
export type RolePermDataItemType = z.infer<typeof RolePermDataItemSchema>;

// 角色添加权限
export const RolePermAddParamSchema = z.object({
    /** 角色ID */
    role_id: z.coerce.number().min(1, "角色ID必须大于0"),
    /** 权限数据列表 */
    perm_data: z.array(RolePermDataItemSchema).min(1, "至少添加一个权限"),
});
export type RolePermAddParamType = z.infer<typeof RolePermAddParamSchema>;

/**
 * 角色添加权限
 * @description 为角色添加指定的资源操作权限
 */
export async function rolePermAdd(
    param: RolePermAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/rbac/role/perm_add", param, config);
    return data;
}

// 角色删除权限
export const RolePermDeleteParamSchema = z.object({
    /** 角色ID */
    role_id: z.coerce.number().min(1, "角色ID必须大于0"),
    /** 权限数据列表 */
    perm_data: z.array(RolePermDataItemSchema).min(1, "至少删除一个权限"),
});
export type RolePermDeleteParamType = z.infer<typeof RolePermDeleteParamSchema>;

/**
 * 角色删除权限
 * @description 删除角色的指定资源操作权限
 */
export async function rolePermDelete(
    param: RolePermDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/rbac/role/perm_delete", param, config);
    return data;
}


