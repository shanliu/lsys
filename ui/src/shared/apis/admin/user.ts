import { authApi } from "@shared/lib/apis/api_auth";
import { cleanEmptyStringParams, parseResData } from "@shared/lib/apis/utils";
import { DictListSchema } from "@shared/types/apis-dict";
import { ApiResult } from "@shared/types/apis-rest";
import { BoolSchema, LimitParam, LimitRes, UnixTimestampSchema, UserDataRes } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

// User Management APIs (System)
export const SystemUserAccountDetailParamSchema = z.object({
    account_id: z.coerce.number().min(1, "账号ID不能为空"),
    enable: BoolSchema.optional(),
    base: BoolSchema.optional(),
    name: BoolSchema.optional(),
    info: BoolSchema.optional(),
    address: BoolSchema.optional(),
    external: z.array(z.string()).optional(),
    email: z.array(z.coerce.number()).optional(),
    mobile: z.array(z.coerce.number()).optional(),
});
export type SystemUserAccountDetailParamType = z.infer<typeof SystemUserAccountDetailParamSchema>;

// 地址信息
export const AddressItemSchema = z.object({
    account_id: z.coerce.number(),
    address_code: z.string(),
    address_detail: z.string(),
    address_info: z.string(),
    change_time: UnixTimestampSchema,
    country_code: z.string(),
    id: z.coerce.number(),
    mobile: z.string(),
    name: z.string(),
    status: z.coerce.number(),
});
export type AddressItemType = z.infer<typeof AddressItemSchema>;

// 邮箱信息
export const EmailItemSchema = z.object({
    account_id: z.coerce.number(),
    change_time: UnixTimestampSchema,
    confirm_time: UnixTimestampSchema,
    email: z.string(),
    id: z.coerce.number(),
    status: z.coerce.number(),
});
export type EmailItemType = z.infer<typeof EmailItemSchema>;

// 手机信息
export const MobileItemSchema = z.object({
    account_id: z.coerce.number(),
    area_code: z.string(),
    change_time: UnixTimestampSchema,
    confirm_time: UnixTimestampSchema,
    id: z.coerce.number(),
    mobile: z.string(),
    status: z.coerce.number(),
});
export type MobileItemType = z.infer<typeof MobileItemSchema>;

// 用户信息
export const UserInfoSchema = z.object({
    account_id: z.coerce.number(),
    birthday: z.string(),
    change_time: UnixTimestampSchema,
    gender: z.string(),
    headimg: z.string(),
    id: z.coerce.number(),
    reg_from: z.string(),
    reg_ip: z.string(),
});
export type UserInfoType = z.infer<typeof UserInfoSchema>;

// 用户名信息
export const UserNameSchema = z.object({
    account_id: z.coerce.number(),
    change_time: UnixTimestampSchema,
    id: z.coerce.number(),
    status: z.coerce.number(),
    username: z.string(),
});
export type UserNameType = z.infer<typeof UserNameSchema>;

// 用户基础信息
export const UserBaseSchema = z.object({
    add_time: UnixTimestampSchema,
    address_count: z.coerce.number(),
    change_time: UnixTimestampSchema,
    confirm_time: UnixTimestampSchema,
    email_count: z.coerce.number(),
    external_count: z.coerce.number(),
    id: z.coerce.number(),
    mobile_count: z.coerce.number(),
    nickname: z.string(),
    password_id: z.coerce.number(),
    status: z.coerce.number(),
    use_name: z.string(),
});
export type UserBaseType = z.infer<typeof UserBaseSchema>;

export const SystemUserAccountDetailDataSchema = z.object({
    address: z.array(AddressItemSchema).nullable().optional(),
    email: z.array(EmailItemSchema).nullable().optional(),
    external: z.any().nullable(),
    info: UserInfoSchema.nullable().optional(),
    mobile: z.array(MobileItemSchema).nullable().optional(),
    name: UserNameSchema.nullable().optional(),
    user: UserBaseSchema,
});
export type SystemUserAccountDetailDataType = z.infer<typeof SystemUserAccountDetailDataSchema>;

export const SystemUserAccountDetailResSchema = z.object({
    data: SystemUserAccountDetailDataSchema,
});
export type SystemUserAccountDetailResType = z.infer<typeof SystemUserAccountDetailResSchema>;

export async function systemUserAccountDetail(
    param: SystemUserAccountDetailParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemUserAccountDetailResType>> {
    const { data } = await authApi().post("/api/system/user/account_detail", param, config);
    return parseResData(data, SystemUserAccountDetailResSchema);
}

// User Account Search
export const SystemUserAccountSearchParamSchema = z.object({
    key_word: z.string().optional(),
    enable: BoolSchema.optional(),
    base: BoolSchema.optional(),
    name: BoolSchema.optional(),
    info: BoolSchema.optional(),
    address: BoolSchema.optional(),
    external: z.array(z.string()).optional(),
    email: z.array(z.coerce.number()).optional(),
    mobile: z.array(z.coerce.number()).optional(),
    ...LimitParam,
});
export type SystemUserAccountSearchParamType = z.infer<typeof SystemUserAccountSearchParamSchema>;

// 搜索结果中的匹配标识
export const SearchCatSchema = z.object({
    type: z.string(),
    val: z.string(),
});
export type SearchCatType = z.infer<typeof SearchCatSchema>;

export const SystemUserAccountItemSchema = z.object({
    address: z.array(AddressItemSchema).nullable().optional(),
    cat: z.array(SearchCatSchema).nullable().optional(),
    email: z.array(EmailItemSchema).nullable().optional(),
    external: z.any().nullable(),
    info: UserInfoSchema.nullable().optional(),
    mobile: z.array(MobileItemSchema).nullable().optional(),
    name: UserNameSchema.nullable().optional(),
    user: UserBaseSchema,
});
export type SystemUserAccountItemType = z.infer<typeof SystemUserAccountItemSchema>;

export const SystemUserAccountSearchResSchema = z.object({
    data: z.array(SystemUserAccountItemSchema),
    ...LimitRes,
});
export type SystemUserAccountSearchResType = z.infer<typeof SystemUserAccountSearchResSchema>;

export async function systemUserAccountSearch(
    param: SystemUserAccountSearchParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemUserAccountSearchResType>> {
    const cleanedParam = cleanEmptyStringParams(param, ['key_word']);
    const { data } = await authApi().post("/api/system/user/account_search", cleanedParam, config);
    return parseResData(data, SystemUserAccountSearchResSchema);
}

// User Change Logs
export const SystemUserChangeLogsParamSchema = z.object({
    log_type: z.string().optional(),
    add_user_id: z.coerce.number().optional().nullable(),
    ...LimitParam,
});
export type SystemUserChangeLogsParamType = z.infer<typeof SystemUserChangeLogsParamSchema>;

export const SystemUserChangeLogItemSchema = z.object({
    add_time: UnixTimestampSchema,
    add_user_id: z.coerce.number(),
    add_user_ip: z.string(),
    device_id: z.string(),
    id: z.coerce.number(),
    log_data: z.string(),
    log_type: z.string(),
    message: z.string(),
    request_id: z.string(),
    request_user_agent: z.string(),
    source_id: z.string(),
    user_data: UserDataRes.nullable().optional(),
});
export type SystemUserChangeLogItemType = z.infer<typeof SystemUserChangeLogItemSchema>;

export const SystemUserChangeLogsResSchema = z.object({
    data: z.array(SystemUserChangeLogItemSchema),
    ...LimitRes,
});
export type SystemUserChangeLogsResType = z.infer<typeof SystemUserChangeLogsResSchema>;

export async function systemUserChangeLogs(
    param: SystemUserChangeLogsParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemUserChangeLogsResType>> {
    const cleanedParam = cleanEmptyStringParams(param, ['log_type']);
    const { data } = await authApi().post("/api/system/user/change_logs", cleanedParam, config);
    return parseResData(data, SystemUserChangeLogsResSchema);
}

// User Login History
export const SystemUserLoginHistoryParamSchema = z.object({
    app_id: z.coerce.number().optional().nullable(),
    oauth_app_id: z.coerce.number().optional().nullable(),
    user_id: z.coerce.number().optional().nullable(),
    is_enable: BoolSchema.nullable(),
    ...LimitParam,
});
export type SystemUserLoginHistoryParamType = z.infer<typeof SystemUserLoginHistoryParamSchema>;

export const SystemUserLoginHistoryItemSchema = z.object({
    id: z.coerce.number(),
    add_time: UnixTimestampSchema,
    app_id: z.coerce.number(),
    device_id: z.string(),
    device_name: z.string(),
    expire_time: UnixTimestampSchema,
    login_ip: z.string(),
    login_type: z.string(),
    logout_time: UnixTimestampSchema,
    oauth_app_id: z.coerce.number(),
    status: z.coerce.number(),
    token_data: z.string(),
    user_id: z.coerce.number(),
    user_data: UserDataRes.nullable().optional(),
});
export type SystemUserLoginHistoryItemType = z.infer<typeof SystemUserLoginHistoryItemSchema>;

export const SystemUserLoginHistoryResSchema = z.object({
    data: z.array(SystemUserLoginHistoryItemSchema),
    ...LimitRes,
});
export type SystemUserLoginHistoryResType = z.infer<typeof SystemUserLoginHistoryResSchema>;

export async function systemUserLoginHistory(
    param: SystemUserLoginHistoryParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemUserLoginHistoryResType>> {
    const { data } = await authApi().post("/api/system/user/login_history", param, config);
    return parseResData(data, SystemUserLoginHistoryResSchema);
}

// User Logout
export const SystemUserLogoutParamSchema = z.object({
    app_id: z.coerce.number(),
    oauth_app_id: z.coerce.number(),
    token_data: z.string().min(1, "登录令牌不能为空"),
});
export type SystemUserLogoutParamType = z.infer<typeof SystemUserLogoutParamSchema>;

export async function systemUserLogout(
    param: SystemUserLogoutParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/user/user_logout", param, config);
    return data;
}

// User Mapping
export const SystemUserMappingResSchema = z.object({
    account_status: DictListSchema,
    change_type: DictListSchema,
    email_status: DictListSchema,
    mobile_status: DictListSchema,
    session_status: DictListSchema,
});
export type SystemUserMappingResType = z.infer<typeof SystemUserMappingResSchema>;

export async function systemUserMapping(
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemUserMappingResType>> {
    const { data } = await authApi().post("/api/system/user/mapping", {}, config);
    return parseResData(data, SystemUserMappingResSchema);
}


