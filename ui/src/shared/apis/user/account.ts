import { authApi } from "@shared/lib/apis/api_auth";
import { cleanEmptyStringParams, parseResData } from "@shared/lib/apis/utils";
import { userStore } from "@shared/lib/auth";
import { DictListSchema } from "@shared/types/apis-dict";
import { ApiResult } from "@shared/types/apis-rest";
import { BoolSchema, LimitParam, LimitRes, UnixTimestampSchema } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

// Account APIs
export const AccountCheckUsernameParamSchema = z.object({
    name: z.string().min(1, "用户名不能为空"),
});
export type AccountCheckUsernameParamType = z.infer<typeof AccountCheckUsernameParamSchema>;

export const AccountCheckUsernameResSchema = z.object({
    pass: z.string(), // "1" 表示通过（用户名可用），其他值表示不通过
});
export type AccountCheckUsernameResType = z.infer<typeof AccountCheckUsernameResSchema>;

export async function accountCheckUsername(
    param: AccountCheckUsernameParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AccountCheckUsernameResType>> {
    const { data } = await authApi().post("/api/user/base/check_username", param, config);
    return parseResData(data, AccountCheckUsernameResSchema);
}

// 获取登录数据
export const AccountLoginDataParamSchema = z.object({
    reload_auth: BoolSchema.optional(),
    auth: BoolSchema.optional(),
    user: BoolSchema.optional(),
    name: BoolSchema.optional(),
    info: BoolSchema.optional(),
    external: z.array(z.string()).optional(),
    email: z.array(z.coerce.number()).optional(),
    mobile: z.array(z.coerce.number()).optional(),
    address: BoolSchema.optional(),
    password_timeout: BoolSchema.optional(),
});
export type AccountLoginDataParamType = z.infer<typeof AccountLoginDataParamSchema>;

// 拆分 auth_data 方便复用，并允许其在未传 auth=true 时缺失
const AccountAuthDataSchema = z.object({
    account_id: z.coerce.number(),
    empty_password: BoolSchema,
    login_data: z.object({
        account_id: z.coerce.number(),
        change_time: UnixTimestampSchema,
        id: z.coerce.number(),
        status: z.coerce.number(),
        username: z.string().optional().default(""),
    }),
    login_time: UnixTimestampSchema,
    login_type: z.string(),
    time_out: z.string(),
    user_id: z.coerce.number(),
    user_nickname: z.string(),
});
export type AccountAuthDataType = z.infer<typeof AccountAuthDataSchema>;

// 后端在未请求 auth / user / info 等标记时，相关块可能缺失，所以这里全部标为可选
export const AccountLoginDataResSchema = z.object({
    auth_data: AccountAuthDataSchema.nullable().optional(),
    jwt: z.string().nullable().optional(), // 修改：支持 null
    user_data: z.object({
        address: z.array(z.any()).nullable().optional(),
        email: z.array(z.any()).nullable().optional(),
        external: z.array(z.any()).nullable().optional(),
        info: z.any().nullable().optional(),
        mobile: z.array(z.any()).nullable().optional(),
        name: z.any().nullable().optional(),
        passwrod_timeut: z.string().nullable().optional(),
        // account 基础信息（你提供的结构）统一命名为 user
        account: z.object({
            add_time: z.coerce.number().optional().nullable(),
            address_count: z.coerce.number().optional().nullable(),
            change_time: z.coerce.number().optional().nullable(),
            confirm_time: z.coerce.number().optional().nullable(),
            email_count: z.coerce.number().optional().nullable(),
            external_count: z.coerce.number().optional().nullable(),
            id: z.coerce.number().optional().nullable(),
            mobile_count: z.coerce.number().optional().nullable(),
            nickname: z.string().optional(),
            password_id: z.coerce.number().optional().nullable(),
            status: z.coerce.number().optional().nullable(),
            use_name: z.coerce.number().optional().nullable(),
        }).nullable().optional(),
    }).nullable().optional(),
});
export type AccountLoginDataResType = z.infer<typeof AccountLoginDataResSchema>;

export async function accountLoginData(
    param: AccountLoginDataParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AccountLoginDataResType>> {
    const { data } = await authApi().post("/api/auth/login_data", param || {}, config);
    return parseResData(data, AccountLoginDataResSchema);
}

// 登录历史
export const AccountLoginHistoryParamSchema = z.object({
    login_type: z.string().optional(),
    login_account: z.string().optional(),
    login_ip: z.string().optional(),
    is_login: z.coerce.number().optional().nullable(), // 1表示成功，0表示失败
    ...LimitParam,
});
export type AccountLoginHistoryParamType = z.infer<typeof AccountLoginHistoryParamSchema>;

export const AccountLoginHistoryItemSchema = z.object({
    account_id: z.coerce.number(),
    add_time: UnixTimestampSchema,
    id: z.coerce.number(),
    is_login: z.coerce.number(), // 1表示成功，0表示失败
    login_account: z.string(),
    login_city: z.string(),
    login_ip: z.string(),
    login_msg: z.string(),
    login_type: z.string(),
});
export type AccountLoginHistoryItemType = z.infer<typeof AccountLoginHistoryItemSchema>;

export const AccountLoginHistoryResSchema = z.object({
    data: z.array(AccountLoginHistoryItemSchema),
    ...LimitRes,
});
export type AccountLoginHistoryResType = z.infer<typeof AccountLoginHistoryResSchema>;

export async function accountLoginHistory(
    param: AccountLoginHistoryParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AccountLoginHistoryResType>> {
    const cleanedParam = cleanEmptyStringParams(param, ['login_type', 'login_account', 'login_ip']);
    const { data } = await authApi().post("/api/user/base/login_history", cleanedParam, config);
    return parseResData(data, AccountLoginHistoryResSchema);
}

// 退出登录
export async function accountLogout(
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/auth/logout", {}, config);
    return data;
}

// 获取密码修改信息
export const AccountPasswordModifyInfoResSchema = z.object({
    last_time: UnixTimestampSchema, // 最后修改时间（密码创建时间戳）
    remaining_time: z.coerce.number(), // 当前密码剩余有效时间（秒数，0表示已过期）
    is_expired: BoolSchema, // 当前密码是否已经过期
    total_timeout: z.coerce.number(), // 密码总有效期（秒，0表示永久有效）
});
export type AccountPasswordModifyInfoResType = z.infer<typeof AccountPasswordModifyInfoResSchema>;

export async function accountPasswordModifyInfo(
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AccountPasswordModifyInfoResType>> {
    const { data } = await authApi().post("/api/user/base/password_modify", {}, config);
    return parseResData(data, AccountPasswordModifyInfoResSchema);
}

// 修改密码
export const AccountPasswordModifyParamSchema = z.object({
    old_password: z.string().optional(),
    new_password: z.string().min(6, "新密码长度至少6位"),
});
export type AccountPasswordModifyParamType = z.infer<typeof AccountPasswordModifyParamSchema>;

export async function accountPasswordModify(
    param: AccountPasswordModifyParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/base/set_password", param, config);
    return data;
}

// 设置信息
export const AccountSetInfoParamSchema = z.object({
    nikename: z.string().optional(),
    gender: z.coerce.number().optional().nullable(),
    headimg: z.string().optional(),
    birthday: z.string().optional(),
});
export type AccountSetInfoParamType = z.infer<typeof AccountSetInfoParamSchema>;

export async function accountSetInfo(
    param: AccountSetInfoParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/base/set_info", param, config);
    return data;
}

// 设置用户名
export const AccountSetUsernameParamSchema = z.object({
    name: z.string().min(1, "用户名不能为空"),
});
export type AccountSetUsernameParamType = z.infer<typeof AccountSetUsernameParamSchema>;

export async function accountSetUsername(
    param: AccountSetUsernameParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/base/set_username", param, config);
    return data;
}

// 删除账号
export const AccountDeleteParamSchema = z.object({
    password: z.string().min(1, "密码不能为空"),
});
export type AccountDeleteParamType = z.infer<typeof AccountDeleteParamSchema>;

export async function accountDelete(
    param: AccountDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/base/delete", param, config);
    return data;
}

// 账号映射
export const AccountMappingResSchema = z.object({
    account_status: DictListSchema,
    email_status: DictListSchema,
    mobile_status: DictListSchema,
    session_status: DictListSchema,
});
export type AccountMappingResType = z.infer<typeof AccountMappingResSchema>;

export async function accountMapping(
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AccountMappingResType>> {
    const { data } = await authApi().post("/api/user/base/mapping", {}, config);
    return parseResData(data, AccountMappingResSchema);
}


export const userLogout = async (
    uid?: number,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> => {
    let userData;
    if (!uid) {
        userData = userStore.getState().current();
    } else {
        userData = userStore.getState().userData.find(u => u.userId === uid);
    }
    if (!userData) return Promise.resolve(
        {
            code: "200",
            state: "ok",
            status: true,
            message: "ok"
        } as ApiResult
    );
    const res = await authApi().post('/api/auth/logout', {}, {
        ...config, headers: {
            Authorization: 'Bearer ' + userData.bearer,
        }
    });
    if (res.data?.status) {
        userStore.getState().logout();
    }
    return res.data as ApiResult;
}


