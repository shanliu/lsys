import { authApi } from "@shared/lib/apis/api_auth";
import { cleanEmptyStringParams, parseResData } from "@shared/lib/apis/utils";
import { ApiResult } from "@shared/types/apis-rest";
import { BoolParamSchema, PageParam, PageRes, UnixTimestampSchema } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

/**
 * RBAC操作管理 API
 * 对应文档: docs/api/system/rbac/op/
 * 管理系统中的资源操作权限，如添加、编辑、删除、查看等操作
 */

// 操作列表参数
export const OpListParamSchema = z.object({
    /** 操作键值 */
    op_key: z.string().optional(),
    /** 操作名称 */
    op_name: z.string().optional(),
    res_type_count:BoolParamSchema.optional(),
    check_role_use:BoolParamSchema.optional(),
    ...PageParam,
});
export type OpListParamType = z.infer<typeof OpListParamSchema>;

// 操作数据结构
export const OpItemSchema = z.object({
    /** 操作ID */
    id: z.coerce.number(),
    /** 操作权限键值 */
    op_key: z.string(),
    /** 操作权限名称 */
    op_name: z.string(),
    /** 创建时间 */
    add_time: UnixTimestampSchema,
    /** 修改时间 */
    change_time: UnixTimestampSchema,
    /** 修改用户ID */
    change_user_id: z.coerce.number(),
    /** 关联的资源类型数量（可选，需要传入 res_type_count: true 参数） */
    res_type_count: z.coerce.number().optional(),
    /** 是否被角色使用（可选，需要传入 check_role_use: true 参数） */
    is_role_use: BoolParamSchema.optional(),
});
export type OpItemType = z.infer<typeof OpItemSchema>;

export const OpListResSchema = z.object({
    data: z.array(OpItemSchema),
    ...PageRes,
});
export type OpListResType = z.infer<typeof OpListResSchema>;

/**
 * 获取操作列表
 * @description 分页获取系统中定义的资源操作权限列表
 */
export async function opList(
    param: OpListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<OpListResType>> {
    const cleanedParam = cleanEmptyStringParams(param, ['op_key', 'op_name']);
    const { data } = await authApi().post("/api/system/rbac/op/list", cleanedParam, config);
    return parseResData(data, OpListResSchema);
}

// 添加操作参数
export const OpAddParamSchema = z.object({
    /** 操作权限键值 */
    op_key: z.string().min(1, "操作权限键值不能为空"),
    /** 操作权限名称 */
    op_name: z.string().min(1, "操作权限名称不能为空"),
});
export type OpAddParamType = z.infer<typeof OpAddParamSchema>;

export const OpAddResSchema = z.object({
    /** 新增操作权限ID */
    id: z.coerce.number(),
});
export type OpAddResType = z.infer<typeof OpAddResSchema>;

/**
 * 添加资源操作
 * @description 添加新的资源操作权限定义，如增删改查等操作
 */
export async function opAdd(
    param: OpAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<OpAddResType>> {
    const { data } = await authApi().post("/api/system/rbac/op/add", param, config);
    return parseResData(data, OpAddResSchema);
}

// 编辑操作参数
export const OpEditParamSchema = z.object({
    /** 操作权限ID */
    op_id: z.coerce.number().min(1, "操作权限ID必须大于0"),
    /** 操作权限键值 */
    op_key: z.string().min(1, "操作权限键值不能为空"),
    /** 操作权限名称 */
    op_name: z.string().min(1, "操作权限名称不能为空"),
});
export type OpEditParamType = z.infer<typeof OpEditParamSchema>;

export const OpEditResSchema = z.object({
    num: z.coerce.number(),
});
export type OpEditResType = z.infer<typeof OpEditResSchema>;

/**
 * 编辑资源操作
 * @description 修改现有的资源操作权限定义
 */
export async function opEdit(
    param: OpEditParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<OpEditResType>> {
    const { data } = await authApi().post("/api/system/rbac/op/edit", param, config);
    return parseResData(data, OpEditResSchema);
}

// 删除操作参数
export const OpDeleteParamSchema = z.object({
    /** 操作权限ID */
    op_id: z.coerce.number().min(1, "操作权限ID必须大于0"),
});
export type OpDeleteParamType = z.infer<typeof OpDeleteParamSchema>;

export const OpDeleteResSchema = z.object({
    num: z.coerce.number(),
});
export type OpDeleteResType = z.infer<typeof OpDeleteResSchema>;

/**
 * 删除资源操作
 * @description 删除指定的资源操作权限定义
 */
export async function opDelete(
    param: OpDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<OpDeleteResType>> {
    const { data } = await authApi().post("/api/system/rbac/op/delete", param, config);
    return parseResData(data, OpDeleteResSchema);
}


