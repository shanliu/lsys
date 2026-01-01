import { baseApi } from "@shared/lib/apis/api_base";
import { parseResData } from "@shared/lib/apis/utils";
import { ApiResult } from "@shared/types/apis-rest";
import { CaptChaParam, MobileSchema } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

// Register APIs
export const RegisterEmailParamSchema = z.object({
    nikename: z.string().min(1, "用户昵称不能为空"),
    email: z.string().email("邮箱格式不正确"),
    code: z.string().min(1, "验证码不能为空"),
    password: z.string().min(6, "密码长度至少6位"),
});
export type RegisterEmailParamType = z.infer<typeof RegisterEmailParamSchema>;

export const RegisterResSchema = z.object({
    id: z.coerce.number(),
});
export type RegisterResType = z.infer<typeof RegisterResSchema>;

export async function registerEmail(
    param: RegisterEmailParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<RegisterResType>> {
    const { data } = await baseApi().post('/api/auth/register/email', param, config);
    return parseResData(data, RegisterResSchema);
}

// Register Email Code - 添加验证码支持
export const RegisterEmailCodeParamSchema = z.object({
    email: z.string().email("邮箱格式不正确"),
    ...CaptChaParam,
});
export type RegisterEmailCodeParamType = z.infer<typeof RegisterEmailCodeParamSchema>;

export const RegisterCodeResSchema = z.object({
    ttl: z.coerce.number().optional(),
});
export type RegisterCodeResType = z.infer<typeof RegisterCodeResSchema>;

export async function registerEmailCode(
    param: RegisterEmailCodeParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<RegisterCodeResType>> {
    const { data } = await baseApi().post('/api/auth/register/email-code', param, config);
    return parseResData(data, RegisterCodeResSchema);
}

// Register SMS
export const RegisterSmsParamSchema = z.object({
    nikename: z.string().min(1, "用户昵称不能为空"),
    mobile: MobileSchema,
    area_code: z.string().min(1, "国际区号不能为空"),
    code: z.string().min(1, "验证码不能为空"),
    password: z.string().min(6, "密码长度至少6位"),
});
export type RegisterSmsParamType = z.infer<typeof RegisterSmsParamSchema>;

export async function registerSms(
    param: RegisterSmsParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<RegisterResType>> {
    const { data } = await baseApi().post('/api/auth/register/sms', param, config);
    return parseResData(data, RegisterResSchema);
}

// Register SMS Code - 添加验证码支持
export const RegisterSmsCodeParamSchema = z.object({
    mobile: MobileSchema,
    area_code: z.string().min(1, "国家区号不能为空"),
    ...CaptChaParam,
});
export type RegisterSmsCodeParamType = z.infer<typeof RegisterSmsCodeParamSchema>;

export async function registerSmsCode(
    param: RegisterSmsCodeParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<RegisterCodeResType>> {
    const { data } = await baseApi().post('/api/auth/register/sms-code', param, config);
    return parseResData(data, RegisterCodeResSchema);
}

// Register by Name - 根据文档修正字段名
export const RegisterNameParamSchema = z.object({
    nikename: z.string().min(1, "昵称不能为空"), // 注意字段名是nikename不是nickname
    name: z.string().min(1, "用户名不能为空"),
    password: z.string().min(6, "密码长度至少6位"),
});
export type RegisterNameParamType = z.infer<typeof RegisterNameParamSchema>;

export async function registerName(
    param: RegisterNameParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<RegisterResType>> {
    const { data } = await baseApi().post('/api/auth/register/name', param, config);
    return parseResData(data, RegisterResSchema);
}

