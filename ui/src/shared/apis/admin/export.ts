
import { authApi } from "@shared/lib/apis/api_auth";
import { parseResData } from "@shared/lib/apis/utils";
import { DictListSchema } from "@shared/types/apis-dict";
import { ApiResult } from "@shared/types/apis-rest";
import { UnixTimestampSchema } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

// ==================== 系统级（管理端）批量导出 ====================
// 接口：POST /api/system/file/export_task/submit
//       POST /api/system/file/export_task/list
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
    const { data } = await authApi().post('/api/system/file/export_task/submit', param, config);
    return parseResData(data, AdminExportSubmitResSchema);
};

export const AdminExportListParamSchema = z.object({
    export_type: z.string().optional(),
    status: z.coerce.number().optional(),
    page: z.object({
        page: z.coerce.number().min(1).optional(),
        limit: z.coerce.number().min(1).max(50).optional(),
    }).optional(),
    count_num: z.coerce.boolean().optional(),
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

export const AdminExportTaskModelSchema = z.object({
    id: z.coerce.number(),
    app_id: z.coerce.number(),
    export_type: z.string(),
    export_params: z.string(),
    status: z.coerce.number(),
    error_message: z.string().optional(),
    add_time: UnixTimestampSchema,
    change_time: UnixTimestampSchema.nullable().optional(),
});

// 服务端 ExportTaskItem = { task: ExportTaskModel, file: ExportTaskFileItem | null }
export const AdminExportTaskSchema = z.object({
    task: AdminExportTaskModelSchema,
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
    const { data } = await authApi().post('/api/system/file/export_task/list', param, config);
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
    const { data } = await authApi().post('/api/system/file/export_task/active_count', param, config);
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
    const { data } = await authApi().post('/api/system/file/export_task/mapping', {}, config);
    return parseResData(data, AdminExportMappingResSchema);
};

// ==================== 导出任务文件下载 ====================

// ==================== 系统导出类型常量 ====================
export const EXPORT_TYPE_SYSTEM_APP_LIST = 'system_app_list';
export const EXPORT_TYPE_SYSTEM_SUB_APP_LIST = 'system_sub_app_list';
export const EXPORT_TYPE_SYSTEM_REQUEST_LIST = 'system_request_list';
export const EXPORT_TYPE_SYSTEM_MAILER_MESSAGE_LIST = 'system_mailer_message_list';
export const EXPORT_TYPE_SYSTEM_SMSER_MESSAGE_LIST = 'system_smser_message_list';
export const EXPORT_TYPE_SYSTEM_ADMIN_FILE_LIST = 'system_admin_file_list';
export const EXPORT_TYPE_SYSTEM_USER_CHANGE_LOG = 'system_user_change_log';
export const EXPORT_TYPE_SYSTEM_USER_ACCESS = 'system_user_access';
