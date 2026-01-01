import { baseApi } from "@shared/lib/apis/api_base";
import { parseResData } from "@shared/lib/apis/utils";
import { ApiResult } from "@shared/types/apis-rest";
import { AxiosRequestConfig } from "axios";
import z from "zod";
import { LoginResType } from "./login";

// WeChat External Login - Get QR Code URL
export const LoginExterUrlParamSchema = z.object({
    login_state: z.string().min(1, "登录状态不能为空"),
    login_callback: z.string().min(1, "回调地址不能为空"),
});
export type LoginExterUrlParamType = z.infer<typeof LoginExterUrlParamSchema>;

export const LoginExterUrlResSchema = z.object({
    login_url: z.string(),
    qr_code: z.string().optional(),
});
export type LoginExterUrlResType = z.infer<typeof LoginExterUrlResSchema>;

export async function loginExterUrl(
    method: string,
    param: LoginExterUrlParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<LoginExterUrlResType>> {
    const { data } = await baseApi().post(`/api/auth/exter_login_url/${method}`, param, config);
    return parseResData(data, LoginExterUrlResSchema);
}

// WeChat External Login - Check State
export const loginExterStateCheckParamSchema = z.object({
    login_state: z.string().min(1, "登录状态不能为空"),
});
export type loginExterStateCheckParamType = z.infer<typeof loginExterStateCheckParamSchema>;

export const loginExterStateCheckResSchema = z.object({
    status: z.string(), // 'waiting', 'scanned', 'confirmed', 'expired'
    user_info: z.object({
        nickname: z.string(),
        avatar: z.string(),
    }).nullable().optional(),
});
export type loginExterStateCheckResType = z.infer<typeof loginExterStateCheckResSchema>;

export async function loginExterStateCheck(
    method: string,
    param: loginExterStateCheckParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<loginExterStateCheckResType>> {
    const { data } = await baseApi().post(`/api/auth/exter_state_check/${method}`, param, config);
    return parseResData(data, loginExterStateCheckResSchema);
}

// WeChat External Login - Complete Callback
export const loginExterCallbackParamSchema = z.object({
    code: z.string().min(1, "微信授权码不能为空"),
    callback_state: z.string().min(1, "回调状态不能为空"),
});
export type loginExterCallbackParamType = z.infer<typeof loginExterCallbackParamSchema>;

export async function loginExterCallback(
    method: string,
    param: loginExterCallbackParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<LoginResType>> {
    const { data } = await baseApi().post(`/api/auth/exter_state_callback/${method}`, param, config);
    return data;
}
