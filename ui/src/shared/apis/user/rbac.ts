import { authApi } from "@shared/lib/apis/api_auth";
import { cleanEmptyStringParams, parseResData } from "@shared/lib/apis/utils";
import { DictListSchema } from "@shared/types/apis-dict";
import { ApiResult } from "@shared/types/apis-rest";
import { BoolParamSchema, BoolSchema, LimitParam, LimitRes, PageParam, PageRes, UnixTimestampSchema, UserDataRes } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import { z } from "zod";
import { RoleAddParamSchema, RoleDeleteParamSchema, RoleEditParamSchema, RoleListParamSchema, RolePermDataParamSchema, RoleUserAddParamSchema, RoleUserDataParamSchema, RoleUserDeleteParamSchema } from "../admin/rbac-role";

// ============= 通用类型定义 =============

// ============= RBAC Base 相关接口 =============

// 审计日志查询参数
export const AuditParamSchema = z.object({
    user_ip: z.string().optional(),
    device_id: z.string().optional(),
    request_id: z.string().optional(),
    res_data: z.string().optional(),
    ...LimitParam,
});
export type AuditParamType = z.infer<typeof AuditParamSchema>;

// 审计日志详情数据
export const AuditDetailItemSchema = z.object({
    add_time: UnixTimestampSchema,
    check_result: z.string(),
    id: z.coerce.number(),
    is_role_all: z.string(),
    is_role_excluce: z.string(),
    is_role_include: z.string(),
    is_root: z.string(),
    is_self: z.string(),
    op_id: z.coerce.number(),
    op_key: z.string(),
    rbac_audit_id: z.coerce.number(),
    res_data: z.string(),
    res_id: z.coerce.number(),
    res_type: z.string(),
    res_user_id: z.coerce.number(),
    role_data: z.string(),
});
export type AuditDetailItemType = z.infer<typeof AuditDetailItemSchema>;

// 审计日志响应数据
export const AuditDataSchema = z.object({
    audit: z.object({
        add_time: UnixTimestampSchema,
        check_result: z.string(),
        device_id: z.string(),
        device_name: z.string(),
        id: z.coerce.number(),
        request_id: z.string(),
        role_key_data: z.string(),
        token_data: z.string(),
        user_app_id: z.string(),
        user_id: z.coerce.number(),
        user_ip: z.string(),
    }),
    detail: z.array(AuditDetailItemSchema),
    user:UserDataRes.nullable().optional(),
});
export type AuditDataType = z.infer<typeof AuditDataSchema>;

export const AuditListResSchema = z.object({
    data: z.array(AuditDataSchema),
    ...LimitRes,
});
export type AuditListResType = z.infer<typeof AuditListResSchema>;

// RBAC映射关系响应数据
export const MappingResSchema = z.object({
    audit_result: DictListSchema,
    role_res_range: DictListSchema,
    role_user_range: DictListSchema,
});
export type MappingResType = z.infer<typeof MappingResSchema>;

// ============= RBAC Resource 相关接口 =============

// 动态资源类型查询参数
export const dynamicResTypeParamSchema = z.object({
    res_type: z.string().optional(),
    ...PageParam,
});
export type DynamicResTypeParamType = z.infer<typeof dynamicResTypeParamSchema>;

// 静态资源操作数据
export const StaticResOpDataItemSchema = z.object({
    op_id: z.coerce.number(),
    op_key: z.string(),
    op_name: z.string(),
});
export type StaticResOpDataItemType = z.infer<typeof StaticResOpDataItemSchema>;

// 静态资源模板数据
export const StaticResTplDataItemSchema = z.object({
    op_data: z.array(StaticResOpDataItemSchema),
    res_name: z.string(),
    res_type: z.string(),
});
export type StaticResTplDataItemType = z.infer<typeof StaticResTplDataItemSchema>;

// 静态资源数据响应
export const StaticResResSchema = z.object({
    tpl_data: z.array(StaticResTplDataItemSchema),
});
export type StaticResResType = z.infer<typeof StaticResResSchema>;

// ============= RBAC Role 相关接口 =============

// 角色添加参数
export const roleAddParamSchema = z.object({
    user_range: z.coerce.number(),
    res_range: z.coerce.number(),
    role_name: z.string().min(1, "角色名称必须"),
    role_key: z.string().optional(),
});
export type RoleAddParamType = z.infer<typeof RoleAddParamSchema>;

// 角色编辑参数
export const roleEditParamSchema = z.object({
    role_id: z.coerce.number(),
    role_name: z.string().min(1, "角色名称必须"),
    role_key: z.string(),
});
export type RoleEditParamType = z.infer<typeof RoleEditParamSchema>;

// 角色删除参数
export const roleDeleteParamSchema = z.object({
    role_id: z.coerce.number(),
});
export type RoleDeleteParamType = z.infer<typeof RoleDeleteParamSchema>;

// 角色列表查询参数
export const roleListParamSchema = z.object({
    role_name: z.string().optional(),
    role_key: z.string().optional(),
    user_data: z.string().optional(),
    user_count: BoolSchema.optional(),
    ids: z.array(z.coerce.number()).optional(),
    user_range: z.coerce.number().optional().nullable(),
    res_range: z.coerce.number().optional().nullable(),
    ...PageParam,
});
export type RoleListParamType = z.infer<typeof RoleListParamSchema>;

// 角色数据响应
export const RoleDataSchema = z.object({
    change_time: UnixTimestampSchema,
    change_user_id: z.coerce.number(),
    id: z.coerce.number(),
    res_range: z.string(),
    role_key: z.string(),
    role_name: z.string(),
    status: z.coerce.number(),
    user_count: z.coerce.number().optional().nullable(),
    user_data: z.string().optional(),
    user_id: z.coerce.number(),
    user_range: z.coerce.number(),
});
export type RoleDataType = z.infer<typeof RoleDataSchema>;

export const RoleListResSchema = z.object({
    data: z.array(RoleDataSchema),
    ...PageRes,
});
export type RoleListResType = z.infer<typeof RoleListResSchema>;

// 可用用户查询参数
export const availableUserParamSchema = z.object({
    user_data: z.string().optional(),
    ...LimitParam,
});
export type AvailableUserParamType = z.infer<typeof availableUserParamSchema>;

// 用户数据响应
export const UserDataSchema = z.object({
    app_id: z.coerce.number(),
    id: z.coerce.number(),
    user_account: z.string(),
    user_data: z.string(),
    user_nickname: z.string(),
});
export type UserDataType = z.infer<typeof UserDataSchema>;

export const AvailableUserResSchema = z.object({
    data: z.array(UserDataSchema),
    ...LimitRes,
});
export type AvailableUserResType = z.infer<typeof AvailableUserResSchema>;

// 角色权限数据项
export const RolePermDataItemSchema = z.object({
    op_id: z.coerce.number(),
    res_id: z.coerce.number(),
});
export type RolePermDataItemType = z.infer<typeof RolePermDataItemSchema>;

// 角色权限相关参数
export const rolePermParamSchema = z.object({
    role_id: z.coerce.number(),
    perm_data: z.array(RolePermDataItemSchema),
});
export type RolePermParamType = z.infer<typeof rolePermParamSchema>;

export const rolePermDataParamSchema = z.object({
    role_id: z.coerce.number(),
    ...PageParam,
});
export type RolePermDataParamType = z.infer<typeof RolePermDataParamSchema>;

// 角色用户数据项
export const RoleUserDataItemSchema = z.object({
    user_id: z.coerce.number(),
    timeout: z.coerce.number(),
});
export type RoleUserDataItemType = z.infer<typeof RoleUserDataItemSchema>;

// 角色用户相关参数
export const roleUserAddParamSchema = z.object({
    role_id: z.coerce.number(),
    user_data: z.array(RoleUserDataItemSchema),
});
export type RoleUserAddParamType = z.infer<typeof RoleUserAddParamSchema>;

export const roleUserDataParamSchema = z.object({
    role_id: z.coerce.number(),
    all: BoolSchema.optional(),
    ...PageParam,
});
export type RoleUserDataParamType = z.infer<typeof RoleUserDataParamSchema>;

export const roleUserDeleteParamSchema = z.object({
    role_id: z.coerce.number(),
    user_data: z.array(z.coerce.number()),
});
export type RoleUserDeleteParamType = z.infer<typeof RoleUserDeleteParamSchema>;

// 角色用户数据响应
export const RoleUserDataSchema = z.object({
    change_time: UnixTimestampSchema,
    change_user_id: z.coerce.number(),
    id: z.coerce.number(),
    role_id: z.coerce.number(),
    status: z.coerce.number(),
    timeout: z.coerce.number(),
    user_data: UserDataSchema,
    user_id: z.coerce.number(),
});
export type RoleUserDataType = z.infer<typeof RoleUserDataSchema>;

export const RoleUserDataResSchema = z.object({
    data: z.array(RoleUserDataSchema),
    ...PageRes,
});
export type RoleUserDataResType = z.infer<typeof RoleUserDataResSchema>;

// ============= API 函数实现 =============

// RBAC Base APIs
export const rbacAudit = async (
    param: AuditParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AuditListResType>> => {
    const cleanedParam = cleanEmptyStringParams(param, ['user_ip', 'device_id', 'request_id', 'res_data']);
    const res = await authApi().post('/api/user/rbac/base/audit', cleanedParam, config);
    return res.data;
};

export const rbacMapping = async (
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<MappingResType>> => {
    const res = await authApi().post('/api/user/rbac/base/mapping', {}, config);
    return res.data;
};

// RBAC Resource APIs
export const rbacDynamicResType = async (
    param: DynamicResTypeParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> => {
    const res = await authApi().post('/api/user/rbac/res/dynamic_res_type', param, config);
    return res.data;
};

export const rbacDynamicResDataTest = async (
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> => {
    const res = await authApi().post('/api/user/rbac/res/dynamic_res_data_test', {}, config);
    return res.data;
};

export const rbacStaticResData = async (
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<StaticResResType>> => {
    const res = await authApi().post('/api/user/rbac/res/static_res_data', {}, config);
    return res.data;
};

// RBAC Role APIs
export const rbacRoleAdd = async (
    param: RoleAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> => {
    const res = await authApi().post('/api/user/rbac/role/add', param, config);
    return res.data;
};

export const rbacRoleEdit = async (
    param: RoleEditParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> => {
    const res = await authApi().post('/api/user/rbac/role/edit', param, config);
    return res.data;
};

export const rbacRoleDelete = async (
    param: RoleDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> => {
    const res = await authApi().post('/api/user/rbac/role/delete', param, config);
    return res.data;
};

export const rbacRoleList = async (
    param: RoleListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<RoleListResType>> => {
    const cleanedParam = cleanEmptyStringParams(param, ['role_name', 'role_key', 'user_data']);
    const res = await authApi().post('/api/user/rbac/role/list', cleanedParam, config);
    return res.data;
};

export const rbacRoleAvailableUser = async (
    param: AvailableUserParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AvailableUserResType>> => {
    const res = await authApi().post('/api/user/rbac/role/available_user', param, config);
    return res.data;
};

export const rbacRolePermAdd = async (
    param: RolePermParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> => {
    const res = await authApi().post('/api/user/rbac/role/perm_add', param, config);
    return res.data;
};

export const rbacRolePermData = async (
    param: RolePermDataParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> => {
    const res = await authApi().post('/api/user/rbac/role/perm_data', param, config);
    return res.data;
};

export const rbacRolePermDelete = async (
    param: RolePermParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> => {
    const res = await authApi().post('/api/user/rbac/role/perm_delete', param, config);
    return res.data;
};

export const rbacRoleUserAdd = async (
    param: RoleUserAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> => {
    const res = await authApi().post('/api/user/rbac/role/user_add', param, config);
    return res.data;
};

export const rbacRoleUserData = async (
    param: RoleUserDataParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<RoleUserDataResType>> => {
    const res = await authApi().post('/api/user/rbac/role/user_data', param, config);
    return res.data;
};

export const rbacRoleUserDelete = async (
    param: RoleUserDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> => {
    const res = await authApi().post('/api/user/rbac/role/user_delete', param, config);
    return res.data;
};

// ============= APP RBAC 新增接口 =============

// 97. 审计数据查询
export const AppRbacAuditDataParamSchema = z.object({
    user_data: z.any().optional(),
    app_id: z.coerce.number(),
    user_ip: z.string().optional(),
    device_id: z.string().optional(),
    request_id: z.string().optional(),
    res_data: z.object({
        res_id: z.coerce.number(),
    }),
    ...LimitParam,
});
export type AppRbacAuditDataParamType = z.infer<typeof AppRbacAuditDataParamSchema>;

// APP审计详情项
export const AppRbacAuditDetailItemSchema = z.object({
    add_time: UnixTimestampSchema,
    check_result: z.string(),
    id: z.coerce.number(),
    is_role_all: z.string(),
    is_role_excluce: z.string(),
    is_role_include: z.string(),
    is_root: z.string(),
    is_self: z.string(),
    op_id: z.coerce.number(),
    op_key: z.string(),
    rbac_audit_id: z.coerce.number(),
    res_data: z.string(),
    res_id: z.coerce.number(),
    res_type: z.string(),
    res_user_id: z.coerce.number(),
    role_data: z.string(),
});
export type AppRbacAuditDetailItemType = z.infer<typeof AppRbacAuditDetailItemSchema>;

// APP审计数据项
export const AppRbacAuditDataItemSchema = z.object({
    audit: z.object({
        add_time: UnixTimestampSchema,
        check_result: z.string(),
        device_id: z.string(),
        device_name: z.string(),
        id: z.coerce.number(),
        request_id: z.string(),
        role_key_data: z.string(),
        token_data: z.string(),
        user_app_id: z.string(),
        user_id: z.coerce.number(),
        user_ip: z.string(),
    }),
    detail: z.array(AppRbacAuditDetailItemSchema),
    user:UserDataRes.nullable().optional(),
});
export type AppRbacAuditDataItemType = z.infer<typeof AppRbacAuditDataItemSchema>;

export const AppRbacAuditDataResSchema = z.object({
    data: z.array(AppRbacAuditDataItemSchema),
    ...LimitRes,
});
export type AppRbacAuditDataResType = z.infer<typeof AppRbacAuditDataResSchema>;

export const appRbacBaseAuditData = async (
    param: AppRbacAuditDataParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacAuditDataResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/base/audit_data', param, config);
    return data;
};

// 98. 获取RBAC映射信息
export const AppRbacBaseMappingParamSchema = z.object({});
export type AppRbacBaseMappingParamType = z.infer<typeof AppRbacBaseMappingParamSchema>;

export const AppRbacBaseMappingResSchema = z.object({
    audit_result: DictListSchema,
    role_res_range: DictListSchema,
    role_user_range: DictListSchema,
});
export type AppRbacBaseMappingResType = z.infer<typeof AppRbacBaseMappingResSchema>;

export const appRbacBaseMapping = async (
    param: AppRbacBaseMappingParamType = {},
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacBaseMappingResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/base/mapping', param, config);
    return data;
};

// ============= APP RBAC OP 操作权限相关接口 =============

// 99. 添加应用操作权限
export const AppRbacOpAddParamSchema = z.object({
    app_id: z.coerce.number(),
    use_app_user: BoolSchema,
    user_param: z.string(),
    op_key: z.string(),
    op_name: z.string(),
});
export type AppRbacOpAddParamType = z.infer<typeof AppRbacOpAddParamSchema>;

export const AppRbacOpAddResSchema = z.object({
    id: z.coerce.number(),
});
export type AppRbacOpAddResType = z.infer<typeof AppRbacOpAddResSchema>;

export const appRbacOpAdd = async (
    param: AppRbacOpAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacOpAddResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/op/add', param, config);
    return parseResData(data, AppRbacOpAddResSchema);
};

// 100. 删除应用操作权限
export const AppRbacOpDeleteParamSchema = z.object({
    op_id: z.coerce.number(),
});
export type AppRbacOpDeleteParamType = z.infer<typeof AppRbacOpDeleteParamSchema>;

export const AppRbacOpDeleteResSchema = z.object({});
export type AppRbacOpDeleteResType = z.infer<typeof AppRbacOpDeleteResSchema>;

export const appRbacOpDelete = async (
    param: AppRbacOpDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacOpDeleteResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/op/delete', param, config);
    return parseResData(data, AppRbacOpDeleteResSchema);
};

// 101. 编辑操作数据
export const AppRbacOpEditParamSchema = z.object({
    op_id: z.coerce.number(),
    op_key: z.string(),
    op_name: z.string(),
});
export type AppRbacOpEditParamType = z.infer<typeof AppRbacOpEditParamSchema>;

export const AppRbacOpEditResSchema = z.object({});
export type AppRbacOpEditResType = z.infer<typeof AppRbacOpEditResSchema>;

export const appRbacOpEdit = async (
    param: AppRbacOpEditParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacOpEditResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/op/edit', param, config);
    return parseResData(data, AppRbacOpEditResSchema);
};

// 102. 应用操作列表
export const AppRbacOpListParamSchema = z.object({
    app_id: z.coerce.number(),
    use_app_user: BoolSchema,
    user_param: z.string(),
    op_name: z.string().optional(),
    op_key: z.string().optional(),
    ids: z.array(z.coerce.number()).optional(),
     res_type_count:BoolParamSchema.optional(),
    check_role_use:BoolParamSchema.optional(),
    ...PageParam,
});
export type AppRbacOpListParamType = z.infer<typeof AppRbacOpListParamSchema>;

// APP操作数据项
export const AppRbacOpDataItemSchema = z.object({
    app_id: z.string(),
    change_time: UnixTimestampSchema,
    change_user_id: z.coerce.number(),
    id: z.coerce.number(),
    op_key: z.string(),
    op_name: z.string(),
    is_role_use: BoolParamSchema.optional(),
    res_type_count: z.coerce.number().optional(),
});
export type AppRbacOpDataItemType = z.infer<typeof AppRbacOpDataItemSchema>;

export const AppRbacOpListResSchema = z.object({
    count: z.string().nullish(),
    data: z.array(AppRbacOpDataItemSchema),
});
export type AppRbacOpListResType = z.infer<typeof AppRbacOpListResSchema>;

export const appRbacOpList = async (
    param: AppRbacOpListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacOpListResType>> => {
    const cleanedParam = cleanEmptyStringParams(param, ['op_name', 'op_key', 'user_param']);
    const { data } = await authApi().post('/api/user/app_rbac/op/list', cleanedParam, config);
    return parseResData(data, AppRbacOpListResSchema);
};

// ============= APP RBAC ROLE 角色相关接口 =============

// 100. APP RBAC角色添加
export const AppRbacRoleAddParamSchema = z.object({
    app_id: z.coerce.number(),
    use_app_user: BoolSchema,
    user_param: z.string(),
    user_range: z.coerce.number(),
    res_range: z.coerce.number(),
    role_name: z.string(),
    role_key: z.string(),
});
export type AppRbacRoleAddParamType = z.infer<typeof AppRbacRoleAddParamSchema>;

export const AppRbacRoleAddResSchema = z.object({
    role_id: z.coerce.number(),
});
export type AppRbacRoleAddResType = z.infer<typeof AppRbacRoleAddResSchema>;

export const appRbacRoleAdd = async (
    param: AppRbacRoleAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacRoleAddResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/role/add', param, config);
    return data;
};

// 100. APP RBAC角色编辑
export const AppRbacRoleEditParamSchema = z.object({
    role_id: z.coerce.number(),
    role_name: z.string(),
    role_key: z.string(),
});
export type AppRbacRoleEditParamType = z.infer<typeof AppRbacRoleEditParamSchema>;

export const AppRbacRoleEditResSchema = z.object({});
export type AppRbacRoleEditResType = z.infer<typeof AppRbacRoleEditResSchema>;

export const appRbacRoleEdit = async (
    param: AppRbacRoleEditParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacRoleEditResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/role/edit', param, config);
    return data;
};

// 101. APP RBAC角色删除
export const AppRbacRoleDeleteParamSchema = z.object({
    role_id: z.coerce.number(),
});
export type AppRbacRoleDeleteParamType = z.infer<typeof AppRbacRoleDeleteParamSchema>;

export const AppRbacRoleDeleteResSchema = z.object({});
export type AppRbacRoleDeleteResType = z.infer<typeof AppRbacRoleDeleteResSchema>;

export const appRbacRoleDelete = async (
    param: AppRbacRoleDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacRoleDeleteResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/role/delete', param, config);
    return data;
};

// 102. APP RBAC角色列表
export const AppRbacRoleListParamSchema = z.object({
    app_id: z.coerce.number().optional().nullable(),
    use_app_user: BoolSchema.optional(),
    user_param: z.string().optional(),
    role_name: z.string().optional(),
    role_key: z.string().optional(),
    user_data: z.string().optional().nullable(),
    user_count: BoolSchema.optional(),
    res_count: BoolSchema.optional(),
    res_op_count: BoolSchema.optional(),
    ids: z.array(z.coerce.number()).optional(),
    user_range: z.coerce.number().optional().nullable(),
    res_range: z.coerce.number().optional().nullable(),
    ...PageParam,
});
export type AppRbacRoleListParamType = z.infer<typeof AppRbacRoleListParamSchema>;

// APP角色数据项
export const AppRbacRoleDataItemSchema = z.object({
    id: z.coerce.number(),
    role_name: z.string(),
    role_key: z.string(),
    app_id: z.coerce.number(),
    use_app_user: z.boolean(),
    user_param: z.string(),
    user_range: z.coerce.number(),
    res_range: z.coerce.number(),
    user_count: z.coerce.number().optional().nullable(),
    res_count: z.coerce.number().optional().nullable(),
    res_op_count: z.coerce.number().optional().nullable(),
});
export type AppRbacRoleDataItemType = z.infer<typeof AppRbacRoleDataItemSchema>;

export const AppRbacRoleListResSchema = z.object({
    data: z.array(AppRbacRoleDataItemSchema),
    ...PageRes,
});
export type AppRbacRoleListResType = z.infer<typeof AppRbacRoleListResSchema>;

export const appRbacRoleList = async (
    param: AppRbacRoleListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacRoleListResType>> => {
    const cleanedParam = cleanEmptyStringParams(param, ['role_name', 'role_key', 'user_data', 'user_param']);
    const { data } = await authApi().post('/api/user/app_rbac/role/list', cleanedParam, config);
    return data;
};

// 103. APP RBAC资源添加
export const AppRbacResAddParamSchema = z.object({
    app_id: z.coerce.number(),
    use_app_user: BoolSchema,
    user_param: z.string(),
    res_name: z.string(),
    res_type: z.string(),
    res_data: z.string(),
});
export type AppRbacResAddParamType = z.infer<typeof AppRbacResAddParamSchema>;

export const AppRbacResAddResSchema = z.object({
    res_id: z.coerce.number(),
});
export type AppRbacResAddResType = z.infer<typeof AppRbacResAddResSchema>;

export const appRbacResAdd = async (
    param: AppRbacResAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacResAddResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/res/add', param, config);
    return data;
};

// 104. APP RBAC资源删除
export const AppRbacResDeleteParamSchema = z.object({
    res_id: z.coerce.number(),
});
export type AppRbacResDeleteParamType = z.infer<typeof AppRbacResDeleteParamSchema>;

export const AppRbacResDeleteResSchema = z.object({});
export type AppRbacResDeleteResType = z.infer<typeof AppRbacResDeleteResSchema>;

export const appRbacResDelete = async (
    param: AppRbacResDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacResDeleteResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/res/delete', param, config);
    return data;
};

// 105. APP RBAC资源编辑
export const AppRbacResEditParamSchema = z.object({
    res_id: z.coerce.number(),
    res_name: z.string(),
    res_type: z.string(),
    res_data: z.string(),
});
export type AppRbacResEditParamType = z.infer<typeof AppRbacResEditParamSchema>;

export const AppRbacResEditResSchema = z.object({});
export type AppRbacResEditResType = z.infer<typeof AppRbacResEditResSchema>;

export const appRbacResEdit = async (
    param: AppRbacResEditParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacResEditResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/res/edit', param, config);
    return data;
};

// 106. APP RBAC资源列表
export const AppRbacResListParamSchema = z.object({
    app_id: z.coerce.number().optional().nullable(),
    use_app_user: BoolSchema.optional(),
    user_param: z.string().optional(),
    res_type: z.string().optional(),
    res_data: z.string().optional(),
    res_name: z.string().optional(),
    perm_count: BoolSchema.optional(),
    op_count: BoolSchema.optional(),
    ids: z.array(z.coerce.number()).optional(),
    ...PageParam,
});
export type AppRbacResListParamType = z.infer<typeof AppRbacResListParamSchema>;

// APP资源数据项
export const AppRbacResDataItemSchema = z.object({
    id: z.coerce.number(),
    res_name: z.string(),
    res_type: z.string(),
    res_data: z.string(),
    app_id: z.coerce.number().optional(),
    use_app_user: z.boolean().optional(),
    user_param: z.string().optional(),
    user_id: z.string().optional(),
    user_data: UserDataSchema.optional(),
    change_time: UnixTimestampSchema.optional(),
    op_count: z.coerce.number().nullish(),
    perm_count: z.coerce.number().nullish(),
});
export type AppRbacResDataItemType = z.infer<typeof AppRbacResDataItemSchema>;

export const AppRbacResListResSchema = z.object({
    data: z.array(AppRbacResDataItemSchema),
    count: z.string().nullish(),
    ...PageRes,
});
export type AppRbacResListResType = z.infer<typeof AppRbacResListResSchema>;

export const appRbacResList = async (
    param: AppRbacResListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacResListResType>> => {
    const cleanedParam = cleanEmptyStringParams(param, ['res_type', 'res_data', 'res_name', 'user_param']);
    const { data } = await authApi().post('/api/user/app_rbac/res/list', cleanedParam, config);
    return data;
};

// 107. APP RBAC资源类型数据
export const AppRbacResTypeDataParamSchema = z.object({
    app_id: z.coerce.number(),
    use_app_user: BoolSchema,
    user_param: z.string(),
    res_type: z.string(),
    ...PageParam,
});
export type AppRbacResTypeDataParamType = z.infer<typeof AppRbacResTypeDataParamSchema>;

// APP资源类型数据项
export const AppRbacResTypeDataItemSchema = z.object({
    res_data: z.string(),
    count: z.coerce.number().optional().nullable(),
});
export type AppRbacResTypeDataItemType = z.infer<typeof AppRbacResTypeDataItemSchema>;

export const AppRbacResTypeDataResSchema = z.object({
    data: z.array(AppRbacResTypeDataItemSchema),
    ...PageRes,
});
export type AppRbacResTypeDataResType = z.infer<typeof AppRbacResTypeDataResSchema>;

export const appRbacResTypeData = async (
    param: AppRbacResTypeDataParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacResTypeDataResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/res/type_data', param, config);
    return data;
};

// 108. APP RBAC资源类型操作添加
export const AppRbacResTypeOpAddParamSchema = z.object({
    app_id: z.coerce.number(),
    use_app_user: BoolSchema,
    user_param: z.string(),
    res_type: z.string(),
    op_ids: z.array(z.coerce.number()),
});
export type AppRbacResTypeOpAddParamType = z.infer<typeof AppRbacResTypeOpAddParamSchema>;

export const AppRbacResTypeOpAddResSchema = z.object({});
export type AppRbacResTypeOpAddResType = z.infer<typeof AppRbacResTypeOpAddResSchema>;

export const appRbacResTypeOpAdd = async (
    param: AppRbacResTypeOpAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacResTypeOpAddResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/res/type_op_add', param, config);
    return data;
};

// 109. APP RBAC资源类型操作数据
export const AppRbacResTypeOpDataParamSchema = z.object({
    app_id: z.coerce.number(),
    use_app_user: BoolSchema,
    user_param: z.string(),
    res_type: z.string(),
    ...PageParam,
});
export type AppRbacResTypeOpDataParamType = z.infer<typeof AppRbacResTypeOpDataParamSchema>;

// APP资源类型操作数据项中的操作数据
export const AppRbacResTypeOpDataOpSchema = z.object({
    app_id: z.string(),
    change_time: z.string(),
    change_user_id: z.string(),
    id: z.coerce.number(),
    op_key: z.string(),
    op_name: z.string(),
    status: z.string(),
    user_id: z.string(),
});

// APP资源类型操作数据项中的关联关系数据
export const AppRbacResTypeOpDataResSchema = z.object({
    app_id: z.string(),
    change_time: z.string(),
    change_user_id: z.string(),
    id: z.coerce.number(),
    op_id: z.string(),
    res_type: z.string(),
    status: z.string(),
    user_id: z.string(),
});

// APP资源类型操作数据项
export const AppRbacResTypeOpDataItemSchema = z.object({
    op_data: AppRbacResTypeOpDataOpSchema,
    op_res: AppRbacResTypeOpDataResSchema,
});
export type AppRbacResTypeOpDataItemType = z.infer<typeof AppRbacResTypeOpDataItemSchema>;

export const AppRbacResTypeOpDataListResSchema = z.object({
    data: z.array(AppRbacResTypeOpDataItemSchema),
    ...PageRes,
});
export type AppRbacResTypeOpDataListResType = z.infer<typeof AppRbacResTypeOpDataListResSchema>;

export const appRbacResTypeOpData = async (
    param: AppRbacResTypeOpDataParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacResTypeOpDataListResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/res/type_op_data', param, config);
    return data;
};

// 110. APP RBAC资源类型操作删除
export const AppRbacResTypeOpDelParamSchema = z.object({
    app_id: z.coerce.number(),
    use_app_user: BoolSchema,
    user_param: z.string(),
    res_type: z.string(),
    op_ids: z.array(z.coerce.number()),
});
export type AppRbacResTypeOpDelParamType = z.infer<typeof AppRbacResTypeOpDelParamSchema>;

export const AppRbacResTypeOpDelResSchema = z.object({});
export type AppRbacResTypeOpDelResType = z.infer<typeof AppRbacResTypeOpDelResSchema>;

export const appRbacResTypeOpDel = async (
    param: AppRbacResTypeOpDelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacResTypeOpDelResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/res/type_op_del', param, config);
    return data;
};

// 112. APP RBAC角色可用用户
export const AppRbacRoleAvailableUserParamSchema = z.object({
    app_id: z.coerce.number(),
    user_any: z.any().nullable().optional(),
    ...LimitParam,
});
export type AppRbacRoleAvailableUserParamType = z.infer<typeof AppRbacRoleAvailableUserParamSchema>;

// APP角色可用用户数据项
export const AppRbacRoleAvailableUserDataItemSchema = z.object({
    id: z.coerce.number(),
    nickname: z.string(),
    username: z.string(),
});
export type AppRbacRoleAvailableUserDataItemType = z.infer<typeof AppRbacRoleAvailableUserDataItemSchema>;

export const AppRbacRoleAvailableUserResSchema = z.object({
    data: z.array(AppRbacRoleAvailableUserDataItemSchema),
    ...LimitRes,
});
export type AppRbacRoleAvailableUserResType = z.infer<typeof AppRbacRoleAvailableUserResSchema>;

export const appRbacRoleAvailableUser = async (
    param: AppRbacRoleAvailableUserParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacRoleAvailableUserResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/role/available_user', param, config);
    return data;
};

// APP角色权限数据项
export const AppRbacRolePermDataItemSchema = z.object({
    op_id: z.coerce.number(),
    res_id: z.coerce.number(),
});
export type AppRbacRolePermDataItemType = z.infer<typeof AppRbacRolePermDataItemSchema>;

// 116. APP RBAC角色权限添加
export const AppRbacRolePermAddParamSchema = z.object({
    role_id: z.coerce.number(),
    perm_data: z.array(AppRbacRolePermDataItemSchema),
});
export type AppRbacRolePermAddParamType = z.infer<typeof AppRbacRolePermAddParamSchema>;

export const AppRbacRolePermAddResSchema = z.object({});
export type AppRbacRolePermAddResType = z.infer<typeof AppRbacRolePermAddResSchema>;

export const appRbacRolePermAdd = async (
    param: AppRbacRolePermAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacRolePermAddResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/role/perm_add', param, config);
    return data;
};

// 117. APP RBAC角色权限数据
export const AppRbacRolePermDataParamSchema = z.object({
    role_id: z.coerce.number(),
    ...PageParam,
});
export type AppRbacRolePermDataParamType = z.infer<typeof AppRbacRolePermDataParamSchema>;

// APP角色权限数据响应项
export const AppRbacRolePermDataResItemSchema = z.object({
    id: z.coerce.number(),
    role_id: z.coerce.number(),
    op_id: z.coerce.number(),
    res_id: z.coerce.number(),
    op_key: z.string().optional(),
    op_name: z.string().optional(),
    res_name: z.string().optional(),
    res_type: z.string().optional(),
    res_data: z.string().optional(),
});
export type AppRbacRolePermDataResItemType = z.infer<typeof AppRbacRolePermDataResItemSchema>;

export const AppRbacRolePermDataResSchema = z.object({
    data: z.array(AppRbacRolePermDataResItemSchema),
    ...PageRes,
});
export type AppRbacRolePermDataResType = z.infer<typeof AppRbacRolePermDataResSchema>;

export const appRbacRolePermData = async (
    param: AppRbacRolePermDataParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacRolePermDataResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/role/perm_data', param, config);
    return data;
};

// 118. APP RBAC角色权限删除
export const AppRbacRolePermDeleteParamSchema = z.object({
    role_id: z.coerce.number(),
    perm_data: z.array(AppRbacRolePermDataItemSchema),
});
export type AppRbacRolePermDeleteParamType = z.infer<typeof AppRbacRolePermDeleteParamSchema>;

export const AppRbacRolePermDeleteResSchema = z.object({});
export type AppRbacRolePermDeleteResType = z.infer<typeof AppRbacRolePermDeleteResSchema>;

export const appRbacRolePermDelete = async (
    param: AppRbacRolePermDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacRolePermDeleteResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/role/perm_delete', param, config);
    return data;
};

// APP角色用户数据项
export const AppRbacRoleUserDataItemSchema = z.object({
    use_app_user: BoolSchema,
    user_param: z.string(),
    timeout: z.coerce.number(),
});
export type AppRbacRoleUserDataItemType = z.infer<typeof AppRbacRoleUserDataItemSchema>;

// 119. APP RBAC角色用户添加
export const AppRbacRoleUserAddParamSchema = z.object({
    role_id: z.coerce.number(),
    user_data: z.array(AppRbacRoleUserDataItemSchema),
});
export type AppRbacRoleUserAddParamType = z.infer<typeof AppRbacRoleUserAddParamSchema>;

export const AppRbacRoleUserAddResSchema = z.object({});
export type AppRbacRoleUserAddResType = z.infer<typeof AppRbacRoleUserAddResSchema>;

export const appRbacRoleUserAdd = async (
    param: AppRbacRoleUserAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacRoleUserAddResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/role/user_add', param, config);
    return data;
};

// 120. APP RBAC角色用户数据
export const AppRbacRoleUserDataParamSchema = z.object({
    role_id: z.coerce.number(),
    all: BoolSchema.optional(),
    ...PageParam,
});
export type AppRbacRoleUserDataParamType = z.infer<typeof AppRbacRoleUserDataParamSchema>;

// APP角色用户数据响应项
export const AppRbacRoleUserDataResItemSchema = z.object({
    id: z.coerce.number(),
    role_id: z.coerce.number(),
    use_app_user: z.boolean(),
    user_param: z.string(),
    timeout: z.coerce.number(),
    user_id: z.coerce.number().optional().nullable(),
    nickname: z.string().optional(),
    username: z.string().optional(),
});
export type AppRbacRoleUserDataResItemType = z.infer<typeof AppRbacRoleUserDataResItemSchema>;

export const AppRbacRoleUserDataResSchema = z.object({
    data: z.array(AppRbacRoleUserDataResItemSchema),
    ...PageRes,
});
export type AppRbacRoleUserDataResType = z.infer<typeof AppRbacRoleUserDataResSchema>;

export const appRbacRoleUserData = async (
    param: AppRbacRoleUserDataParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacRoleUserDataResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/role/user_data', param, config);
    return data;
};

// 121. APP RBAC角色用户删除
export const AppRbacRoleUserDeleteParamSchema = z.object({
    role_id: z.coerce.number(),
    use_app_user: BoolSchema,
    user_data: z.array(z.string()),
});
export type AppRbacRoleUserDeleteParamType = z.infer<typeof AppRbacRoleUserDeleteParamSchema>;

export const AppRbacRoleUserDeleteResSchema = z.object({});
export type AppRbacRoleUserDeleteResType = z.infer<typeof AppRbacRoleUserDeleteResSchema>;

export const appRbacRoleUserDelete = async (
    param: AppRbacRoleUserDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRbacRoleUserDeleteResType>> => {
    const { data } = await authApi().post('/api/user/app_rbac/role/user_delete', param, config);
    return data;
};


