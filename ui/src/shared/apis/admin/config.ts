import { authApi } from "@shared/lib/apis/api_auth";
import { parseResData } from "@shared/lib/apis/utils";
import { ApiResult } from "@shared/types/apis-rest";
import { BoolSchema } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

/**
 * 系统配置管理 API
 * 对应文档: docs/api/system/config/
 * 包括站点配置、OAuth配置等系统配置管理功能
 */

/**
 * 站点配置管理 API
 */

// 站点配置数据结构
export const SiteConfigSchema = z.object({
    /** 禁用旧密码标志 */
    dis_old_password: z.string(),
    /** 站点提示信息 */
    site_tips: z.string(),
    /** 超时时间(秒) */
    timeout: z.coerce.number(),
});
export type SiteConfigType = z.infer<typeof SiteConfigSchema>;

export const SiteConfigGetResSchema = z.object({
    /** 站点配置信息 */
    config: SiteConfigSchema,
});
export type SiteConfigGetResType = z.infer<typeof SiteConfigGetResSchema>;

/**
 * 获取站点配置信息
 * @description 获取系统的站点配置，包含站点提示、密码策略等设置
 */
export async function getSiteConfig(
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SiteConfigGetResType>> {
    const { data } = await authApi().post("/api/system/config/site_config/get", {}, config);
    return parseResData(data, SiteConfigGetResSchema);
}

// 设置站点配置参数
export const SiteConfigSetParamSchema = z.object({
    /** 站点提示信息 */
    site_tips: z.string(),
    /** 密码超时时间(秒) */
    password_timeout: z.coerce.number().min(0, "密码超时时间不能为负数"),
    /** 是否禁用旧密码 */
    disable_old_password: BoolSchema,
});
export type SiteConfigSetParamType = z.infer<typeof SiteConfigSetParamSchema>;

/**
 * 设置站点配置
 * @description 设置系统的站点配置，包含站点提示、密码策略等设置
 */
export async function setSiteConfig(
    param: SiteConfigSetParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/config/site_config/set", param, config);
    return data;
}

/**
 * OAuth配置管理 API
 */

// 微信OAuth配置数据结构
export const WechatOAuthConfigSchema = z.object({
    /** 微信应用ID */
    app_id: z.string(),
    /** 微信应用密钥 */
    app_secret: z.string(),
});
export type WechatOAuthConfigType = z.infer<typeof WechatOAuthConfigSchema>;

export const WechatOAuthConfigGetResSchema = z.object({
    /** 微信OAuth配置信息 */
    config: WechatOAuthConfigSchema,
});
export type WechatOAuthConfigGetResType = z.infer<typeof WechatOAuthConfigGetResSchema>;

/**
 * 获取微信OAuth配置
 * @description 获取系统的微信OAuth配置信息，用于微信登录功能
 */
export async function getWechatOAuthConfig(
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<WechatOAuthConfigGetResType>> {
    const { data } = await authApi().post("/api/system/config/oauth_config/wechat/get", {}, config);
    return parseResData(data, WechatOAuthConfigGetResSchema);
}

// 设置微信OAuth配置参数
export const WechatOAuthConfigSetParamSchema = z.object({
    /** 微信应用ID */
    app_id: z.string().min(1, "微信应用ID不能为空").regex(/^[a-zA-Z0-9_-]+$/, "微信应用ID只能包含数字、字母、下划线和横杠"),
    /** 微信应用密钥 */
    app_secret: z.string().min(1, "微信应用密钥不能为空").regex(/^[a-zA-Z0-9_-]+$/, "微信应用密钥只能包含数字、字母、下划线和横杠"),
});
export type WechatOAuthConfigSetParamType = z.infer<typeof WechatOAuthConfigSetParamSchema>;

/**
 * 设置微信OAuth配置
 * @description 设置系统的微信OAuth配置信息，用于微信登录功能
 */
export async function setWechatOAuthConfig(
    param: WechatOAuthConfigSetParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/config/oauth_config/wechat/set", param, config);
    return data;
}




