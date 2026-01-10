import { authApi } from "@shared/lib/apis/api_auth";
import { cleanEmptyStringParams, parseResData } from "@shared/lib/apis/utils";
import { DictListSchema } from "@shared/types/apis-dict";
import { ApiResult } from "@shared/types/apis-rest";
import { BoolSchema, LimitParam, LimitResSchema, UnixTimestampSchema, UserDataResSchema } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

/**
 * RBAC基础功能 API
 * 对应文档: docs/api/system/rbac/base/
 * 提供RBAC系统的基础功能，包括映射信息、审核数据、权限检查等
 */

// RBAC基础映射数据
export const RbacBaseMappingResSchema = z.object({
    /** 授权结果字典 */
    audit_result: DictListSchema,
    /** 角色资源范围字典 */
    role_res_range: DictListSchema,
    /** 角色用户范围字典 */
    role_user_range: DictListSchema,
});
export type RbacBaseMappingResType = z.infer<typeof RbacBaseMappingResSchema>;

/**
 * 获取RBAC基础映射信息
 * @description 获取RBAC系统的基础字典数据，包括授权结果、角色范围等枚举值
 */
export async function rbacBaseMapping(
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<RbacBaseMappingResType>> {
    const { data } = await authApi().post("/api/system/rbac/base/mapping", {}, config);
    return parseResData(data, RbacBaseMappingResSchema);
}

// 审核数据参数
export const RbacAuditDataParamSchema = z.object({
    /** 用户ID */
    user_id: z.coerce.number().optional().nullable(),
    /** 资源键 */
    res_key: z.string().optional(),
    /** 资源操作 */
    res_op: z.string().optional(),
    /** 审计结果 */
    result: z.coerce.number().optional().nullable(),
    ...LimitParam,
});
export type RbacAuditDataParamType = z.infer<typeof RbacAuditDataParamSchema>;

// 审计详细数据结构
export const AuditDetailSchema = z.object({
    /** 详细记录ID */
    id: z.coerce.number().nullish().default(0),
    /** 添加时间 */
    add_time: UnixTimestampSchema.nullish().transform(val => val ? new Date(val) : new Date()).optional(),
    /** 审计结果: allow-授权通过, deny-授权拒绝 */
    check_result: z.string().nullish().default(""),
    /** 是否角色全部匹配 */
    is_role_all: z.string().nullish().default("0"),
    /** 是否角色排除 */
    is_role_excluce: z.string().nullish().default("0"),
    /** 是否角色包含 */
    is_role_include: z.string().nullish().default("0"),
    /** 操作ID */
    op_id: z.string().nullish().default("0"),
    /** 操作键 */
    op_key: z.string().nullish().default(""),
    /** 审计记录ID */
    rbac_audit_id: z.string().nullish().default(""),
    /** 资源授权状态 */
    res_auth: z.string().nullish().default(""),
    /** 资源数据 */
    res_data: z.string().nullish().default(""),
    /** 资源ID */
    res_id: z.string().nullish().default("0"),
    /** 资源类型 */
    res_type: z.string().nullish().default(""),
    /** 资源用户ID */
    res_user_id: z.string().nullish().default("0"),
    /** 角色数据 */
    role_data: z.string().nullish().default(""),
});
export type AuditDetailType = z.infer<typeof AuditDetailSchema>;

// 审计主记录
export const AuditSchema = z.object({
    /** 审计记录ID */
    id: z.coerce.number(),
    /** 用户ID */
    user_id: z.coerce.number(),
    /** 用户IP */
    user_ip: z.string(),
    /** 设备ID */
    device_id: z.string(),
    /** 设备名称 */
    device_name: z.string().nullish().default(""),
    /** 请求ID */
    request_id: z.string(),
    /** 审计结果: allow-授权通过, deny-授权拒绝 */
    check_result: z.string().nullish().default(""),
    /** 角色键数据 (JSON) */
    role_key_data: z.string().nullish().default(""),
    /** Token数据 */
    token_data: z.string().nullish().default(""),
    /** 用户应用ID */
    user_app_id: z.string().nullish().default(""),
    /** 添加时间 */
    add_time: UnixTimestampSchema,
});
export type AuditType = z.infer<typeof AuditSchema>;

// 审计数据结构
export const AuditDataSchema = z.object({
    /** 审计主记录 */
    audit: AuditSchema,
    /** 详细审计数据 */
    detail: z.array(AuditDetailSchema).default([]),
    user:UserDataResSchema.nullable().optional(),
});
export type AuditDataType = z.infer<typeof AuditDataSchema>;

export const RbacAuditDataResSchema = z.object({
    /** 审计数据列表 */
    data: z.array(AuditDataSchema),
    ...LimitResSchema,
});
export type RbacAuditDataResType = z.infer<typeof RbacAuditDataResSchema>;

/**
 * 获取RBAC审计数据
 * @description 获取RBAC权限系统的审计数据，包括审计记录和详细信息
 */
export async function rbacAuditData(
    param: RbacAuditDataParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<RbacAuditDataResType>> {
    const cleanedParam = cleanEmptyStringParams(param, ['res_key', 'res_op']);
    const { data } = await authApi().post("/api/system/rbac/base/audit_data", cleanedParam, config);
    return parseResData(data, RbacAuditDataResSchema as any);
}

// 检查资源用户权限参数
export const CheckResUserFromUserParamSchema = z.object({
    /** 访问用户ID */
    access_user_id: z.coerce.number().min(1, "访问用户ID必须大于0"),
    /** 应用请求ID */
    app_req_id: z.coerce.number().min(1, "应用请求ID必须大于0"),
    /** 确认状态 */
    confirm_status: z.coerce.number(),
    /** 确认备注 */
    confirm_note: z.string().optional(),
});
export type CheckResUserFromUserParamType = z.infer<typeof CheckResUserFromUserParamSchema>;

export const CheckResUserFromUserResSchema = z.object({
    /** 处理结果 */
    result: BoolSchema,
    /** 结果说明 */
    message: z.string().optional(),
});
export type CheckResUserFromUserResType = z.infer<typeof CheckResUserFromUserResSchema>;

/**
 * 检查资源用户权限
 * @description 检查用户对特定资源的访问权限并进行确认操作
 */
export async function checkResUserFromUser(
    param: CheckResUserFromUserParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<CheckResUserFromUserResType>> {
    const { data } = await authApi().post("/api/system/rbac/base/check_res_user_from_user", param, config);
    return parseResData(data, CheckResUserFromUserResSchema);
}


