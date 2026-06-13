import { authApi } from "@shared/lib/apis/api_auth";
import { parseResData } from "@shared/lib/apis/utils";
import { DictListSchema } from "@shared/types/apis-dict";
import { ApiResult } from "@shared/types/apis-rest";
import { BoolSchema, LimitParam, LimitResSchema, PageParam, PageResSchema, UnixTimestampSchema } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

// ==================== 字典映射 ====================

export const UserCollectorMappingResSchema = z.object({
    script_status: DictListSchema,
    record_status: DictListSchema,
    log_level: DictListSchema,
});
export type UserCollectorMappingResType = z.infer<typeof UserCollectorMappingResSchema>;

export const userCollectorMapping = async (
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserCollectorMappingResType>> => {
    const { data } = await authApi().post('/api/user/app_collector/mapping', {}, config);
    return parseResData(data, UserCollectorMappingResSchema);
};

// ==================== 脚本列表 ====================

export const CollectorScriptListParamSchema = z.object({
    app_id: z.coerce.number(),
    status: z.coerce.number().nullable().optional(),
    ...PageParam,
});
export type CollectorScriptListParamType = z.infer<typeof CollectorScriptListParamSchema>;

export const CollectorScriptItemSchema = z.object({
    id: z.coerce.number(),
    app_id: z.coerce.number(),
    name: z.string(),
    script_md5: z.string(),
    timeout_secs: z.coerce.number(),
    memory_limit: z.coerce.number(),
    status: z.coerce.number(),
    add_time: UnixTimestampSchema,
    change_time: UnixTimestampSchema,
});
export type CollectorScriptItemType = z.infer<typeof CollectorScriptItemSchema>;

export const CollectorScriptListResSchema = z.object({
    data: z.array(CollectorScriptItemSchema),
    ...PageResSchema,
});
export type CollectorScriptListResType = z.infer<typeof CollectorScriptListResSchema>;

export const userCollectorScriptList = async (
    param: CollectorScriptListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<CollectorScriptListResType>> => {
    const { data } = await authApi().post('/api/user/app_collector/scripts', param, config);
    return parseResData(data, CollectorScriptListResSchema);
};

// ==================== 添加脚本 ====================

export const CollectorScriptAddParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    name: z.string().min(1, "脚本名称不能为空"),
    script_code: z.string().min(1, "脚本代码不能为空"),
    timeout_secs: z.coerce.number().optional(),
    memory_limit: z.coerce.number().optional(),
});
export type CollectorScriptAddParamType = z.infer<typeof CollectorScriptAddParamSchema>;

export const CollectorScriptAddResSchema = z.object({
    id: z.coerce.number(),
});
export type CollectorScriptAddResType = z.infer<typeof CollectorScriptAddResSchema>;

export const userCollectorScriptAdd = async (
    param: CollectorScriptAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<CollectorScriptAddResType>> => {
    const { data } = await authApi().post('/api/user/app_collector/script_add', param, config);
    return parseResData(data, CollectorScriptAddResSchema);
};

// ==================== 编辑脚本 ====================

export const CollectorScriptEditParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    script_id: z.coerce.number().min(1, "脚本ID必须大于0"),
    name: z.string().min(1, "脚本名称不能为空"),
    script_code: z.string().min(1, "脚本代码不能为空"),
    timeout_secs: z.coerce.number().optional(),
    memory_limit: z.coerce.number().optional(),
});
export type CollectorScriptEditParamType = z.infer<typeof CollectorScriptEditParamSchema>;

export const userCollectorScriptEdit = async (
    param: CollectorScriptEditParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> => {
    const { data } = await authApi().post('/api/user/app_collector/script_edit', param, config);
    return data;
};

// ==================== 变更脚本状态 ====================

export const CollectorScriptStatusParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    script_id: z.coerce.number().min(1, "脚本ID必须大于0"),
    status: z.coerce.number(),
});
export type CollectorScriptStatusParamType = z.infer<typeof CollectorScriptStatusParamSchema>;

export const userCollectorScriptStatus = async (
    param: CollectorScriptStatusParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> => {
    const { data } = await authApi().post('/api/user/app_collector/script_status', param, config);
    return data;
};

// ==================== 删除脚本 ====================

export const CollectorScriptDeleteParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    script_id: z.coerce.number().min(1, "脚本ID必须大于0"),
});
export type CollectorScriptDeleteParamType = z.infer<typeof CollectorScriptDeleteParamSchema>;

export const userCollectorScriptDelete = async (
    param: CollectorScriptDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> => {
    const { data } = await authApi().post('/api/user/app_collector/script_del', param, config);
    return data;
};

// ==================== 脚本详情 ====================

export const CollectorScriptDetailParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    script_id: z.coerce.number().min(1, "脚本ID必须大于0"),
});
export type CollectorScriptDetailParamType = z.infer<typeof CollectorScriptDetailParamSchema>;

export const CollectorScriptDetailResSchema = z.object({
    id: z.coerce.number(),
    user_id: z.coerce.number().optional().nullable(),
    app_id: z.coerce.number(),
    name: z.string(),
    script_code: z.string(),
    script_md5: z.string(),
    timeout_secs: z.coerce.number(),
    memory_limit: z.coerce.number(),
    status: z.coerce.number(),
    add_time: UnixTimestampSchema,
    change_time: UnixTimestampSchema,
});
export type CollectorScriptDetailResType = z.infer<typeof CollectorScriptDetailResSchema>;

export const userCollectorScriptDetail = async (
    param: CollectorScriptDetailParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<CollectorScriptDetailResType>> => {
    const { data } = await authApi().post('/api/user/app_collector/script_detail', param, config);
    return parseResData(data, CollectorScriptDetailResSchema);
};

// ==================== 脚本文件列表 ====================

export const CollectorFileListParamSchema = z.object({
    app_id: z.coerce.number(),
    script_id: z.coerce.number(),
    ...LimitParam,
});
export type CollectorFileListParamType = z.infer<typeof CollectorFileListParamSchema>;

export const CollectorFileTagSchema = z.object({
    tag_name: z.string(),
    add_time: UnixTimestampSchema,
});

export const CollectorFileItemSchema = z.object({
    file_id: z.coerce.number(),
    file_name: z.string(),
    file_md5: z.string(),
    file_size: z.coerce.number(),
    storage_type: z.string(),
    content_type: z.string(),
    file_key: z.string(),
    tags: z.array(z.any()).optional(),
    add_time: UnixTimestampSchema,
});
export type CollectorFileItemType = z.infer<typeof CollectorFileItemSchema>;

export const CollectorFileListResSchema = z.object({
    data: z.array(CollectorFileItemSchema),
    ...LimitResSchema,
});
export type CollectorFileListResType = z.infer<typeof CollectorFileListResSchema>;

export const userCollectorFileList = async (
    param: CollectorFileListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<CollectorFileListResType>> => {
    const { data } = await authApi().post('/api/user/app_collector/script_files', param, config);
    return parseResData(data, CollectorFileListResSchema);
};

// ==================== 采集记录列表 ====================

export const CollectorRecordListParamSchema = z.object({
    app_id: z.coerce.number(),
    script_id: z.coerce.number(),
    status: z.coerce.number().nullable().optional(),
    attr_file: BoolSchema.optional(),
    attr_file_local: BoolSchema.optional(),
    attr_file_oss: BoolSchema.optional(),
    attr_file_tag: BoolSchema.optional(),
    ...LimitParam,
});
export type CollectorRecordListParamType = z.infer<typeof CollectorRecordListParamSchema>;

export const CollectorRecordItemSchema = z.object({
    id: z.coerce.number(),
    request_id: z.string(),
    script_id: z.coerce.number(),
    user_id: z.coerce.number(),
    app_id: z.coerce.number(),
    task_id: z.coerce.number(),
    exec_params: z.string(),
    status: z.coerce.number(),
    elapsed_ms: z.coerce.number(),
    error_message: z.string(),
    add_time: UnixTimestampSchema,
    start_time: UnixTimestampSchema,
    finish_time: UnixTimestampSchema,
    file: CollectorFileItemSchema.nullable(),
    has_more_files: z.coerce.number(),
});
export type CollectorRecordItemType = z.infer<typeof CollectorRecordItemSchema>;

export const CollectorRecordListResSchema = z.object({
    data: z.array(CollectorRecordItemSchema),
    ...LimitResSchema,
});
export type CollectorRecordListResType = z.infer<typeof CollectorRecordListResSchema>;

export const userCollectorRecordList = async (
    param: CollectorRecordListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<CollectorRecordListResType>> => {
    const { data } = await authApi().post('/api/user/app_collector/script_records', param, config);
    return parseResData(data, CollectorRecordListResSchema);
};

// ==================== 采集日志列表 ====================

export const CollectorLogListParamSchema = z.object({
    script_id: z.coerce.number(),
    request_id: z.string().nullable().optional(),
    level: z.coerce.number().nullable().optional(),
    ...LimitParam,
});
export type CollectorLogListParamType = z.infer<typeof CollectorLogListParamSchema>;

export const CollectorLogItemSchema = z.object({
    id: z.coerce.number(),
    request_id: z.string(),
    script_id: z.coerce.number(),
    user_id: z.coerce.number(),
    app_id: z.coerce.number(),
    level: z.coerce.number(),
    message: z.string(),
    add_time: UnixTimestampSchema,
});
export type CollectorLogItemType = z.infer<typeof CollectorLogItemSchema>;

export const CollectorLogListResSchema = z.object({
    data: z.array(CollectorLogItemSchema),
    ...LimitResSchema,
});
export type CollectorLogListResType = z.infer<typeof CollectorLogListResSchema>;

export const userCollectorLogList = async (
    param: CollectorLogListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<CollectorLogListResType>> => {
    const { data } = await authApi().post('/api/user/app_collector/script_logs', param, config);
    return parseResData(data, CollectorLogListResSchema);
};

// ==================== 记录关联文件列表 ====================

export const CollectorRecordFileListParamSchema = z.object({
    app_id: z.coerce.number(),
    request_id: z.string().min(1, "request_id不能为空"),
    ...LimitParam,
});
export type CollectorRecordFileListParamType = z.infer<typeof CollectorRecordFileListParamSchema>;

export const CollectorRecordFileListResSchema = z.object({
    data: z.array(CollectorFileItemSchema),
    ...LimitResSchema,
});
export type CollectorRecordFileListResType = z.infer<typeof CollectorRecordFileListResSchema>;

export const userCollectorRecordFileList = async (
    param: CollectorRecordFileListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<CollectorRecordFileListResType>> => {
    const { data } = await authApi().post('/api/user/app_collector/record_files', param, config);
    return parseResData(data, CollectorRecordFileListResSchema);
};

// ==================== 记录关联日志列表 ====================

export const CollectorRecordLogListParamSchema = z.object({
    request_id: z.string().min(1, "request_id不能为空"),
    level: z.coerce.number().nullable().optional(),
    ...LimitParam,
});
export type CollectorRecordLogListParamType = z.infer<typeof CollectorRecordLogListParamSchema>;

export const CollectorRecordLogListResSchema = z.object({
    data: z.array(CollectorLogItemSchema),
    ...LimitResSchema,
});
export type CollectorRecordLogListResType = z.infer<typeof CollectorRecordLogListResSchema>;

export const userCollectorRecordLogList = async (
    param: CollectorRecordLogListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<CollectorRecordLogListResType>> => {
    const { data } = await authApi().post('/api/user/app_collector/record_logs', param, config);
    return parseResData(data, CollectorRecordLogListResSchema);
};

// ==================== 提交采集任务 ====================

export const CollectorSubmitTaskParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    script_id: z.coerce.number().min(1, "脚本ID必须大于0"),
    request_id: z.string().nullable().optional(),
    params: z.any().optional(),
});
export type CollectorSubmitTaskParamType = z.infer<typeof CollectorSubmitTaskParamSchema>;

export const CollectorSubmitTaskResSchema = z.object({
    request_id: z.string(),
    record_id: z.coerce.number(),
    task_id: z.coerce.number(),
    script_name: z.string(),
});
export type CollectorSubmitTaskResType = z.infer<typeof CollectorSubmitTaskResSchema>;

export const userCollectorSubmitTask = async (
    param: CollectorSubmitTaskParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<CollectorSubmitTaskResType>> => {
    const { data } = await authApi().post('/api/user/app_collector/submit_task', param, config);
    return parseResData(data, CollectorSubmitTaskResSchema);
};

