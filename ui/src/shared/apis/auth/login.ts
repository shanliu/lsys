import { baseApi } from "@shared/lib/apis/api_base";
import { parseResData } from "@shared/lib/apis/utils";
import { handleLoginResponse } from '@shared/lib/auth';
import { DictItemSchema, DictListSchema } from '@shared/types/apis-dict';
import { ApiResult } from "@shared/types/apis-rest";
import { BoolSchema, CaptChaParam, MobileSchema, UnixTimestampSchema } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

// 登录类型字典项 Schema (继承基础字典项并添加 validity 字段)
export const LoginTypeDictItemSchema = DictItemSchema.extend({
    /** 有效期（秒） */
    validity: z.string(),
});

export type LoginTypeDictItemType = z.infer<typeof LoginTypeDictItemSchema>;

// 登录类型字典列表 Schema
export const LoginTypeDictListSchema = z.array(LoginTypeDictItemSchema);

export type LoginTypeDictListType = z.infer<typeof LoginTypeDictListSchema>;

// 应用映射数据
export const LoginMapResSchema = z.object({
    /** 登录类型字典 */
    login_type: LoginTypeDictListSchema,
    login_status: DictListSchema,
    exter_type:DictListSchema,
});
export type LoginMapResType = z.infer<typeof LoginMapResSchema>;

/**
 * 获取登录字典映射数据
 * @description 获取登录相关的字典数据，包括登录类型等枚举值
 */
export async function appMapping(
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<LoginMapResType>> {
    const { data } = await baseApi().post("/api/auth/login/mapping", {}, config);
    return parseResData(data, LoginMapResSchema);
}


// Login Auth Data Schema (登录成功时的用户数据)
export const LoginAuthDataSchema = z.object({
    account_id: z.coerce.number(),
    empty_password: BoolSchema,
    login_data: z.object({
        account_id: z.coerce.number(),
        change_time: UnixTimestampSchema,
        confirm_time: UnixTimestampSchema.optional(),
        id: z.coerce.number(),
        status: z.coerce.number(),
        username: z.string().optional(),
        email: z.string().optional(),
        mobile: z.string().optional(),
        area_code: z.string().optional(),
    }).optional(),
    app_data: z.object({
        client_id: z.string(),
        change_time: UnixTimestampSchema,
        app_name: z.string(),
        app_id: z.coerce.number(),
    }).optional(),
    login_time: UnixTimestampSchema,
    login_type: z.string(),
    time_out: z.coerce.number(),
    user_id: z.coerce.number(),
    user_nickname: z.string(),
});

// Login Response Schema (成功登录)
export const LoginResSchema = z.object({
    auth_data: LoginAuthDataSchema,
    jwt: z.string().optional(),
    passwrod_timeout: z.string().optional(),
});
export type LoginResType = z.infer<typeof LoginResSchema>;

// MFA 需要验证响应 Schema
export const LoginMfaRequiredSchema = z.object({
    mfa_token: z.string(),
});
export type LoginMfaRequiredType = z.infer<typeof LoginMfaRequiredSchema>;

// 统一登录响应类型：成功 或 需要 MFA
export type LoginResponseType =
    | { kind: 'success'; data: LoginResType }
    | { kind: 'mfa_required'; mfa_token: string };

/**
 * 解析登录响应，判断是成功还是需要 MFA
 */
export function parseLoginResponse(response: any): LoginResponseType {
    if (response?.mfa_token) {
        return { kind: 'mfa_required', mfa_token: response.mfa_token };
    }
    return { kind: 'success', data: response as LoginResType };
}

// 扩展 ApiResult 以支持 MFA 响应
export type LoginApiResult = ApiResult<LoginResType> & {
    /** 需要 MFA 验证时返回的令牌 */
    mfa_token?: string;
};

/**
 * 处理登录响应的统一逻辑
 * 返回 true 表示登录成功（已处理），false 表示需要 MFA（需要页面跳转）
 * 
 * 后端响应格式（需要MFA时）:
 * {
 *   "response": { "mfa_token": "..." },
 *   "result": { "code": "500", "message": "need-mfa-valid", "state": "mfa_need" }
 * }
 * 
 * 经过 axios 拦截器处理后变为 ApiResult 格式:
 * {
 *   "status": true,
 *   "code": "500", 
 *   "state": "mfa_need",
 *   "message": "need-mfa-valid",
 *   "response": { "mfa_token": "..." }
 * }
 */
export function handleLoginApiResponse(data: any): { success: boolean; mfa_token?: string } {
    // 检查是否需要 MFA 验证 (ApiResult 格式的 state 字段)
    if (data?.state === 'mfa_need' && data?.response?.mfa_token) {
        return { success: false, mfa_token: data.response.mfa_token };
    }
    
    if (data?.status && data?.response) {
        // 正常登录成功
        handleLoginResponse(data.response);
        return { success: true };
    }
    return { success: false };
}

// Login APIs
export const LoginNamePasswordParamSchema = z.object({
    name: z.string().min(1, "用户名不能为空"),
    password: z.string().min(1, "密码不能为空"),
    ...CaptChaParam,
});
export type LoginNamePasswordParamType = z.infer<typeof LoginNamePasswordParamSchema>;



export async function loginNamePassword(
    param: LoginNamePasswordParamType,
    config?: AxiosRequestConfig<any>
): Promise<LoginApiResult> {
    const res = await baseApi().post('/api/auth/login/name', param, config);
    const result = handleLoginApiResponse(res.data);
    if (!result.success && !result.mfa_token) {
        res.data.status = false;
    }
    if (result.mfa_token) {
        res.data.mfa_token = result.mfa_token;
    }
    return res.data;
}

// SMS Code Login
export const LoginSmsCodeParamSchema = z.object({
    mobile: MobileSchema,
    area_code: z.string().min(1, "区号不能为空"),
    code: z.string().min(1, "短信验证码不能为空"),
    ...CaptChaParam,
});
export type LoginSmsCodeParamType = z.infer<typeof LoginSmsCodeParamSchema>;

export async function loginSmsCode(
    param: LoginSmsCodeParamType,
    config?: AxiosRequestConfig<any>
): Promise<LoginApiResult> {
    const res = await baseApi().post('/api/auth/login/sms-code', param, config);
    const result = handleLoginApiResponse(res.data);
    if (!result.success && !result.mfa_token) {
        res.data.status = false;
    }
    if (result.mfa_token) {
        res.data.mfa_token = result.mfa_token;
    }
    return res.data;
}

// SMS Code Send
export const LoginSmsCodeSendParamSchema = z.object({
    mobile: MobileSchema,
    area_code: z.string().min(1, "国际区号不能为空"),
    ...CaptChaParam,
});
export type LoginSmsCodeSendParamType = z.infer<typeof LoginSmsCodeSendParamSchema>;

export const LoginSmsCodeSendResSchema = z.object({
    ttl: z.coerce.number(),
});
export type LoginSmsCodeSendResType = z.infer<typeof LoginSmsCodeSendResSchema>;

export async function loginSmsCodeSend(
    param: LoginSmsCodeSendParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<LoginSmsCodeSendResType>> {
    const { data } = await baseApi().post('/api/auth/login/sms-send-code', param, config);
    return parseResData(data, LoginSmsCodeSendResSchema);
}

// SMS Password Login
export const LoginSmsPasswordParamSchema = z.object({
    mobile: MobileSchema,
    area_code: z.string().min(1, "区号不能为空"),
    password: z.string().min(1, "密码不能为空"),
    ...CaptChaParam,
});
export type LoginSmsPasswordParamType = z.infer<typeof LoginSmsPasswordParamSchema>;

export async function loginSmsPassword(
    param: LoginSmsPasswordParamType,
    config?: AxiosRequestConfig<any>
): Promise<LoginApiResult> {
    const res = await baseApi().post('/api/auth/login/sms', param, config);
    const result = handleLoginApiResponse(res.data);
    if (!result.success && !result.mfa_token) {
        res.data.status = false;
    }
    if (result.mfa_token) {
        res.data.mfa_token = result.mfa_token;
    }
    return res.data;
}

// Email Code Login
export const LoginEmailCodeParamSchema = z.object({
    email: z.string().email("邮箱格式不正确"),
    code: z.string().min(1, "邮箱验证码不能为空"),
    ...CaptChaParam,
});
export type LoginEmailCodeParamType = z.infer<typeof LoginEmailCodeParamSchema>;

export async function loginEmailCode(
    param: LoginEmailCodeParamType,
    config?: AxiosRequestConfig<any>
): Promise<LoginApiResult> {
    const res = await baseApi().post('/api/auth/login/email-code', param, config);
    const result = handleLoginApiResponse(res.data);
    if (!result.success && !result.mfa_token) {
        res.data.status = false;
    }
    if (result.mfa_token) {
        res.data.mfa_token = result.mfa_token;
    }
    return res.data;
}

// Email Code Send
export const LoginEmailCodeSendParamSchema = z.object({
    email: z.string().email("邮箱格式不正确"),
    ...CaptChaParam,
});
export type LoginEmailCodeSendParamType = z.infer<typeof LoginEmailCodeSendParamSchema>;

export const LoginEmailCodeSendResSchema = z.object({
    ttl: z.coerce.number(),
});
export type LoginEmailCodeSendResType = z.infer<typeof LoginEmailCodeSendResSchema>;

export async function loginEmailCodeSend(
    param: LoginEmailCodeSendParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<LoginEmailCodeSendResType>> {
    const { data } = await baseApi().post('/api/auth/login/email-send-code', param, config);
    return parseResData(data, LoginEmailCodeSendResSchema);
}

// Email Password Login
export const LoginEmailPasswordParamSchema = z.object({
    email: z.string().email("邮箱格式不正确"),
    password: z.string().min(1, "密码不能为空"),
    ...CaptChaParam,
});
export type LoginEmailPasswordParamType = z.infer<typeof LoginEmailPasswordParamSchema>;

export async function loginEmailPassword(
    param: LoginEmailPasswordParamType,
    config?: AxiosRequestConfig<any>
): Promise<LoginApiResult> {
    const res = await baseApi().post('/api/auth/login/email', param, config);
    const result = handleLoginApiResponse(res.data);
    if (!result.success && !result.mfa_token) {
        res.data.status = false;
    }
    if (result.mfa_token) {
        res.data.mfa_token = result.mfa_token;
    }
    return res.data;
}

// App Code Login
export const LoginAppCodeParamSchema = z.object({
    client_id: z.string().min(1, "应用ID不能为空"),
    token_data: z.string().min(1, "登录令牌不能为空"),
    ...CaptChaParam,
});
export type LoginAppCodeParamType = z.infer<typeof LoginAppCodeParamSchema>;

export async function loginAppCode(
    param: LoginAppCodeParamType,
    config?: AxiosRequestConfig<any>
): Promise<LoginApiResult> {
    const res = await baseApi().post('/api/auth/login/app-code', param, config);
    const result = handleLoginApiResponse(res.data);
    if (!result.success && !result.mfa_token) {
        res.data.status = false;
    }
    if (result.mfa_token) {
        res.data.mfa_token = result.mfa_token;
    }
    return res.data;
}

// MFA TOTP Verify
export const MfaVerifyParamSchema = z.object({
    mfa_token: z.string().min(1, "MFA令牌不能为空"),
    code: z.string().length(6, "验证码必须为6位数字"),
});
export type MfaVerifyParamType = z.infer<typeof MfaVerifyParamSchema>;

/**
 * MFA TOTP 验证
 * @description 使用 TOTP 验证码完成多因素认证登录
 */
export async function mfaVerify(
    param: MfaVerifyParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<LoginResType>> {
    const res = await baseApi().post('/api/auth/login/mfa-verify', param, config);
    if (res.data?.status && res.data?.response) {
        handleLoginResponse(res.data.response);
    } else {
        res.data.status = false;
    }
    return res.data;
}

