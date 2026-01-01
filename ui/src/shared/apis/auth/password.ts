import { baseApi } from "@shared/lib/apis/api_base";
import { ApiResult } from "@shared/types/apis-rest";
import { CaptChaParam, MobileSchema } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

// Password Reset APIs
export const PasswordEmailParamSchema = z.object({
    email: z.string().email("邮箱格式不正确"),
    code: z.string().min(1, "验证码不能为空"),
    new_password: z.string().min(6, "密码长度至少6位"),
});
export type PasswordEmailParamType = z.infer<typeof PasswordEmailParamSchema>;

export async function passwordEmail(
    param: PasswordEmailParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await baseApi().post('/api/auth/password/email', param, config);
    return data;
}

// Password Email Code - 发送邮箱验证码
export const PasswordEmailCodeParamSchema = z.object({
    email: z.string().email("邮箱格式不正确"),
    ...CaptChaParam,
});
export type PasswordEmailCodeParamType = z.infer<typeof PasswordEmailCodeParamSchema>;

export async function passwordEmailCode(
    param: PasswordEmailCodeParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await baseApi().post('/api/auth/password/email_code', param, config);
    return data;
}

// Password Mobile - 手机验证码重置密码
export const PasswordMobileParamSchema = z.object({
    mobile: MobileSchema,
    area_code: z.string().min(1, "国际区号不能为空"),
    code: z.string().min(1, "验证码不能为空"),
    new_password: z.string().min(6, "密码长度至少6位"),
});
export type PasswordMobileParamType = z.infer<typeof PasswordMobileParamSchema>;

export async function passwordMobile(
    param: PasswordMobileParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await baseApi().post('/api/auth/password/mobile', param, config);
    return data;
}

// Password Mobile Code - 发送手机验证码
export const PasswordMobileCodeParamSchema = z.object({
    mobile: MobileSchema,
    area_code: z.string().min(1, "国家区号不能为空"),
    ...CaptChaParam,
});
export type PasswordMobileCodeParamType = z.infer<typeof PasswordMobileCodeParamSchema>;

export async function passwordMobileCode(
    param: PasswordMobileCodeParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await baseApi().post('/api/auth/password/mobile_code', param, config);
    return data;
}

