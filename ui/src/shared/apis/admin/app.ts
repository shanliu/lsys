
// ...existing code...
import { authApi } from "@shared/lib/apis/api_auth";
import { cleanEmptyStringParams, parseResData } from "@shared/lib/apis/utils";
import { DictListSchema } from "@shared/types/apis-dict";
import { ApiResult } from "@shared/types/apis-rest";
import { BoolSchema, PageParam, PageRes, UnixTimestampSchema, UserDataRes } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

/**
 * 应用管理 API
 * 对应文档: docs/api/system/app/
 * 包括应用列表、应用确认、OAuth配置、子应用管理等功能
 */

// 应用映射数据
export const AppMapResSchema = z.object({
    /** 应用状态字典 */
    app_status: DictListSchema,
    /** 外部功能字典 */
    exter_features: DictListSchema,
    /** 申请状态字典 */
    request_status: DictListSchema,
    /** 申请类型字典 */
    request_type: DictListSchema,
    /** 密钥状态字典 */
    secret_status: DictListSchema,
});
export type AppMapResType = z.infer<typeof AppMapResSchema>;

/**
 * 获取应用字典映射数据
 * @description 获取应用管理相关的字典数据，包括状态、类型等枚举值
 */
export async function appMapping(
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppMapResType>> {
    const { data } = await authApi().post("/api/system/app/mapping", {}, config);
    return parseResData(data, AppMapResSchema);
}

/**
 * 应用列表管理
 */

// OAuth客户端数据结构
export const OAuthClientDataSchema = z.object({
    /** 回调域名 */
    callback_domain: z.string(),
    /** 授权范围数据 */
    scope_data: z.string(),
});
export type OAuthClientDataType = z.infer<typeof OAuthClientDataSchema>;

// OAuth服务器范围数据
export const OAuthServerScopeDataSchema = z.object({
    /** 授权范围键名 */
    scope_key: z.string(),
    /** 授权范围名称 */
    scope_name: z.string(),
});
export type OAuthServerScopeDataType = z.infer<typeof OAuthServerScopeDataSchema>;

// 子应用统计数据
export const SubAppCountSchema = z.object({
    /** 禁用子应用数量 */
    disable: z.coerce.number(),
    /** 启用子应用数量 */
    enable: z.coerce.number(),
    /** 初始化子应用数量 */
    init: z.coerce.number(),
});
export type SubAppCountType = z.infer<typeof SubAppCountSchema>;

// 某应用的子应用列表参数
export const SubAppListParamSchema = z.object({
    /** 应用ID */
    app_id: z.coerce.number(),
    /** 状态 */
    status: z.coerce.number().optional().nullable(),
    ...PageParam,
});
export type SubAppListParamType = z.infer<typeof SubAppListParamSchema>;


// 某应用的子应用项数据结构
export const SubAppItemSchema = z.object({
    /** 应用ID */
    id: z.coerce.number(),
    /** 应用名称 */
    name: z.string(),
    /** 客户端ID */
    client_id: z.string().nullable().optional(),
    /** 状态 */
    status: z.coerce.number(),
    /** 所属用户ID */
    user_id: z.coerce.number(),
    /** 修改时间(秒) */
    change_time: UnixTimestampSchema,
    /** 修改用户ID */
    change_user_id: z.coerce.number(),
    /** 外部功能列表 */
    exter_feature: z.array(z.any()),
    /** 是否支持OAuth客户端 */
    oauth_client: BoolSchema,
    /** OAuth客户端数据 */
    oauth_client_data: OAuthClientDataSchema.nullable().optional(),
    /** 是否开通子应用功能 */
    sup_app: BoolSchema,
    /** 用户数据 */
    user_data: UserDataRes.nullable().optional(),
});
export type SubAppItemType = z.infer<typeof SubAppItemSchema>;

export const SubAppListResSchema = z.object({
    data: z.array(SubAppItemSchema),
    ...PageRes,
});
export type SubAppListResType = z.infer<typeof SubAppListResSchema>;




/**
 * 获取某应用的子应用列表
 * @description 分页获取某应用的子应用列表
 */
export async function subAppList(
    param: SubAppListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SubAppListResType>> {
    const { data } = await authApi().post("/api/system/app/sub_list", param, config);
    return parseResData(data, SubAppListResSchema as any);
}


// 应用列表查询参数
export const AppListParamSchema = z.object({
    /** 用户ID */
    user_id: z.coerce.number().optional().nullable(),
    /** 应用ID */
    app_id: z.coerce.number().optional().nullable(),
    /** 应用名称 */
    app_name: z.string().optional(),
    /** 状态 */
    status: z.coerce.number().optional().nullable(),
    /** 客户端ID */
    client_id: z.string().optional(),
    /** 客户端ID */
    detail_data: z.boolean().default(false),
    ...PageParam,
});
export type AppListParamType = z.infer<typeof AppListParamSchema>;


// 应用项数据结构
export const AppItemSchema = z.object({
    /** 应用ID */
    id: z.coerce.number(),
    /** 应用名称 */
    name: z.string(),
    /** 客户端ID */
    client_id: z.string().nullable().optional(),
    /** 状态 */
    status: z.coerce.number(),
    /** 所属用户ID */
    user_id: z.coerce.number(),
    /** 修改时间(秒) */
    change_time: UnixTimestampSchema,
    /** 修改用户ID */
    change_user_id: z.coerce.number(),
    /** 外部功能列表 */
    exter_feature: z.array(z.any()).nullable().optional(),
    /** 是否支持外部登录 */
    exter_login: BoolSchema,
    /** 是否支持OAuth客户端 */
    oauth_client: BoolSchema,
    /** OAuth客户端数据 */
    oauth_client_data: OAuthClientDataSchema.nullable().optional(),
    /** 是否开通OAuth服务功能 */
    oauth_server: BoolSchema,
    /** OAuth服务器范围数据 */
    oauth_server_scope_data: z.array(OAuthServerScopeDataSchema).nullable().optional(),
    /** 是否开通子应用功能 */
    sup_app: BoolSchema,
    /** 子应用统计数量 */
    sub_app_count: SubAppCountSchema.nullable().optional(),
    /** 用户数据 */
    user_data: UserDataRes.nullable().optional(),
    /** 请求数量 */
    req_count: z.coerce.number().optional().nullable(),
});
export type AppItemType = z.infer<typeof AppItemSchema>;

export const AppListResSchema = z.object({
    data: z.array(AppItemSchema),
    ...PageRes,
});
export type AppListResType = z.infer<typeof AppListResSchema>;

/**
 * 获取应用列表
 * @description 分页获取系统中的应用列表，支持多种筛选条件
 */
export async function appList(
    param: AppListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppListResType>> {
    const cleanedParam = cleanEmptyStringParams(param, ['app_name', 'client_id']);
    const { data } = await authApi().post("/api/system/app/list", cleanedParam, config);
    return parseResData(data, AppListResSchema as any);
}

/**
 * 应用操作管理
 */

// 应用确认参数
export const AppConfirmParamSchema = z.object({
    /** 应用请求ID */
    app_req_id: z.coerce.number().min(1, "应用请求ID必须大于0"),
    /** 确认状态 */
    confirm_status: z.coerce.number().min(1, "确认状态必须大于0"),
    /** 确认备注 */
    confirm_note: z.string(),
});
export type AppConfirmParamType = z.infer<typeof AppConfirmParamSchema>;

/**
 * 确认应用
 * @description 确认待审核的应用，使其变为正常状态
 */
export async function appConfirm(
    param: AppConfirmParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/app/confirm", param, config);
    return data;
}

// 应用删除参数
export const AppDeleteParamSchema = z.object({
    /** 应用ID */
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
});
export type AppDeleteParamType = z.infer<typeof AppDeleteParamSchema>;

/**
 * 删除应用
 * @description 删除指定的应用及其相关数据
 */
export async function appDelete(
    param: AppDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/app/delete", param, config);
    return data;
}

// 应用禁用参数
export const AppDisableParamSchema = z.object({
    /** 应用ID */
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
});
export type AppDisableParamType = z.infer<typeof AppDisableParamSchema>;

/**
 * 禁用应用
 * @description 禁用指定的应用，使其无法正常使用
 */
export async function appDisable(
    param: AppDisableParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/app/disable", param, config);
    return data;
}

// 应用登出参数
export const AppLogoutParamSchema = z.object({
    /** 应用ID */
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
});
export type AppLogoutParamType = z.infer<typeof AppLogoutParamSchema>;

/**
 * 应用登出
 * @description 强制指定应用的所有用户登出
 */
export async function appLogout(
    param: AppLogoutParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/app/app_logout", param, config);
    return data;
}

/**
 * OAuth配置管理
 */

// OAuth客户端确认参数
export const OAuthClientConfirmParamSchema = z.object({
    /** 应用ID */
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    /** 确认状态 */
    confirm_status: z.coerce.number().min(1, "确认状态必须大于0"),
    /** 确认备注 */
    confirm_note: z.string(),
});
export type OAuthClientConfirmParamType = z.infer<typeof OAuthClientConfirmParamSchema>;

/**
 * 确认OAuth客户端
 * @description 确认应用的OAuth客户端功能
 */
export async function oauthClientConfirm(
    param: OAuthClientConfirmParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/app/oauth_client_confirm", param, config);
    return data;
}

// OAuth客户端范围确认参数
export const OAuthClientScopeConfirmParamSchema = z.object({
    /** 申请ID */
    app_req_id: z.coerce.number().min(1, "申请ID必须大于0"),
    /** 确认状态 */
    confirm_status: z.coerce.number().min(1, "确认状态必须大于0"),
    /** 确认备注 */
    confirm_note: z.string().optional(),
});
export type OAuthClientScopeConfirmParamType = z.infer<typeof OAuthClientScopeConfirmParamSchema>;

/**
 * 确认OAuth客户端范围
 * @description 确认OAuth客户端的授权范围配置
 */
export async function oauthClientScopeConfirm(
    param: OAuthClientScopeConfirmParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/app/oauth_client_scope_confirm", param, config);
    return data;
}

// OAuth服务器确认参数
export const OAuthServerConfirmParamSchema = z.object({
    /** 应用ID */
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    /** 确认状态 */
    confirm_status: z.coerce.number().min(1, "确认状态必须大于0"),
    /** 确认备注 */
    confirm_note: z.string(),
});
export type OAuthServerConfirmParamType = z.infer<typeof OAuthServerConfirmParamSchema>;

/**
 * 确认OAuth服务器
 * @description 确认应用的OAuth服务器功能
 */
export async function oauthServerConfirm(
    param: OAuthServerConfirmParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/app/oauth_server_confirm", param, config);
    return data;
}

// 清理OAuth访问令牌参数
export const OAuthClearAccessTokenParamSchema = z.object({
    /** 应用ID */
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    /** 访问令牌 */
    access_token: z.string().min(1, "访问令牌不能为空"),
});
export type OAuthClearAccessTokenParamType = z.infer<typeof OAuthClearAccessTokenParamSchema>;

/**
 * 清理OAuth访问令牌
 * @description 清理指定应用的OAuth访问令牌
 */
export async function oauthClearAccessToken(
    param: OAuthClearAccessTokenParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/app/oauth_clear_access_token", param, config);
    return data;
}

// 清理OAuth刷新令牌参数
export const OAuthClearRefreshTokenParamSchema = z.object({
    /** 应用ID */
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    /** 刷新令牌 */
    refresh_token: z.string().min(1, "刷新令牌不能为空"),
});
export type OAuthClearRefreshTokenParamType = z.infer<typeof OAuthClearRefreshTokenParamSchema>;

/**
 * 清理OAuth刷新令牌
 * @description 清理指定应用的OAuth刷新令牌
 */
export async function oauthClearRefreshToken(
    param: OAuthClearRefreshTokenParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/app/oauth_clear_refresh_token", param, config);
    return data;
}

/**
 * 外部功能管理
 */

// 确认外部功能参数
export const ConfirmExterFeatureParamSchema = z.object({
    /** 应用请求ID */
    app_req_id: z.coerce.number().min(1, "应用请求ID必须大于0"),
    /** 确认状态 */
    confirm_status: z.coerce.number().min(1, "确认状态必须大于0"),
    /** 确认备注 */
    confirm_note: z.string(),
});
export type ConfirmExterFeatureParamType = z.infer<typeof ConfirmExterFeatureParamSchema>;

/**
 * 确认外部功能
 * @description 确认应用的外部功能配置
 */
export async function confirmExterFeature(
    param: ConfirmExterFeatureParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/app/confirm_exter_feature", param, config);
    return data;
}

// 确认内部功能外部登录参数
export const ConfirmInnerFeatureExterLoginConfirmParamSchema = z.object({
    /** 应用ID */
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    /** 确认状态 */
    confirm_status: z.coerce.number().min(1, "确认状态必须大于0"),
    /** 确认备注 */
    confirm_note: z.string(),
});
export type ConfirmInnerFeatureExterLoginConfirmParamType = z.infer<typeof ConfirmInnerFeatureExterLoginConfirmParamSchema>;

/**
 * 确认内部功能外部登录
 * @description 确认应用的内部功能外部登录配置
 */
export async function confirmInnerFeatureExterLoginConfirm(
    param: ConfirmInnerFeatureExterLoginConfirmParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/app/confirm_inner_feature_exter_login_confirm", param, config);
    return data;
}

// 确认内部功能子应用参数
export const ConfirmInnerFeatureSubAppConfirmParamSchema = z.object({
    /** 应用ID */
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    /** 确认状态 */
    confirm_status: z.coerce.number().min(1, "确认状态必须大于0"),
    /** 确认备注 */
    confirm_note: z.string().optional(),
});
export type ConfirmInnerFeatureSubAppConfirmParamType = z.infer<typeof ConfirmInnerFeatureSubAppConfirmParamSchema>;

/**
 * 确认内部功能子应用
 * @description 确认应用的内部功能子应用配置
 */
export async function confirmInnerFeatureSubAppConfirm(
    param: ConfirmInnerFeatureSubAppConfirmParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/app/confirm_inner_feature_sub_app_confirm", param, config);
    return data;
}

/**
 * 应用申请管理
 */

// 应用申请列表参数
export const AppRequestListParamSchema = z.object({
    /** ID */
    id: z.coerce.number().optional().nullable(),
    /** 应用ID */
    app_id: z.coerce.number().optional().nullable(),
    /** 申请状态 */
    status:  z.coerce.number().optional(),
    ...PageParam,
});
export type AppRequestListParamType = z.infer<typeof AppRequestListParamSchema>;

export const AppRequestItemSchema = z.object({
    /** 申请ID */
    id: z.coerce.number(),
    /** 应用ID */
    app_id: z.coerce.number(),
    /** 父应用ID */
    parent_app_id: z.coerce.number().nullable(),

    app_name: z.string().nullable(),

    app_client: z.string().nullable(),

    app_status: z.coerce.number().nullable(),
    /** 请求用户ID */
    request_user_id: z.coerce.number(),
    /** 请求类型 */
    request_type: z.coerce.number(),
    /** 状态 */
    status: z.coerce.number().optional().nullable(),
    /** 请求时间 */
    request_time: UnixTimestampSchema,
    /** 确认时间 */
    confirm_time: UnixTimestampSchema.optional(),
    /** 确认用户ID */
    confirm_user_id: z.coerce.number().optional().nullable(),
    /** 确认备注 */
    confirm_note: z.string().optional(),
    /** 变更数据 */
    change_data: z.object({
        client_id: z.string().optional(),
        name: z.string().optional(),
    }).nullable().optional(),
    /** 功能数据 */
    feature_data: z.object({
        feature: z.string().optional(),
    }).nullable().optional(),
    /** OAuth客户端数据 */
    oauth_client_data: z.object({
        scope_data: z.array(z.string()).optional(),
    }).nullable().optional(),
    /** 用户数据 */
    request_user_data: UserDataRes.nullable().optional(),
    confirm_user_data: UserDataRes.nullable().optional(),
});
export type AppRequestItemType = z.infer<typeof AppRequestItemSchema>;

export const AppRequestListResSchema = z.object({
    data: z.array(AppRequestItemSchema),
    ...PageRes,
});
export type AppRequestListResType = z.infer<typeof AppRequestListResSchema>;

/**
 * 获取应用申请列表
 * @description 分页获取系统中的应用申请记录
 */
export async function appRequestList(
    param: AppRequestListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRequestListResType>> {
    const { data } = await authApi().post("/api/system/app/request_list", param, config);
    return parseResData(data, AppRequestListResSchema);
}


