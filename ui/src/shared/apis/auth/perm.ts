
import { authApi } from "@shared/lib/apis/api_auth";
import { parseResData } from "@shared/lib/apis/utils";
import { ApiResult } from "@shared/types/apis-rest";
import { AxiosRequestConfig } from "axios";
import z from "zod";

// 权限校验参数项
export const PermCheckParamItemSchema = z.object({
    name: z.string(),
    data: z.any().nullable().optional(),
});
export type PermCheckParamItemType = z.infer<typeof PermCheckParamItemSchema>;

// 权限校验参数类型
export const PermCheckParamSchema = z.array(PermCheckParamItemSchema);
export type PermCheckParamType = z.infer<typeof PermCheckParamSchema>;

// 权限校验响应类型 zod schema
// status: 0 = 失败不显示, 1 = 成功, 2 = 失败提示错误
export const PermCheckItemSchema = z.object({
    name: z.string(),
    data: z.any().nullable().optional(),
    status: z.coerce.number(),
    msg: z.string().nullable().optional(),
});
export type PermCheckItemType = z.infer<typeof PermCheckItemSchema>;

export const PermCheckRecordSchema = z.array(PermCheckItemSchema);
export type PermCheckRecordType = z.infer<typeof PermCheckRecordSchema>;

// 服务端返回的包含 record 层级的响应
export const PermCheckResSchema = z.object({
    record: PermCheckRecordSchema
});
/**
 * 校验权限
 * @param param { check_data: [{ name, data? }] }
 * @param config axios 配置
 * @returns 权限校验结果
 */

export async function checkPerm(
    check_data: PermCheckParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<PermCheckRecordType>> {
    const { data } = await authApi().post("/api/auth/perm/check", {
        check_data,
    }, config);

    // 使用 any 来避免类型推断问题，然后手动处理数据转换
    const result = await parseResData(data, PermCheckResSchema as any);

    // 如果解析成功，移除 record 层级，直接返回子元素
    if (result.status && result.response) {
        const responseWithRecord = result.response as { record: PermCheckRecordType };
        return {
            ...result,
            response: responseWithRecord.record
        };
    }

    return {
        ...result,
        response: []
    };
}

