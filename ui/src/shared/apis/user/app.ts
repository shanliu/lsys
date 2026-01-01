import { authApi } from "@shared/lib/apis/api_auth";
import { cleanEmptyStringParams, parseResData } from "@shared/lib/apis/utils";
import { DictListSchema } from "@shared/types/apis-dict";
import { ApiResult } from "@shared/types/apis-rest";
import { BoolSchema, LimitParam, LimitRes, PageParam, PageRes, UnixTimestampSchema } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

export const AppListParamSchema = z.object({
    parent_app_id: z.coerce.number().optional().nullable(),
    app_id: z.coerce.number().optional().nullable(),
    status: z.coerce.number().optional().nullable(),
    client_id: z.string().optional().nullable(),
    attr_inner_feature: BoolSchema.optional().nullable(),
    attr_exter_feature: BoolSchema.optional().nullable(),
    attr_sub_app_count: BoolSchema.optional().nullable(),
    attr_oauth_client_data: BoolSchema.optional().nullable(),
    attr_oauth_server_data: BoolSchema.optional().nullable(),
    attr_parent_app: BoolSchema.optional().nullable(),
    ...PageParam,
});
export type AppListParamType = z.infer<typeof AppListParamSchema>;

// OAuth Server Scope Item Schema
const OAuthServerScopeItemSchema = z.object({
    scope_key: z.string(),
    scope_name: z.string(),
});

export type OAuthServerScopeItemType = z.infer<typeof OAuthServerScopeItemSchema>;

// App List Item Schema
const AppListItemSchema = z.object({
    change_time: UnixTimestampSchema,
    change_user_id: z.coerce.number(),
    client_id: z.string(),
    exter_feature: z.array(z.any()),
    exter_login: BoolSchema,
    id: z.coerce.number(),
    name: z.string(),
    oauth_client: BoolSchema,
    oauth_client_data: z.object({
        callback_domain: z.string().optional(),
        scope_data: z.string().optional(),
    }).nullable().optional(),
    oauth_server: BoolSchema,
    oauth_server_scope_data: z.array(OAuthServerScopeItemSchema).nullable().optional(),
    parent_app_id: z.coerce.number(),
    parent_app: z.object({
        id: z.coerce.number(),
        name: z.string(),
        client_id: z.string(),
        user_id: z.coerce.number(),
        status: z.coerce.number(),
    }).nullable().optional(),
    status: z.coerce.number(),
    sub_app_count: z.object({
        disable: z.coerce.number(),
        enable: z.coerce.number(),
        init: z.coerce.number(),
    }).nullable().optional(),
    sup_app: BoolSchema,
    user_data: z.object({
        app_id: z.coerce.number(),
        id: z.coerce.number(),
        user_account: z.string(),
        user_data: z.string(),
        user_nickname: z.string(),
    }).nullable().optional(),
    user_id: z.coerce.number(),
});
export type AppListItemType = z.infer<typeof AppListItemSchema>;

export const AppListResSchema = z.object({
    data: z.array(AppListItemSchema),
    ...PageRes,
});
export type AppListResType = z.infer<typeof AppListResSchema>;

//应用列表数据
export async function appList(
    param: AppListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppListResType>> {
    const cleanedParam = cleanEmptyStringParams(param, ['client_id']);
    const { data } = await authApi().post("/api/user/app/list", cleanedParam, config);
    return parseResData(data, AppListResSchema);
}

// 应用映射数据
export const AppMapResSchema = z.object({
    app_status: DictListSchema,
    exter_features: DictListSchema,
    notify_method: DictListSchema,
    notify_status: DictListSchema,
    request_status: DictListSchema,
    request_type: DictListSchema,
});
export type AppMapResType = z.infer<typeof AppMapResSchema>;

export async function appMapping(
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppMapResType>> {

    const { data } = await authApi().post("/api/user/app/mapping", {}, config);

    return parseResData(data, AppMapResSchema);
}

// 添加应用
export const AppAddParamSchema = z.object({
    parent_app_id: z.coerce.number().nullable().optional(),
    name: z.string().min(1, "应用名称不能为空").max(50, "应用名称不能超过50个字符"),
    client_id: z.string().min(1, "应用标识不能为空").max(50, "应用标识不能超过50个字符"),
});
export type AppAddParamType = z.infer<typeof AppAddParamSchema>;

export const AppAddResSchema = z.object({
    id: z.coerce.number(),
});
export type AppAddResType = z.infer<typeof AppAddResSchema>;

export async function appAdd(
    param: AppAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppAddResType>> {
    const { data } = await authApi().post("/api/user/app/add", param, config);
    return parseResData(data, AppAddResSchema);
}

// 修改应用
export const AppChangeParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    name: z.string().min(1, "应用名称不能为空"),
    client_id: z.string().min(1, "应用标识不能为空"),
});
export type AppChangeParamType = z.infer<typeof AppChangeParamSchema>;

export async function appChange(
    param: AppChangeParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app/change", param, config);
    return data;
}

// 删除应用
export const AppDeleteParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
});
export type AppDeleteParamType = z.infer<typeof AppDeleteParamSchema>;

export async function appDelete(
    param: AppDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app/delete", param, config);
    return data;
}

// 确认应用申请
export const AppConfirmParamSchema = z.object({
    app_req_id: z.coerce.number().min(1, "申请请求ID必须大于0"),
    confirm_status: z.coerce.number().min(1, "确认状态必须大于0"),
    confirm_note: z.string().optional(),
});
export type AppConfirmParamType = z.infer<typeof AppConfirmParamSchema>;

export async function appConfirm(
    param: AppConfirmParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app/confirm", param, config);
    return data;
}

// 查看应用密钥信息
export const AppSecretViewParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    app_secret: BoolSchema,
    notify_secret: BoolSchema,
    oauth_secret: BoolSchema,
});
export type AppSecretViewParamType = z.infer<typeof AppSecretViewParamSchema>;

// Secret Item Schema
const SecretItemSchema = z.object({
    secret_data: z.string(),
    time_out: UnixTimestampSchema,
});

export type SecretItemType = z.infer<typeof SecretItemSchema>;

export const AppSecretViewResSchema = z.object({
    app_secret: z.array(SecretItemSchema).nullable().optional(),
    notify_secret: z.object({
        secret: z.string(),
        timeout: UnixTimestampSchema,
    }).nullable().optional(),
    oauth_secret: z.array(SecretItemSchema).nullable().optional(),
});
export type AppSecretViewResType = z.infer<typeof AppSecretViewResSchema>;

export async function appSecretView(
    param: AppSecretViewParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppSecretViewResType>> {
    const { data } = await authApi().post("/api/user/app/secret_view", param, config);
    return parseResData(data, AppSecretViewResSchema);
}

// 添加应用密钥
export const AppSecretAddParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    secret: z.string().nullable().optional(),
    secret_timeout: z.coerce.number().min(0, "密钥超时时间必须大于等于0"),
});
export type AppSecretAddParamType = z.infer<typeof AppSecretAddParamSchema>;

export const AppSecretAddResSchema = z.object({
    data: z.string(),
});
export type AppSecretAddResType = z.infer<typeof AppSecretAddResSchema>;

export async function appSecretAdd(
    param: AppSecretAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppSecretAddResType>> {
    const { data } = await authApi().post("/api/user/app/app_secret_add", param, config);
    return parseResData(data, AppSecretAddResSchema);
}

// 修改应用密钥
export const AppSecretChangeParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    old_secret: z.string().min(1, "旧密钥不能为空"),
    secret: z.string().min(1, "新密钥不能为空"),
    secret_timeout: z.coerce.number().min(0, "密钥超时时间必须大于等于0"),
});
export type AppSecretChangeParamType = z.infer<typeof AppSecretChangeParamSchema>;

export const AppSecretChangeResSchema = z.object({
    data: z.string(),
});
export type AppSecretChangeResType = z.infer<typeof AppSecretChangeResSchema>;

export async function appSecretChange(
    param: AppSecretChangeParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppSecretChangeResType>> {
    const { data } = await authApi().post("/api/user/app/app_secret_change", param, config);
    return parseResData(data, AppSecretChangeResSchema);
}

// 删除应用密钥
export const AppSecretDelParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    old_secret: z.string().min(1, "旧密钥不能为空"),
});
export type AppSecretDelParamType = z.infer<typeof AppSecretDelParamSchema>;

export async function appSecretDel(
    param: AppSecretDelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app/app_secret_del", param, config);
    return data;
}

// 审核子应用的外部功能申请
export const AppConfirmExterFeatureParamSchema = z.object({
    app_req_id: z.coerce.number().min(1, "申请ID必须大于0"),
    confirm_status: z.coerce.number().min(1, "审核状态必须大于0"),
    confirm_note: z.string().min(1, "审核备注不能为空"),
});
export type AppConfirmExterFeatureParamType = z.infer<typeof AppConfirmExterFeatureParamSchema>;

export async function appConfirmExterFeature(
    param: AppConfirmExterFeatureParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app/confirm_exter_feature", param, config);
    return data;
}

// 获取可用于申请子应用的父应用列表
export const AppParentListParamSchema = z.object({
    key_word: z.string().optional(),
    ...PageParam,
});
export type AppParentListParamType = z.infer<typeof AppParentListParamSchema>;

// App Parent List Item Schema
const AppParentListItemSchema = z.object({
    change_time: UnixTimestampSchema,
    client_id: z.string(),
    id: z.coerce.number(),
    name: z.string(),
    status: z.coerce.number(),
    user_data: z.object({
        app_id: z.coerce.number(),
        id: z.coerce.number(),
        user_account: z.string(),
        user_data: z.string(),
        user_nickname: z.string(),
    }),
    user_id: z.coerce.number(),
});
export type AppParentListItemType = z.infer<typeof AppParentListItemSchema>;

export const AppParentListResSchema = z.object({
    data: z.array(AppParentListItemSchema),
    ...PageRes,
});
export type AppParentListResType = z.infer<typeof AppParentListResSchema>;

export async function appParentList(
    param: AppParentListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppParentListResType>> {
    const cleanedParam = cleanEmptyStringParams(param, ['key_word']);
    const { data } = await authApi().post("/api/user/app/parent_app", cleanedParam, config);
    return parseResData(data, AppParentListResSchema);
}

// 应用外部功能申请
export const AppRequestExterFeatureParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    featuer_data: z.array(z.string()),
});
export type AppRequestExterFeatureParamType = z.infer<typeof AppRequestExterFeatureParamSchema>;

export async function appRequestExterFeature(
    param: AppRequestExterFeatureParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app/request_exter_feature", param, config);
    return data;
}

// 申请外部登录功能
export const AppRequestExterLoginParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
});
export type AppRequestExterLoginParamType = z.infer<typeof AppRequestExterLoginParamSchema>;

export async function appRequestExterLogin(
    param: AppRequestExterLoginParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app/request_inner_feature_exter_login_request", param, config);
    return data;
}

// 子应用权限申请
export const AppSubAppRequestParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
});
export type AppSubAppRequestParamType = z.infer<typeof AppSubAppRequestParamSchema>;

export async function appSubAppRequest(
    param: AppSubAppRequestParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app/sub_app_request", param, config);
    return data;
}

// 应用请求列表
export const AppRequestListParamSchema = z.object({
    id: z.coerce.number().optional().nullable(),
    app_id: z.coerce.number().optional().nullable(),
    status: z.coerce.number().optional().nullable(),
    request_type: z.coerce.number().optional().nullable(),
    ...PageParam,
});
export type AppRequestListParamType = z.infer<typeof AppRequestListParamSchema>;

// 应用请求项数据结构
export const AppRequestItemSchema = z.object({
    /** 申请ID */
    id: z.coerce.number(),
    /** 应用ID */
    app_id: z.coerce.number(),
    /** 父应用ID */
    parent_app_id: z.coerce.number().nullable().optional(),
    /** 父应用名称 */
    parent_app_name: z.string().nullable().optional(),
    /** 父应用标识 */
    parent_app_client_id: z.string().nullable().optional(),
    //父应用属于的USERid
    parent_app_user_id: z.coerce.number().nullable().optional(),
    /** 应用名称 */
    app_name: z.string().nullable().optional(),
    /** 应用标识 */
    app_client: z.string().nullable().optional(),
    /** 应用状态 */
    app_status: z.coerce.number().nullable().optional(),
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
    /** 确认用户数据 */
    confirm_user_data: z.object({
        app_id: z.coerce.number(),
        id: z.coerce.number(),
        user_account: z.string(),
        user_data: z.string(),
        user_nickname: z.string(),
    }).nullable().optional(),
    /** 确认备注 */
    confirm_note: z.string().nullable().optional(),
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
});
export type AppRequestItemType = z.infer<typeof AppRequestItemSchema>;

export const AppRequestListResSchema = z.object({
    data: z.array(AppRequestItemSchema),
    ...PageRes,
});
export type AppRequestListResType = z.infer<typeof AppRequestListResSchema>;

export async function appRequestList(
    param: AppRequestListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppRequestListResType>> {
    const { data } = await authApi().post("/api/user/app/request_list", param, config);
    return parseResData(data, AppRequestListResSchema);
}

// 指定应用的子应用请求列表
export const AppSubRequestListParamSchema = z.object({
    id: z.coerce.number().optional().nullable(),
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    sub_app_id: z.coerce.number().optional().nullable(),
    status: z.coerce.number().optional().nullable(),
    ...PageParam,
});
export type AppSubRequestListParamType = z.infer<typeof AppSubRequestListParamSchema>;

// 子应用请求项数据结构
export const AppSubRequestItemSchema = z.object({
    /** 申请ID */
    id: z.coerce.number(),
    /** 应用ID */
    app_id: z.coerce.number(),
    /** 请求用户ID */
    request_user_id: z.coerce.number(),
    /** 请求类型 */
    request_type: z.coerce.number(),
    /** 状态 */
    status: z.coerce.number(),
    /** 请求时间 */
    request_time: UnixTimestampSchema,
    /** 确认时间 */
    confirm_time: UnixTimestampSchema.optional(),
    /** 确认用户ID */
    confirm_user_id: z.coerce.number().optional().nullable(),
    /** 确认备注 */
    confirm_note: z.string().nullable().optional(),
    /** 变更数据 */
    change_data: z.object({
        client_id: z.string().optional(),
        name: z.string().optional(),
    }).nullable().optional(),
    /** 功能数据 - 申请外部功能数据数组 */
    feature_data: z.object({
        feature: z.string().optional(),
    }).nullable().optional(),
    /** OAuth客户端数据 */
    oauth_client_data: z.object({
        scope_data: z.array(z.string()).optional(),
    }).nullable().optional(),
    /** 请求用户数据 */
    user_data: z.object({
        app_id: z.coerce.number(),
        id: z.coerce.number(),
        user_account: z.string(),
        user_data: z.string(),
        user_nickname: z.string(),
    }).nullable().optional(),
    /** 应用名称 */
    app_name: z.string().nullable(),
    /** 应用标识 */
    app_client: z.string().nullable(),
    /** 应用状态 */
    app_status: z.coerce.number().nullable(),
});
export type AppSubRequestItemType = z.infer<typeof AppSubRequestItemSchema>;

export const AppSubRequestListResSchema = z.object({
    data: z.array(AppSubRequestItemSchema),
    ...PageRes,
});
export type AppSubRequestListResType = z.infer<typeof AppSubRequestListResSchema>;

export async function appSubRequestList(
    param: AppSubRequestListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppSubRequestListResType>> {
    const { data } = await authApi().post("/api/user/app/sub_request_list", param, config);
    return parseResData(data, AppSubRequestListResSchema);
}

// 查看指定子应用的密钥信息
export const AppSubSecretViewParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    app_secret: BoolSchema.optional(),
    notify_secret: BoolSchema.optional(),
    oauth_secret: BoolSchema.optional(),
});
export type AppSubSecretViewParamType = z.infer<typeof AppSubSecretViewParamSchema>;

export const AppSubSecretViewResSchema = z.object({
    app_secret: z.array(SecretItemSchema).nullable().optional(),
    notify_secret: z.object({
        secret: z.string(),
        timeout: UnixTimestampSchema,
    }).nullable().optional(),
    oauth_secret: z.array(SecretItemSchema).nullable().optional(),
});
export type AppSubSecretViewResType = z.infer<typeof AppSubSecretViewResSchema>;

export async function appSubSecretView(
    param: AppSubSecretViewParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppSubSecretViewResType>> {
    const { data } = await authApi().post("/api/user/app/sub_app_secret_view", param, config);
    return parseResData(data, AppSubSecretViewResSchema);
}

// 回调密钥变更
export const AppNotifySecretChangeParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    secret: z.string().min(1, "密钥不能为空").optional(),
    secret_timeout: z.coerce.number().min(0, "超时时间必须大于等于0"),
});
export type AppNotifySecretChangeParamType = z.infer<typeof AppNotifySecretChangeParamSchema>;

export const AppNotifySecretChangeResSchema = z.object({
    data: z.string(),
});
export type AppNotifySecretChangeResType = z.infer<typeof AppNotifySecretChangeResSchema>;

export async function appNotifySecretChange(
    param: AppNotifySecretChangeParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppNotifySecretChangeResType>> {
    const { data } = await authApi().post("/api/user/app/notify_secret_change", param, config);
    return parseResData(data, AppNotifySecretChangeResSchema);
}

// 应用回调通知删除
export const AppNotifyDelParamSchema = z.object({
    id: z.coerce.number().min(1, "通知ID必须大于0"),
});
export type AppNotifyDelParamType = z.infer<typeof AppNotifyDelParamSchema>;

export async function appNotifyDel(
    param: AppNotifyDelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app/notify_del", param, config);
    return data;
}

// 应用回调通知列表
export const AppNotifyListParamSchema = z.object({
    app_id: z.coerce.number().optional().nullable(),
    method: z.string().optional(),
    status: z.coerce.number().optional(),
    attr_callback_data: BoolSchema.optional().nullable(),
    ...LimitParam,
});
export type AppNotifyListParamType = z.infer<typeof AppNotifyListParamSchema>;

export const AppNotifyListItemSchema = z.object({
    app_id: z.coerce.number(),
    call_url: z.string(),
    id: z.coerce.number(),
    next_time: UnixTimestampSchema,
    notify_key: z.string(),
    notify_method: z.string(),
    notify_type: z.string(),
    notify_payload: z.string(),
    publish_time: UnixTimestampSchema,
    result: z.string(),
    status: z.coerce.number(),
    try_max: z.coerce.number(),
    try_num: z.coerce.number(),
});
export type AppNotifyListItemType = z.infer<typeof AppNotifyListItemSchema>;

export const AppNotifyListResSchema = z.object({
    data: z.array(AppNotifyListItemSchema),
    ...LimitRes,
});
export type AppNotifyListResType = z.infer<typeof AppNotifyListResSchema>;

export async function appNotifyList(
    param: AppNotifyListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppNotifyListResType>> {
    if (param?.method=="")delete param.method;
    const { data } = await authApi().post("/api/user/app/notify_list", param, config);
    return data;
}
export const AppSubNotifyGetConfigParamSchema = z.object({
    app_id: z.coerce.number().min(1, "子应用ID必须大于0"),
});
export type AppSubNotifyGetConfigParamType = z.infer<typeof AppSubNotifyGetConfigParamSchema>;


// 子应用回调信息配置查询
export async function appSubNotifyGetConfig(
    param: AppSubNotifyGetConfigParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app/sub_app_notify_get_config", param, config);
    return data;
}

// 子应用设置回调路径
export const AppSubNotifySetConfigParamSchema = z.object({
    app_id: z.coerce.number().min(1, "子应用ID必须大于0"),
    url: z.string().min(1, "回调通知地址不能为空"),
});
export type AppSubNotifySetConfigParamType = z.infer<typeof AppSubNotifySetConfigParamSchema>;

export async function appSubNotifySetConfig(
    param: AppSubNotifySetConfigParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app/sub_app_notify_set_config", param, config);
    return data;
}

// OAuth登录申请
export const AppOAuthClientRequestParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    scope_data: z.array(z.string()),
});
export type AppOAuthClientRequestParamType = z.infer<typeof AppOAuthClientRequestParamSchema>;

export async function appOAuthClientRequest(
    param: AppOAuthClientRequestParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app/oauth_client_request", param, config);
    return data;
}

// OAuth登录可用的scope列表
export const AppOAuthClientScopeDataParamSchema = z.object({
    app_id: z.coerce.number().optional().nullable(),
});
export type AppOAuthClientScopeDataParamType = z.infer<typeof AppOAuthClientScopeDataParamSchema>;

// OAuth Scope Item Schema
const OAuthScopeItemSchema = z.object({
    desc: z.string(),
    key: z.string(),
    name: z.string(),
});
export type OAuthScopeItemType = z.infer<typeof OAuthScopeItemSchema>;

export const AppOAuthClientScopeDataResSchema = z.object({
    scope: z.array(OAuthScopeItemSchema),
});
export type AppOAuthClientScopeDataResType = z.infer<typeof AppOAuthClientScopeDataResSchema>;

export async function appOAuthClientScopeData(
    param: AppOAuthClientScopeDataParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppOAuthClientScopeDataResType>> {
    const { data } = await authApi().post("/api/user/app/oauth_client_scope_data", param, config);
    return parseResData(data, AppOAuthClientScopeDataResSchema);
}

// OAuth登录申请新scope
export const AppOAuthClientScopeRequestParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    scope_data: z.array(z.string()),
});
export type AppOAuthClientScopeRequestParamType = z.infer<typeof AppOAuthClientScopeRequestParamSchema>;

export async function appOAuthClientScopeRequest(
    param: AppOAuthClientScopeRequestParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app/oauth_client_scope_request", param, config);
    return data;
}

// OAuth登录密钥添加
export const AppOAuthClientSecretAddParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    secret: z.string().nullable().optional(),
    secret_timeout: z.coerce.number().min(0, "密钥超时时间必须大于等于0"),
});
export type AppOAuthClientSecretAddParamType = z.infer<typeof AppOAuthClientSecretAddParamSchema>;

export const AppOAuthClientSecretAddResSchema = z.object({
    data: z.string(),
});
export type AppOAuthClientSecretAddResType = z.infer<typeof AppOAuthClientSecretAddResSchema>;

export async function appOAuthClientSecretAdd(
    param: AppOAuthClientSecretAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppOAuthClientSecretAddResType>> {
    const { data } = await authApi().post("/api/user/app/oauth_client_secret_add", param, config);
    return parseResData(data, AppOAuthClientSecretAddResSchema);
}

// OAuth登录密钥更改
export const AppOAuthClientSecretChangeParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    old_secret: z.string().min(1, "旧密钥不能为空"),
    secret: z.string().min(1, "新密钥不能为空"),
    secret_timeout: z.coerce.number().min(0, "密钥超时时间必须大于等于0"),
});
export type AppOAuthClientSecretChangeParamType = z.infer<typeof AppOAuthClientSecretChangeParamSchema>;

export const AppOAuthClientSecretChangeResSchema = z.object({
    data: z.string(),
});
export type AppOAuthClientSecretChangeResType = z.infer<typeof AppOAuthClientSecretChangeResSchema>;

export async function appOAuthClientSecretChange(
    param: AppOAuthClientSecretChangeParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppOAuthClientSecretChangeResType>> {
    const { data } = await authApi().post("/api/user/app/oauth_client_secret_change", param, config);
    return parseResData(data, AppOAuthClientSecretChangeResSchema);
}

// OAuth登录密钥删除
export const AppOAuthClientSecretDelParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    old_secret: z.string().min(1, "旧的密钥不能为空"),
});
export type AppOAuthClientSecretDelParamType = z.infer<typeof AppOAuthClientSecretDelParamSchema>;

export async function appOAuthClientSecretDel(
    param: AppOAuthClientSecretDelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app/oauth_client_secret_del", param, config);
    return data;
}

// 设置OAuth登录回调域名
export const AppOAuthClientSetDomainParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    callback_domain: z.string().min(1, "回调域名不能为空"),
});
export type AppOAuthClientSetDomainParamType = z.infer<typeof AppOAuthClientSetDomainParamSchema>;

export async function appOAuthClientSetDomain(
    param: AppOAuthClientSetDomainParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app/oauth_client_set_domain", param, config);
    return data;
}

// OAuth服务端申请
export const AppOAuthServerRequestParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
});
export type AppOAuthServerRequestParamType = z.infer<typeof AppOAuthServerRequestParamSchema>;

export async function appOAuthServerRequest(
    param: AppOAuthServerRequestParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app/oauth_server_request", param, config);
    return data;
}

// OAuth Server Setting Scope Item Schema
const OAuthServerSettingScopeItemSchema = z.object({
    key: z.string().min(1, "权限标识不能为空"),
    name: z.string().min(1, "权限名称不能为空"),
    desc: z.string().min(1, "权限描述不能为空"),
});
export type OAuthServerSettingScopeItemType = z.infer<typeof OAuthServerSettingScopeItemSchema>;

// OAuth服务器设置
export const AppOAuthServerSettingParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    scope_data: z.array(OAuthServerSettingScopeItemSchema),
});
export type AppOAuthServerSettingParamType = z.infer<typeof AppOAuthServerSettingParamSchema>;

export async function appOAuthServerSetting(
    param: AppOAuthServerSettingParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app/oauth_server_setting", param, config);
    return data;
}

// 审核子应用OAuth登录请求
export const AppOAuthServerClientConfirmParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    confirm_status: z.coerce.number().min(1, "审核状态必须大于0"),
    confirm_note: z.string().min(1, "审核备注不能为空"),
});
export type AppOAuthServerClientConfirmParamType = z.infer<typeof AppOAuthServerClientConfirmParamSchema>;

export async function appOAuthServerClientConfirm(
    param: AppOAuthServerClientConfirmParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app/oauth_server_client_confirm", param, config);
    return data;
}

// 子应用OAuth登录申请scope权限审核
export const AppOAuthServerClientScopeConfirmParamSchema = z.object({
    app_req_id: z.coerce.number().min(1, "应用请求ID必须大于0"),
    confirm_status: z.coerce.number().min(1, "确认状态必须大于0"),
    confirm_note: z.string().min(1, "确认备注不能为空"),
});
export type AppOAuthServerClientScopeConfirmParamType = z.infer<typeof AppOAuthServerClientScopeConfirmParamSchema>;

export async function appOAuthServerClientScopeConfirm(
    param: AppOAuthServerClientScopeConfirmParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app/oauth_server_client_scope_confirm", param, config);
    return data;
}

// 获取子应用列表
export const AppSubAppListParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    sub_app_id: z.coerce.number().optional().nullable(),
    status: z.coerce.number().optional().nullable(),
    ...PageParam,
});
export type AppSubAppListParamType = z.infer<typeof AppSubAppListParamSchema>;

// Sub App List Item Schema
const AppSubAppListItemSchema = z.object({
    change_time: UnixTimestampSchema,
    change_user_id: z.coerce.number(),
    client_id: z.string(),
    exter_feature: z.array(z.string()),
    id: z.coerce.number(),
    name: z.string(),
    oauth_client: BoolSchema,
    oauth_client_data: z.object({
        callback_domain: z.string(),
        scope_data: z.string(),
    }).nullable().optional(),
    status: z.coerce.number(),
    sup_app: BoolSchema,
    user_data: z.object({
        app_id: z.coerce.number(),
        id: z.coerce.number(),
        user_account: z.string(),
        user_data: z.string(),
        user_nickname: z.string(),
    }).nullable().optional(),
    user_id: z.coerce.number(),
    /** 子应用请求待处理数量 */
    sub_req_pending_count: z.coerce.number().optional().nullable(),
});
export type AppSubAppListItemType = z.infer<typeof AppSubAppListItemSchema>;

export const AppSubAppListResSchema = z.object({
    data: z.array(AppSubAppListItemSchema),
    ...PageRes,
});
export type AppSubAppListResType = z.infer<typeof AppSubAppListResSchema>;

export async function appSubAppList(
    param: AppSubAppListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppSubAppListResType>> {
    const { data } = await authApi().post("/api/user/app/sub_app_list", param, config);
    return parseResData(data, AppSubAppListResSchema);
}

/**
 * 应用统计数据接口
 */
export const AppStatParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    days: z.coerce.number().min(1, "天数必须大于0").max(365, "天数不能超过365"),
});
export type AppStatParamType = z.infer<typeof AppStatParamSchema>;

// 通知数据项
const NotifyDataItemSchema = z.object({
    date: z.string(),
    notify_type: z.string(),
    status: z.coerce.number(),
    total: z.coerce.number(),
});

// OAuth访问数据项
const OAuthAccessItemSchema = z.object({
    date: z.string(),
    total: z.coerce.number(),
});

// 请求数据项
const RequestItemSchema = z.object({
    date: z.string(),
    status: z.coerce.number(),
    total: z.coerce.number(),
});

// 子应用数据项
const SubAppItemSchema = z.object({
    date: z.string(),
    status: z.coerce.number(),
    total: z.coerce.number(),
});

export const AppStatResSchema = z.object({
    data: z.object({
        notify_data: z.object({
            all: z.array(NotifyDataItemSchema),
            success: z.array(NotifyDataItemSchema),
        }).optional(),
        oauth_access: z.array(OAuthAccessItemSchema).optional(),
        request: z.object({
            all: z.array(RequestItemSchema),
            processed: z.array(RequestItemSchema),
        }).optional(),
        sub_app: z.object({
            all: z.array(SubAppItemSchema),
            enable: z.array(SubAppItemSchema),
        }).optional(),
    }).optional(),
});
export type AppStatResType = z.infer<typeof AppStatResSchema>;

export async function appStat(
    param: AppStatParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AppStatResType>> {
    const { data } = await authApi().post("/api/user/app/stat", param, config);
    return parseResData(data, AppStatResSchema);
}


