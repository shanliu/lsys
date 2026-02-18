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

/**
 * 外部扩展能力管理 API
 */

// 外部扩展能力项数据结构
export const ExterFeatureItemSchema = z.object({
    /** 记录ID */
    id: z.coerce.number(),
    /** 扩展能力标识 */
    key: z.string(),
    /** 扩展能力名称/标题 */
    title: z.string(),
});
export type ExterFeatureItemType = z.infer<typeof ExterFeatureItemSchema>;

// 外部扩展能力列表响应
export const ExterFeatureListResSchema = z.object({
    /** 数据列表 */
    data: z.array(ExterFeatureItemSchema),
});
export type ExterFeatureListResType = z.infer<typeof ExterFeatureListResSchema>;

// 列表查询参数
export const ExterFeatureListParamSchema = z.object({
    /** 分页参数 */
    page: z.object({
        page: z.coerce.number(),
        limit: z.coerce.number(),
    }).optional(),
});
export type ExterFeatureListParamType = z.infer<typeof ExterFeatureListParamSchema>;

/**
 * 获取外部扩展能力列表
 * @description 获取系统中定义的外部扩展能力列表（来自数据库配置）
 */
export async function getExterFeatureList(
    param?: ExterFeatureListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<ExterFeatureListResType>> {
    const { data } = await authApi().post("/api/system/app/exter_feature_list", param || {}, config);
    return parseResData(data, ExterFeatureListResSchema);
}

// 新增外部扩展能力参数
export const ExterFeatureAddParamSchema = z.object({
    /** 扩展能力标识 */
    feature_key: z.string().min(1, "标识不能为空").regex(/^[a-zA-Z0-9_-]+$/, "标识只能包含数字、字母、下划线和横杠"),
    /** 扩展能力名称/标题 */
    title: z.string().min(1, "名称不能为空"),
});
export type ExterFeatureAddParamType = z.infer<typeof ExterFeatureAddParamSchema>;

// 新增响应
export const ExterFeatureAddResSchema = z.object({
    /** 新增记录ID */
    id: z.coerce.number(),
});
export type ExterFeatureAddResType = z.infer<typeof ExterFeatureAddResSchema>;

/**
 * 新增外部扩展能力
 * @description 添加一个新的外部扩展能力定义
 */
export async function addExterFeature(
    param: ExterFeatureAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<ExterFeatureAddResType>> {
    const { data } = await authApi().post("/api/system/app/exter_feature_add", param, config);
    return parseResData(data, ExterFeatureAddResSchema);
}

// 编辑外部扩展能力参数
export const ExterFeatureEditParamSchema = z.object({
    /** 记录ID */
    id: z.coerce.number(),
    /** 扩展能力标识 */
    feature_key: z.string().min(1, "标识不能为空").regex(/^[a-zA-Z0-9_-]+$/, "标识只能包含数字、字母、下划线和横杠"),
    /** 扩展能力名称/标题 */
    title: z.string().min(1, "名称不能为空"),
});
export type ExterFeatureEditParamType = z.infer<typeof ExterFeatureEditParamSchema>;

/**
 * 编辑外部扩展能力
 * @description 修改一个已存在的外部扩展能力定义
 */
export async function editExterFeature(
    param: ExterFeatureEditParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/app/exter_feature_edit", param, config);
    return data;
}

// 删除外部扩展能力参数
export const ExterFeatureDelParamSchema = z.object({
    /** 记录ID */
    id: z.coerce.number(),
});
export type ExterFeatureDelParamType = z.infer<typeof ExterFeatureDelParamSchema>;

/**
 * 删除外部扩展能力
 * @description 删除一个外部扩展能力定义
 */
export async function delExterFeature(
    param: ExterFeatureDelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/app/exter_feature_del", param, config);
    return data;
}


