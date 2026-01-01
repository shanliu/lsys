import { authApi } from "@shared/lib/apis/api_auth";
import { cleanEmptyStringParams, parseResData } from "@shared/lib/apis/utils";
import { ApiResult } from "@shared/types/apis-rest";
import { BoolSchema, PageParam, PageRes, UnixTimestampSchema } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

/**
 * RBAC 资源管理 API
 * 对应文档: docs/api/system/rbac/res/
 */

// 添加资源
export const ResourceAddParamSchema = z.object({
    /** 资源名称 */
    res_name: z.string().min(1, "资源名称不能为空"),
    /** 资源类型 */
    res_type: z.string().min(1, "资源类型不能为空"),
    /** 资源数据 */
    res_data: z.string().min(1, "资源数据不能为空"),
});
export type ResourceAddParamType = z.infer<typeof ResourceAddParamSchema>;

export const ResourceAddResSchema = z.object({
    /** 新创建的资源ID */
    id: z.coerce.number(),
});
export type ResourceAddResType = z.infer<typeof ResourceAddResSchema>;

/**
 * 添加资源
 * @description 创建一个新的RBAC资源，用于权限控制
 */
export async function resourceAdd(
    param: ResourceAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<ResourceAddResType>> {
    const { data } = await authApi().post("/api/system/rbac/res/add", param, config);
    return parseResData(data, ResourceAddResSchema);
}

// 资源列表
export const ResourceListParamSchema = z.object({
    /** 用户ID */
    user_id: z.coerce.number().optional().nullable(),
    /** 资源类型 */
    res_type: z.string().optional(),
    /** 资源数据 */
    res_data: z.string().optional(),
    /** 资源名称 */
    res_name: z.string().optional(),
    perm_count: BoolSchema.optional(),
    op_count: BoolSchema.optional(),
    /** ID列表 */
    ids: z.array(z.coerce.number()).optional(),
    ...PageParam,
});
export type ResourceListParamType = z.infer<typeof ResourceListParamSchema>;

export const ResourceItemSchema = z.object({
    /** 应用ID */
    app_id: z.coerce.number().optional().nullable(),
    /** 修改时间 */
    change_time: UnixTimestampSchema,
    /** 修改用户ID */
    change_user_id: z.coerce.number(),
    /** 资源ID */
    id: z.coerce.number(),
    /** 资源数据 */
    res_data: z.string(),
    /** 资源名称 */
    res_name: z.string(),
    /** 资源类型 */
    res_type: z.string(),
    /** 状态 */
    status: z.coerce.number().optional().nullable(),
    /** 用户ID */
    user_id: z.coerce.number(),
});
export type ResourceItemType = z.infer<typeof ResourceItemSchema>;

export const ResourceListResSchema = z.object({
    data: z.array(ResourceItemSchema),
    ...PageRes,
});
export type ResourceListResType = z.infer<typeof ResourceListResSchema>;

/**
 * 获取资源列表
 * @description 分页获取系统中的RBAC资源列表，支持多种筛选条件
 */
export async function resourceList(
    param: ResourceListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<ResourceListResType>> {
    const cleanedParam = cleanEmptyStringParams(param, ['res_type', 'res_data', 'res_name']);
    const { data } = await authApi().post("/api/system/rbac/res/list", cleanedParam, config);
    return parseResData(data, ResourceListResSchema);
}

// 编辑资源
export const ResourceEditParamSchema = z.object({
    /** 资源ID */
    res_id: z.coerce.number().min(1, "资源ID必须大于0"),
    /** 资源名称 */
    res_name: z.string().min(1, "资源名称不能为空"),
    /** 资源类型 */
    res_type: z.string().min(1, "资源类型不能为空"),
    /** 资源数据 */
    res_data: z.string().min(1, "资源数据不能为空"),
});
export type ResourceEditParamType = z.infer<typeof ResourceEditParamSchema>;

/**
 * 编辑资源
 * @description 修改现有资源的基本信息
 */
export async function resourceEdit(
    param: ResourceEditParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/rbac/res/edit", param, config);
    return data;
}

// 删除资源
export const ResourceDeleteParamSchema = z.object({
    /** 资源ID */
    res_id: z.coerce.number().min(1, "资源ID必须大于0"),
});
export type ResourceDeleteParamType = z.infer<typeof ResourceDeleteParamSchema>;

/**
 * 删除资源
 * @description 删除指定的RBAC资源，同时会删除相关的权限关联
 */
export async function resourceDelete(
    param: ResourceDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/rbac/res/delete", param, config);
    return data;
}

// 资源类型数据
export const ResourceTypeDataParamSchema = z.object({
    /** 资源类型，可选参数用于筛选 */
    res_type: z.string().optional(),
    ...PageParam,
});
export type ResourceTypeDataParamType = z.infer<typeof ResourceTypeDataParamSchema>;

export const ResourceTypeItemSchema = z.object({
    /** 类型ID */
    id: z.coerce.number(),
    /** 资源类型 */
    res_type: z.string(),
    /** 类型描述 */
    description: z.string().optional(),
    /** 创建时间 */
    create_time: UnixTimestampSchema,
});
export type ResourceTypeItemType = z.infer<typeof ResourceTypeItemSchema>;

export const ResourceTypeDataResSchema = z.object({
    data: z.array(ResourceTypeItemSchema),
    ...PageRes,
});
export type ResourceTypeDataResType = z.infer<typeof ResourceTypeDataResSchema>;

/**
 * 获取资源类型数据
 * @description 获取系统中定义的资源类型列表
 */
export async function resourceTypeData(
    param: ResourceTypeDataParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<ResourceTypeDataResType>> {
    const cleanedParam = cleanEmptyStringParams(param, ['res_type']);
    const { data } = await authApi().post("/api/system/rbac/res/type_data", cleanedParam, config);
    return parseResData(data, ResourceTypeDataResSchema);
}

// 资源类型操作数据
export const ResourceTypeOpDataParamSchema = z.object({
    /** 资源类型 */
    res_type: z.string().min(1, "资源类型不能为空"),
    ...PageParam,
});
export type ResourceTypeOpDataParamType = z.infer<typeof ResourceTypeOpDataParamSchema>;

/** 资源操作关联中的操作数据 */
export const ResourceOpDataSchema = z.object({
    /** 应用ID */
    app_id: z.coerce.number().optional().nullable(),
    /** 修改时间 */
    change_time: UnixTimestampSchema.optional(),
    /** 修改用户ID */
    change_user_id: z.coerce.number().optional(),
    /** 操作ID */
    id: z.coerce.number(),
    /** 操作键 */
    op_key: z.string(),
    /** 操作名称 */
    op_name: z.string(),
    /** 状态 */
    status: z.coerce.number().optional(),
    /** 用户ID */
    user_id: z.coerce.number().optional(),
});
export type ResourceOpDataType = z.infer<typeof ResourceOpDataSchema>;

/** 资源操作关联中的关联数据 */
export const ResourceOpResSchema = z.object({
    /** 应用ID */
    app_id: z.coerce.number().optional().nullable(),
    /** 修改时间 */
    change_time: UnixTimestampSchema.optional(),
    /** 修改用户ID */
    change_user_id: z.coerce.number().optional(),
    /** 关联ID */
    id: z.coerce.number(),
    /** 操作ID */
    op_id: z.coerce.number(),
    /** 资源类型 */
    res_type: z.string(),
    /** 状态 */
    status: z.coerce.number().optional(),
    /** 用户ID */
    user_id: z.coerce.number().optional(),
});
export type ResourceOpResType = z.infer<typeof ResourceOpResSchema>;

/** 资源类型操作数据项 */
export const ResourceOpItemSchema = z.object({
    /** 操作数据 */
    op_data: ResourceOpDataSchema,
    /** 关联数据 */
    op_res: ResourceOpResSchema,
});
export type ResourceOpItemType = z.infer<typeof ResourceOpItemSchema>;

export const ResourceTypeOpDataResSchema = z.object({
    data: z.array(ResourceOpItemSchema),
    ...PageRes,
});
export type ResourceTypeOpDataResType = z.infer<typeof ResourceTypeOpDataResSchema>;

/**
 * 获取资源类型操作数据
 * @description 获取指定资源类型支持的操作列表
 */
export async function resourceTypeOpData(
    param: ResourceTypeOpDataParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<ResourceTypeOpDataResType>> {
    const { data } = await authApi().post("/api/system/rbac/res/type_op_data", param, config);
    return parseResData(data, ResourceTypeOpDataResSchema);
}

// 添加资源类型操作
export const ResourceTypeOpAddParamSchema = z.object({
    /** 资源类型 */
    res_type: z.string().min(1, "资源类型不能为空"),
    /** 操作权限ID列表 */
    op_ids: z.array(z.coerce.number()).min(1, "至少选择一个操作权限"),
});
export type ResourceTypeOpAddParamType = z.infer<typeof ResourceTypeOpAddParamSchema>;

/**
 * 添加资源类型操作
 * @description 为指定资源类型添加新的操作定义
 */
export async function resourceTypeOpAdd(
    param: ResourceTypeOpAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/rbac/res/type_op_add", param, config);
    return data;
}

// 删除资源类型操作
export const ResourceTypeOpDelParamSchema = z.object({
    /** 资源类型 */
    res_type: z.string().min(1, "资源类型不能为空"),
    /** 删除操作ID列表 */
    op_ids: z.array(z.coerce.number()).min(1, "至少选择一个操作"),
});
export type ResourceTypeOpDelParamType = z.infer<typeof ResourceTypeOpDelParamSchema>;

/**
 * 删除资源类型操作
 * @description 删除指定资源类型的操作定义
 */
export async function resourceTypeOpDel(
    param: ResourceTypeOpDelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/rbac/res/type_op_del", param, config);
    return data;
}

// 静态资源数据
export const StaticResourceDataParamSchema = z.object({
    /** 资源类型 */
    res_type: z.string().optional(),
    ...PageParam,
});
export type StaticResourceDataParamType = z.infer<typeof StaticResourceDataParamSchema>;

export const StaticResourceDataResSchema = z.object({
    data: z.array(ResourceItemSchema),
    ...PageRes,
});
export type StaticResourceDataResType = z.infer<typeof StaticResourceDataResSchema>;

/**
 * 获取静态资源数据
 * @description 获取系统定义的静态资源列表，这些资源通常是预定义的系统资源
 */
export async function staticResourceData(
    param: StaticResourceDataParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<StaticResourceDataResType>> {
    const { data } = await authApi().post("/api/system/rbac/res/static_res_data", param, config);
    return parseResData(data, StaticResourceDataResSchema);
}

// 动态资源类型
export const DynamicResourceTypeParamSchema = z.object({
    /** 用户ID，用于获取用户相关的动态资源类型 */
    user_id: z.coerce.number().optional().nullable(),
    ...PageParam,
});
export type DynamicResourceTypeParamType = z.infer<typeof DynamicResourceTypeParamSchema>;

export const DynamicResourceTypeResSchema = z.object({
    data: z.array(ResourceTypeItemSchema),
    ...PageRes,
});
export type DynamicResourceTypeResType = z.infer<typeof DynamicResourceTypeResSchema>;

/**
 * 获取动态资源类型
 * @description 获取系统中的动态资源类型列表，这些类型可能根据用户或业务逻辑动态生成
 */
export async function dynamicResourceType(
    param: DynamicResourceTypeParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<DynamicResourceTypeResType>> {
    const { data } = await authApi().post("/api/system/rbac/res/dynamic_res_type", param, config);
    return parseResData(data, DynamicResourceTypeResSchema);
}

// 全局用户动态资源数据
export const DynamicResourceDataGlobalUserParamSchema = z.object({
    /** 用户ID */
    user_id: z.coerce.number().min(1, "用户ID必须大于0"),
    /** 资源类型 */
    res_type: z.string().optional(),
    ...PageParam,
});
export type DynamicResourceDataGlobalUserParamType = z.infer<typeof DynamicResourceDataGlobalUserParamSchema>;

export const DynamicResourceDataGlobalUserResSchema = z.object({
    data: z.array(ResourceItemSchema),
    ...PageRes,
});
export type DynamicResourceDataGlobalUserResType = z.infer<typeof DynamicResourceDataGlobalUserResSchema>;

/**
 * 获取全局用户动态资源数据
 * @description 获取指定用户的全局动态资源列表，这些资源可能根据用户权限动态生成
 */
export async function dynamicResourceDataGlobalUser(
    param: DynamicResourceDataGlobalUserParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<DynamicResourceDataGlobalUserResType>> {
    const { data } = await authApi().post("/api/system/rbac/res/dynamic_res_data_global_user", param, config);
    return parseResData(data, DynamicResourceDataGlobalUserResSchema);
}


