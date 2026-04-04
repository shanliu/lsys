
import { authApi } from "@shared/lib/apis/api_auth";
import { parseResData } from "@shared/lib/apis/utils";
import { DictListSchema } from "@shared/types/apis-dict";
import { ApiResult } from "@shared/types/apis-rest";
import { UnixTimestampSchema } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

// ==================== 系统级（管理端）批量导出 ====================
// 接口：POST /api/system/file/export_submit
//       POST /api/system/file/export_list
// app_id 固定为 0（系统级任务，由服务端自动填充）

export const AdminExportSubmitParamSchema = z.object({
    export_type: z.string(),
    params: z.record(z.unknown()).optional(),
});
export type AdminExportSubmitParamType = z.infer<typeof AdminExportSubmitParamSchema>;

export const AdminExportSubmitResSchema = z.object({
    id: z.coerce.number(),
});
export type AdminExportSubmitResType = z.infer<typeof AdminExportSubmitResSchema>;

export const adminExportSubmit = async (
    param: AdminExportSubmitParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AdminExportSubmitResType>> => {
    const { data } = await authApi().post('/api/system/file/export_submit', param, config);
    return parseResData(data, AdminExportSubmitResSchema);
};

export const AdminExportListParamSchema = z.object({
    export_type: z.string().optional(),
    status: z.coerce.number().optional(),
    page: z.object({
        page: z.coerce.number().min(1).optional(),
        limit: z.coerce.number().min(1).max(50).optional(),
    }).optional(),
    count_num: z.boolean().optional(),
});
export type AdminExportListParamType = z.infer<typeof AdminExportListParamSchema>;

export const AdminExportTaskFileSchema = z.object({
    file_id: z.coerce.number(),
    file_name: z.string(),
    file_size: z.coerce.number(),
    content_type: z.string(),
    file_url: z.string().nullable().optional(),
});
export type AdminExportTaskFileType = z.infer<typeof AdminExportTaskFileSchema>;

export const AdminExportTaskSchema = z.object({
    id: z.coerce.number(),
    app_id: z.coerce.number(),
    export_type: z.string(),
    export_params: z.string(),
    status: z.coerce.number(),
    error_message: z.string().optional(),
    add_time: UnixTimestampSchema,
    change_time: UnixTimestampSchema.nullable().optional(),
    file: AdminExportTaskFileSchema.nullable().optional(),
});
export type AdminExportTaskType = z.infer<typeof AdminExportTaskSchema>;

export const AdminExportListResSchema = z.object({
    data: z.array(AdminExportTaskSchema),
    total: z.coerce.number().nullable().optional(),
});
export type AdminExportListResType = z.infer<typeof AdminExportListResSchema>;

export const adminExportList = async (
    param: AdminExportListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AdminExportListResType>> => {
    const { data } = await authApi().post('/api/system/file/export_list', param, config);
    return parseResData(data, AdminExportListResSchema);
};

// ==================== 活跃导出任务计数 ====================

export const AdminExportActiveCountParamSchema = z.object({
    export_type: z.string().optional(),
});
export type AdminExportActiveCountParamType = z.infer<typeof AdminExportActiveCountParamSchema>;

export const AdminExportActiveCountResSchema = z.object({
    count: z.coerce.number(),
});
export type AdminExportActiveCountResType = z.infer<typeof AdminExportActiveCountResSchema>;

export const adminExportActiveCount = async (
    param: AdminExportActiveCountParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AdminExportActiveCountResType>> => {
    const { data } = await authApi().post('/api/system/file/export_active_count', param, config);
    return parseResData(data, AdminExportActiveCountResSchema);
};

// ==================== 导出任务字典映射 ====================

export const AdminExportMappingResSchema = z.object({
    export_task_status: DictListSchema,
});
export type AdminExportMappingResType = z.infer<typeof AdminExportMappingResSchema>;

export const adminExportTaskMapping = async (
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AdminExportMappingResType>> => {
    const { data } = await authApi().post('/api/system/file/export_task_mapping', {}, config);
    return parseResData(data, AdminExportMappingResSchema);
};

// ==================== 系统导出类型常量 ====================
export const EXPORT_TYPE_SYSTEM_APP_LIST = 'system_app_list';
export const EXPORT_TYPE_SYSTEM_SUB_APP_LIST = 'system_sub_app_list';
export const EXPORT_TYPE_SYSTEM_REQUEST_LIST = 'system_request_list';
export const EXPORT_TYPE_SYSTEM_ROLE_USER_AVAILABLE = 'system_role_user_available';
export const EXPORT_TYPE_SYSTEM_RBAC_AUDIT = 'system_rbac_audit';
export const EXPORT_TYPE_SYSTEM_RBAC_ROLE_PERM = 'system_rbac_role_perm';
export const EXPORT_TYPE_SYSTEM_RBAC_RES = 'system_rbac_res';
export const EXPORT_TYPE_SYSTEM_RBAC_ROLE_USER = 'system_rbac_role_user';
export const EXPORT_TYPE_SYSTEM_LOGIN_HISTORY = 'system_login_history';
export const EXPORT_TYPE_SYSTEM_RBAC_RES_TYPE = 'system_rbac_res_type';
export const EXPORT_TYPE_SYSTEM_RBAC_RES_TYPE_OP = 'system_rbac_res_type_op';
export const EXPORT_TYPE_SYSTEM_ACCOUNT_SEARCH = 'system_account_search';
export const EXPORT_TYPE_SYSTEM_RBAC_OP = 'system_rbac_op';
export const EXPORT_TYPE_SYSTEM_CHANGE_LOG = 'system_change_log';
export const EXPORT_TYPE_SYSTEM_RBAC_ROLE = 'system_rbac_role';
