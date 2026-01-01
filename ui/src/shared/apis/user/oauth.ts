import { authApi } from "@shared/lib/apis/api_auth";
import { parseResData } from "@shared/lib/apis/utils";
import { ApiResult } from "@shared/types/apis-rest";
import { AxiosRequestConfig } from "axios";
import z from "zod";

// OAuth Authorization Do API - 完成OAuth登录授权
export const OauthDoParamSchema = z.object({
    client_id: z.string().min(1, "客户端ID不能为空"),
    scope: z.string().min(1, "授权范围不能为空"),
    redirect_uri: z.string().min(1, "授权回调地址不能为空"),
});
export type OauthDoParamType = z.infer<typeof OauthDoParamSchema>;

export const OauthDoResSchema = z.object({
    code: z.string(),
});
export type OauthDoResType = z.infer<typeof OauthDoResSchema>;

export async function oauthDo(
    param: OauthDoParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<OauthDoResType>> {
    const { data } = await authApi().post('/api/oauth/do', param, config);
    return parseResData(data, OauthDoResSchema);
}

// OAuth Scope API - 获取授权Scope信息
export const OauthScopeParamSchema = z.object({
    client_id: z.string().min(1, "应用ID不能为空"),
    scope: z.string().min(1, "授权范围key不能为空"),
});
export type OauthScopeParamType = z.infer<typeof OauthScopeParamSchema>;

export const OauthScopeItemSchema = z.object({
    desc: z.string(),
    key: z.string(),
    name: z.string(),
});
export type OauthScopeItemType = z.infer<typeof OauthScopeItemSchema>;

export const OauthScopeResSchema = z.object({
    scope: z.array(OauthScopeItemSchema),
});
export type OauthScopeResType = z.infer<typeof OauthScopeResSchema>;

export async function oauthScope(
    param: OauthScopeParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<OauthScopeResType>> {
    const { data } = await authApi().post('/api/oauth/scope', param, config);
    return parseResData(data, OauthScopeResSchema);
}



