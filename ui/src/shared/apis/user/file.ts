import { authApi } from "@shared/lib/apis/api_auth";
import { cleanEmptyStringParams, parseResData } from "@shared/lib/apis/utils";
import { DictItemSchema, DictListSchema } from "@shared/types/apis-dict";
import { ApiResult } from "@shared/types/apis-rest";
import { BoolSchema, LimitParam, LimitResSchema, PageParam, PageResSchema, UnixTimestampSchema, UserDataResSchema } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

// ==================== 字典映射 ====================

export const UserFileMappingParamSchema = z.object({});
export type UserFileMappingParamType = z.infer<typeof UserFileMappingParamSchema>;

export const StorageTypeDictItemSchema = DictItemSchema.extend({
    type: z.enum(['local', 'oss']),
    provider_type: z.string().optional(),
    is_private: BoolSchema.optional(),
});
export type StorageTypeDictItemType = z.infer<typeof StorageTypeDictItemSchema>;

export const UserFileMappingResSchema = z.object({
    upload_chunk_max: z.coerce.number().default(5 * 1024 * 1024),
    max_upload_size: z.coerce.number().default(0),
    storage_type: z.array(StorageTypeDictItemSchema),
    file_source_type: DictListSchema,
    file_status: DictListSchema,
    file_chunk_status: DictListSchema,
    file_ref_status: DictListSchema,
    lineage_rel_type: DictListSchema,
});
export type UserFileMappingResType = z.infer<typeof UserFileMappingResSchema>;

export const userFileMapping = async (
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileMappingResType>> => {
    const { data } = await authApi().post('/api/user/app_file/mapping', {}, config);
    return parseResData(data, UserFileMappingResSchema);
};

// ==================== 文件列表 ====================

export const UserFileListParamSchema = z.object({
    app_id: z.coerce.number(),
    user_id: z.coerce.number().nullable().optional(),
    status: z.coerce.number().nullable().optional(),
    url: z.string().nullable().optional(),
    source_url: z.string().nullable().optional(),
    add_time_start: z.coerce.number().nullable().optional(),
    add_time_end: z.coerce.number().nullable().optional(),
    storage_type: z.string().nullable().optional(),
    file_md5: z.string().nullable().optional(),
    tag_names: z.array(z.string()).nullable().optional(),
    attr_tag: z.coerce.boolean().nullable().optional(),
    attr_lineage: z.coerce.boolean().nullable().optional(),
    ...LimitParam,
});
export type UserFileListParamType = z.infer<typeof UserFileListParamSchema>;

// 本地存储属性
export const FileAttrLocalSchema = z.object({
    id: z.coerce.number(),
    source_type: z.string(),
    local_path: z.string(),
    file_chunk_total: z.coerce.number(),
    file_chunk_succ: z.coerce.number(),
    file_chunk_size: z.coerce.number(),
}).nullable().optional();
export type FileAttrLocalType = z.infer<typeof FileAttrLocalSchema>;

// OSS 存储属性
export const FileAttrOssSchema = z.object({
    id: z.coerce.number(),
    object_key: z.string(),
    object_url: z.string(),
    bucket: z.string(),
    region: z.string(),
    oss_size: z.coerce.number(),
}).nullable().optional();
export type FileAttrOssType = z.infer<typeof FileAttrOssSchema>;

export const FileTagSchema = z.object({
    tag_name: z.string(),
    add_time: UnixTimestampSchema,
});
export type FileTagType = z.infer<typeof FileTagSchema>;

export const FileLineageCountSchema = z.object({
    rel_type: z.coerce.number(),
    storage_type: z.string(),
    count: z.coerce.number(),
});
export type FileLineageCountType = z.infer<typeof FileLineageCountSchema>;

export const UserFileItemSchema = z.object({
    id: z.coerce.number(),
    file_id: z.coerce.number(),
    file_key: z.string().nullable(),
    file_name: z.string(),
    file_md5: z.string().nullable(),
    file_size: z.coerce.number(),
    storage_type: z.string(),
    status: z.coerce.number(),
    content_type: z.string().nullable(),
    source_url: z.string().nullable(),
    add_time: UnixTimestampSchema,
    expire_time: UnixTimestampSchema.nullable().optional(),
    user_id: z.coerce.number(),
    tags: z.array(FileTagSchema).nullable().optional(),
    tag_count: z.coerce.number().nullable().optional(),
    first_tag: FileTagSchema.nullable().optional(),
    local_id: z.coerce.number().nullable().optional(),
    source_type: z.string().nullable().optional(),
    local_path: z.string().nullable().optional(),
    file_chunk_total: z.coerce.number().nullable().optional(),
    file_chunk_succ: z.coerce.number().nullable().optional(),
    file_chunk_size: z.coerce.number().nullable().optional(),
    oss_id: z.coerce.number().nullable().optional(),
    object_key: z.string().nullable().optional(),
    object_url: z.string().nullable().optional(),
    bucket: z.string().nullable().optional(),
    region: z.string().nullable().optional(),
    oss_size: z.coerce.number().nullable().optional(),
    lineage_counts: z.array(FileLineageCountSchema).nullable().optional(),
    is_downloading:     BoolSchema.optional(),
});
export type UserFileItemType = z.infer<typeof UserFileItemSchema>;

export const UserFileListResSchema = z.object({
    data: z.array(UserFileItemSchema),
    ...LimitResSchema,
});
export type UserFileListResType = z.infer<typeof UserFileListResSchema>;

export const userFileList = async (
    param: UserFileListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileListResType>> => {
    const cleanedParam = cleanEmptyStringParams(param, ['url', 'source_url', 'storage_type', 'file_md5']);
    const { data } = await authApi().post('/api/user/app_file/list', cleanedParam, config);
    return parseResData(data, UserFileListResSchema);
};

// ==================== 标签名列表 ====================

export const UserFileTagNamesParamSchema = z.object({
    app_id: z.coerce.number(),
    tag_name_prefix: z.string().nullable().optional(),
    limit: z.coerce.number().nullable().optional(),
});
export type UserFileTagNamesParamType = z.infer<typeof UserFileTagNamesParamSchema>;

export const UserFileTagNamesResSchema = z.object({
    data: z.array(z.string()),
});
export type UserFileTagNamesResType = z.infer<typeof UserFileTagNamesResSchema>;

export const userFileTagNames = async (
    param: UserFileTagNamesParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileTagNamesResType>> => {
    const { data } = await authApi().post('/api/user/app_file/tag_names', param, config);
    return parseResData(data, UserFileTagNamesResSchema);
};

// ==================== 文件删除 ====================

export const UserFileDeleteParamSchema = z.object({
    file_ref_id: z.coerce.number(),
});
export type UserFileDeleteParamType = z.infer<typeof UserFileDeleteParamSchema>;

export const userFileDelete = async (
    param: UserFileDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<any>> => {
    const { data } = await authApi().post('/api/user/app_file/delete', param, config);
    return data;
};

// ==================== MD5 秒传 ====================

export const UserFileUploadByMd5ParamSchema = z.object({
    app_id: z.coerce.number(),
    file_md5: z.string(),
    file_name: z.string(),
});
export type UserFileUploadByMd5ParamType = z.infer<typeof UserFileUploadByMd5ParamSchema>;

export const UserFileUploadByMd5ResSchema = z.object({
    matched: BoolSchema,
    id: z.coerce.number().optional(),
});
export type UserFileUploadByMd5ResType = z.infer<typeof UserFileUploadByMd5ResSchema>;

export const userFileUploadByMd5 = async (
    param: UserFileUploadByMd5ParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileUploadByMd5ResType>> => {
    const { data } = await authApi().post('/api/user/app_file/upload_by_md5', param, config);
    return parseResData(data, UserFileUploadByMd5ResSchema);
};

// ==================== 创建上传任务 ====================

export const UserFileUploadChunkSchema = z.object({
    offset: z.coerce.number(),
    len: z.coerce.number(),
    md5: z.string().optional(),
});
export type UserFileUploadChunkType = z.infer<typeof UserFileUploadChunkSchema>;

export const UserFileUploadCreateParamSchema = z.object({
    app_id: z.coerce.number(),
    file_name: z.string(),
    chunks: z.array(UserFileUploadChunkSchema),
});
export type UserFileUploadCreateParamType = z.infer<typeof UserFileUploadCreateParamSchema>;

export const UserFileUploadCreateResSchema = z.object({
    id: z.coerce.number(),
    file_id: z.coerce.number(),
    file_name: z.string(),
    status: z.coerce.number(),
});
export type UserFileUploadCreateResType = z.infer<typeof UserFileUploadCreateResSchema>;

export const userFileUploadCreate = async (
    param: UserFileUploadCreateParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileUploadCreateResType>> => {
    const { data } = await authApi().post('/api/user/app_file/upload_create', param, config);
    return parseResData(data, UserFileUploadCreateResSchema);
};

// ==================== 上传分片数据 ====================

/**
 * 上传分片数据（multipart/form-data）
 * @param fileUserId 文件用户关联ID（由 upload_create 返回）
 * @param chunkIndex 分片索引
 * @param file 文件 Blob
 * @param config Axios 配置（支持 onUploadProgress、signal 等）
 */
export const userFileUploadData = async (
    fileUserId: number,
    chunkIndex: number,
    file: Blob,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<any>> => {
    const formData = new FormData();
    formData.append('id', String(fileUserId));
    formData.append('chunk_index', String(chunkIndex));
    formData.append('file', file);

    const { data } = await authApi().post('/api/user/app_file/upload_data', formData, {
        ...config,
    });
    return data;
};

// ==================== URL 下载创建 ====================

export const UserFileFromUrlParamSchema = z.object({
    app_id: z.coerce.number(),
    source_url: z.string(),
});
export type UserFileFromUrlParamType = z.infer<typeof UserFileFromUrlParamSchema>;

export const UserFileFromUrlResSchema = z.object({
    id: z.coerce.number(),
});
export type UserFileFromUrlResType = z.infer<typeof UserFileFromUrlResSchema>;

export const userFileFromUrl = async (
    param: UserFileFromUrlParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileFromUrlResType>> => {
    const { data } = await authApi().post('/api/user/app_file/from_url', param, config);
    return parseResData(data, UserFileFromUrlResSchema);
};

// ==================== 文件日志列表 ====================

export const UserFileLogsParamSchema = z.object({
    app_id: z.coerce.number(),
    file_id: z.coerce.number(),
    ...PageParam,
});
export type UserFileLogsParamType = z.infer<typeof UserFileLogsParamSchema>;

export const UserFileLogItemSchema = z.object({
    id: z.coerce.number(),
    file_chunk_id: z.coerce.number(),
    message: z.string(),
    user_data: UserDataResSchema.nullable().optional(),
    add_time: UnixTimestampSchema,
});
export type UserFileLogItemType = z.infer<typeof UserFileLogItemSchema>;

export const UserFileLogsResSchema = z.object({
    data: z.array(UserFileLogItemSchema),
    ...PageResSchema,
});
export type UserFileLogsResType = z.infer<typeof UserFileLogsResSchema>;

export const userFileLogs = async (
    param: UserFileLogsParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileLogsResType>> => {
    const { data } = await authApi().post('/api/user/app_file/logs', param, config);
    return parseResData(data, UserFileLogsResSchema);
};
// ==================== 文件分片列表 ====================

export const UserFileChunksParamSchema = z.object({
    app_id: z.coerce.number(),
    file_id: z.coerce.number(),
    ...PageParam,
});
export type UserFileChunksParamType = z.infer<typeof UserFileChunksParamSchema>;

export const UserFileChunkItemSchema = z.object({
    id: z.coerce.number(),
    file_id: z.coerce.number(),
    chunk_index: z.coerce.number(),
    start_offset: z.coerce.number(),
    chunk_md5: z.string(),
    upload_md5: z.string(),
    chunk_path: z.string(),
    file_size: z.coerce.number(),
    complete_size: z.coerce.number(),
    status: z.coerce.number(),
    add_time: UnixTimestampSchema,
    change_time: UnixTimestampSchema,
});
export type UserFileChunkItemType = z.infer<typeof UserFileChunkItemSchema>;

export const UserFileChunksResSchema = z.object({
    data: z.array(UserFileChunkItemSchema),
    ...PageResSchema,
});
export type UserFileChunksResType = z.infer<typeof UserFileChunksResSchema>;

export const userFileChunks = async (
    param: UserFileChunksParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileChunksResType>> => {
    const { data } = await authApi().post('/api/user/app_file/chunks', param, config);
    return parseResData(data, UserFileChunksResSchema);
};

// ==================== 文件标签管理 ====================

// 获取单个文件的标签列表
export const UserFileTagsParamSchema = z.object({
    id: z.coerce.number(),
});
export type UserFileTagsParamType = z.infer<typeof UserFileTagsParamSchema>;

export const UserFileTagItemSchema = z.object({
    id: z.coerce.number(),
    tag_name: z.string(),
    add_time: UnixTimestampSchema,
});
export type UserFileTagItemType = z.infer<typeof UserFileTagItemSchema>;

export const UserFileTagsResSchema = z.object({
    data: z.array(UserFileTagItemSchema),
});
export type UserFileTagsResType = z.infer<typeof UserFileTagsResSchema>;

export const userFileTags = async (
    param: UserFileTagsParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileTagsResType>> => {
    const { data } = await authApi().post('/api/user/app_file/tags', param, config);
    return parseResData(data, UserFileTagsResSchema);
};

// 添加标签
export const UserFileTagAddParamSchema = z.object({
    id: z.coerce.number(),
    tag_name: z.string(),
});
export type UserFileTagAddParamType = z.infer<typeof UserFileTagAddParamSchema>;

export const UserFileTagAddResSchema = z.object({
    id: z.coerce.number(),
});
export type UserFileTagAddResType = z.infer<typeof UserFileTagAddResSchema>;

export const userFileTagAdd = async (
    param: UserFileTagAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileTagAddResType>> => {
    const { data } = await authApi().post('/api/user/app_file/tag_add', param, config);
    return parseResData(data, UserFileTagAddResSchema);
};

// 移除标签
export const UserFileTagRemoveParamSchema = z.object({
    id: z.coerce.number(),
    tag_name: z.string(),
});
export type UserFileTagRemoveParamType = z.infer<typeof UserFileTagRemoveParamSchema>;

export const UserFileTagRemoveResSchema = z.object({
    affected: z.coerce.number(),
});
export type UserFileTagRemoveResType = z.infer<typeof UserFileTagRemoveResSchema>;

export const userFileTagRemove = async (
    param: UserFileTagRemoveParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileTagRemoveResType>> => {
    const { data } = await authApi().post('/api/user/app_file/tag_remove', param, config);
    return parseResData(data, UserFileTagRemoveResSchema);
};

// ==================== 文件批量导出 ====================

export const UserFileExportSubmitParamSchema = z.object({
    app_id: z.coerce.number(),
    export_type: z.string(),
    params: z.record(z.unknown()).optional(),
});
export type UserFileExportSubmitParamType = z.infer<typeof UserFileExportSubmitParamSchema>;

export const UserFileExportSubmitResSchema = z.object({
    id: z.coerce.number(),
});
export type UserFileExportSubmitResType = z.infer<typeof UserFileExportSubmitResSchema>;

export const userFileExportSubmit = async (
    param: UserFileExportSubmitParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileExportSubmitResType>> => {
    const { data } = await authApi().post('/api/user/app_export_task/export_submit', param, config);
    return parseResData(data, UserFileExportSubmitResSchema);
};

// ── 用户级（非 APP）导出提交 ──────────────────────────────────────────────────

export const UserExportSubmitParamSchema = z.object({
    export_type: z.string(),
    params: z.record(z.unknown()).optional(),
});
export type UserExportSubmitParamType = z.infer<typeof UserExportSubmitParamSchema>;

export const userExportSubmit = async (
    param: UserExportSubmitParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileExportSubmitResType>> => {
    const { data } = await authApi().post('/api/user/export_task/export_submit', param, config);
    return parseResData(data, UserFileExportSubmitResSchema);
};

// ==================== 用户导出类型常量 ====================

// 文件（用户账号级）
export const EXPORT_TYPE_USER_FILE_LIST = 'user_file_list';

// APP 文件
export const EXPORT_TYPE_APP_FILE_LIST = 'app_file_list';

// APP Collector 脚本
export const EXPORT_TYPE_APP_SCRIPT_RECORDS = 'app_script_records';

// 通知
export const EXPORT_TYPE_APP_NOTIFY_LIST = 'app_notify_list';

// 邮件
export const EXPORT_TYPE_USER_MAILER_MESSAGE_LIST = 'user_mailer_message_list';

// 短信
export const EXPORT_TYPE_USER_SMSER_MESSAGE_LIST = 'user_smser_message_list';

// 登录历史
export const EXPORT_TYPE_USER_LOGIN_HISTORY = 'user_login_history';

// RBAC
export const EXPORT_TYPE_APP_ROLE_DATA = 'app_role_data';
export const EXPORT_TYPE_APP_RES_DATA = 'app_res_data';

export const UserFileExportListParamSchema = z.object({
    app_id: z.coerce.number().optional(),
    export_type: z.string().optional(),
    status: z.coerce.number().optional(),
    page: z.object({
        page: z.coerce.number().min(1).optional(),
        limit: z.coerce.number().min(1).max(50).optional(),
    }).optional(),
    count_num: z.coerce.boolean().optional(),
});
export type UserFileExportListParamType = z.infer<typeof UserFileExportListParamSchema>;

// 与服务端 ExportTaskModel 字段对应
export const UserFileExportTaskFileSchema = z.object({
    file_id: z.coerce.number(),
    file_name: z.string(),
    file_size: z.coerce.number(),
    content_type: z.string(),
    file_url: z.string().nullable().optional(),
});
export type UserFileExportTaskFileType = z.infer<typeof UserFileExportTaskFileSchema>;

// 服务端 ExportTaskModel 平铺字段
export const ExportTaskModelSchema = z.object({
    id: z.coerce.number(),
    app_id: z.coerce.number(),
    export_type: z.string(),
    export_params: z.string(),          // JSON 字符串，需前端 JSON.parse 后展示
    status: z.coerce.number(),          // 1=Pending 2=Running 3=Success 4=Failed 5=Deleted
    error_message: z.string().optional(),
    add_time: UnixTimestampSchema,
    change_time: UnixTimestampSchema.nullable().optional(),
});

// 服务端 ExportTaskItem = { task: ExportTaskModel, file: ExportTaskFileItem | null }
export const UserFileExportTaskSchema = z.object({
    task: ExportTaskModelSchema,
    file: UserFileExportTaskFileSchema.nullable().optional(),
});
export type UserFileExportTaskType = z.infer<typeof UserFileExportTaskSchema>;

export const UserFileExportListResSchema = z.object({
    data: z.array(UserFileExportTaskSchema),
    total: z.coerce.number().nullable().optional(),
});
export type UserFileExportListResType = z.infer<typeof UserFileExportListResSchema>;

export const userFileExportList = async (
    param: UserFileExportListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileExportListResType>> => {
    const { data } = await authApi().post('/api/user/app_export_task/export_list', param, config);
    return parseResData(data, UserFileExportListResSchema);
};

// 用户级（非 APP）导出列表，不含 app_id 参数
export const UserExportListParamSchema = z.object({
    export_type: z.string().optional(),
    status: z.coerce.number().optional(),
    page: z.object({
        page: z.coerce.number().min(1).optional(),
        limit: z.coerce.number().min(1).max(50).optional(),
    }).optional(),
    count_num: z.coerce.boolean().optional(),
});
export type UserExportListParamType = z.infer<typeof UserExportListParamSchema>;

export const userExportList = async (
    param: UserExportListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileExportListResType>> => {
    const { data } = await authApi().post('/api/user/export_task/export_list', param, config);
    return parseResData(data, UserFileExportListResSchema);
};

// ==================== 活跃导出任务计数 ====================

export const UserFileExportActiveCountParamSchema = z.object({
    app_id: z.coerce.number().optional(),
    export_type: z.string().optional(),
});
export type UserFileExportActiveCountParamType = z.infer<typeof UserFileExportActiveCountParamSchema>;

export const UserFileExportActiveCountResSchema = z.object({
    count: z.coerce.number(),
});
export type UserFileExportActiveCountResType = z.infer<typeof UserFileExportActiveCountResSchema>;

export const userFileExportActiveCount = async (
    param: UserFileExportActiveCountParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileExportActiveCountResType>> => {
    const { data } = await authApi().post('/api/user/app_export_task/export_active_count', param, config);
    return parseResData(data, UserFileExportActiveCountResSchema);
};

// 用户级（非 APP）活跃任务计数，不含 app_id
export const UserExportActiveCountParamSchema = z.object({
    export_type: z.string().optional(),
});
export type UserExportActiveCountParamType = z.infer<typeof UserExportActiveCountParamSchema>;

export const userExportActiveCount = async (
    param: UserExportActiveCountParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileExportActiveCountResType>> => {
    const { data } = await authApi().post('/api/user/export_task/export_active_count', param, config);
    return parseResData(data, UserFileExportActiveCountResSchema);
};

// ==================== 导出任务字典映射 ====================

export const UserExportMappingResSchema = z.object({
    export_task_status: DictListSchema,
});
export type UserExportMappingResType = z.infer<typeof UserExportMappingResSchema>;

export const userExportTaskMapping = async (
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserExportMappingResType>> => {
    const { data } = await authApi().post('/api/user/app_export_task/mapping', {}, config);
    return parseResData(data, UserExportMappingResSchema);
};

// ==================== 文件操作 ====================

// --- 更新过期时间 ---
export const UserFileUpdateExpireTimeParamSchema = z.object({
    app_id: z.coerce.number(),
    file_ref_id: z.coerce.number(),
    expire_time: z.coerce.number(),
});
export type UserFileUpdateExpireTimeParamType = z.infer<typeof UserFileUpdateExpireTimeParamSchema>;

export const UserFileUpdateExpireTimeResSchema = z.object({
    updated: BoolSchema,
    rows_affected: z.coerce.number(),
});
export type UserFileUpdateExpireTimeResType = z.infer<typeof UserFileUpdateExpireTimeResSchema>;

export const userFileUpdateExpireTime = async (
    param: UserFileUpdateExpireTimeParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileUpdateExpireTimeResType>> => {
    const { data } = await authApi().post('/api/user/app_file/update_expire_time', param, config);
    return parseResData(data, UserFileUpdateExpireTimeResSchema);
};

// --- OSS 同步到本地 ---
export const UserFileSyncOssToLocalParamSchema = z.object({
    app_id: z.coerce.number(),
    file_ref_id: z.coerce.number(),
    storage_type: z.string().min(1, '存储类型不能为空'),
});
export type UserFileSyncOssToLocalParamType = z.infer<typeof UserFileSyncOssToLocalParamSchema>;

export const UserFileSyncOssToLocalResSchema = z.object({
    file_id: z.coerce.number(),
    file_ref_id: z.coerce.number(),
    storage_type: z.string(),
    file_name: z.string(),
    file_size: z.coerce.number(),
});
export type UserFileSyncOssToLocalResType = z.infer<typeof UserFileSyncOssToLocalResSchema>;

export const userFileSyncOssToLocal = async (
    param: UserFileSyncOssToLocalParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileSyncOssToLocalResType>> => {
    const { data } = await authApi().post('/api/user/app_file/sync_oss_to_local', param, config);
    return parseResData(data, UserFileSyncOssToLocalResSchema);
};

// --- 本地同步到 OSS ---
export const UserFileSyncLocalToOssParamSchema = z.object({
    app_id: z.coerce.number(),
    file_ref_id: z.coerce.number(),
    storage_type: z.string().min(1, '存储类型不能为空'),
});
export type UserFileSyncLocalToOssParamType = z.infer<typeof UserFileSyncLocalToOssParamSchema>;

export const UserFileSyncLocalToOssResSchema = z.object({
    file_id: z.coerce.number(),
    file_ref_id: z.coerce.number(),
    storage_type: z.string(),
    file_name: z.string(),
    file_size: z.coerce.number(),
});
export type UserFileSyncLocalToOssResType = z.infer<typeof UserFileSyncLocalToOssResSchema>;

export const userFileSyncLocalToOss = async (
    param: UserFileSyncLocalToOssParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileSyncLocalToOssResType>> => {
    const { data } = await authApi().post('/api/user/app_file/sync_local_to_oss', param, config);
    return parseResData(data, UserFileSyncLocalToOssResSchema);
};

// --- 文件拷贝 ---
export const UserFileCopyParamSchema = z.object({
    app_id: z.coerce.number(),
    file_ref_id: z.coerce.number(),
    storage_type: z.string().nullable().optional(),
});
export type UserFileCopyParamType = z.infer<typeof UserFileCopyParamSchema>;

export const UserFileCopyResSchema = z.object({
    file_id: z.coerce.number(),
    file_ref_id: z.coerce.number(),
    storage_type: z.string(),
    file_name: z.string(),
    file_size: z.coerce.number(),
});
export type UserFileCopyResType = z.infer<typeof UserFileCopyResSchema>;

export const userFileCopy = async (
    param: UserFileCopyParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileCopyResType>> => {
    const { data } = await authApi().post('/api/user/app_file/copy', param, config);
    return parseResData(data, UserFileCopyResSchema);
};


// ==================== 下载中文件列表 ====================

export const UserFileDownloadingListParamSchema = z.object({
    app_id: z.coerce.number(),
    user_id: z.coerce.number().nullable().optional(),
    is_downloading: z.coerce.boolean().optional(),
    ...LimitParam,
});
export type UserFileDownloadingListParamType = z.infer<typeof UserFileDownloadingListParamSchema>;

/**
 * 下载中文件条目（扁平结构，与后端响应对齐）
 * 相比普通文件列表，不包含标签、关联、OSS 等非必要字段
 */
export const UserFileDownloadingItemSchema = z.object({
    id: z.coerce.number(),
    file_id: z.coerce.number(),
    file_name: z.string(),
    file_md5: z.string().nullable(),
    file_size: z.coerce.number(),
    storage_type: z.string(),
    status: z.coerce.number(),
    content_type: z.string().nullable(),
    source_url: z.string().nullable(),
    url: z.string().nullable().optional(),
    add_time: UnixTimestampSchema,
    user_id: z.coerce.number(),
    is_downloading:BoolSchema.optional(),
    // 本地分片信息（用于显示下载进度）
    source_type: z.string().nullable().optional(),
    file_chunk_total: z.coerce.number().nullable().optional(),
    file_chunk_succ: z.coerce.number().nullable().optional(),
    file_chunk_size: z.coerce.number().nullable().optional(),
});
export type UserFileDownloadingItemType = z.infer<typeof UserFileDownloadingItemSchema>;

export const UserFileDownloadingListResSchema = z.object({
    data: z.array(UserFileDownloadingItemSchema),
    ...LimitResSchema,
});
export type UserFileDownloadingListResType = z.infer<typeof UserFileDownloadingListResSchema>;

export const userFileDownloadingList = async (
    param: UserFileDownloadingListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileDownloadingListResType>> => {
    const { data } = await authApi().post('/api/user/app_file/downloading_list', param, config);
    return parseResData(data, UserFileDownloadingListResSchema);
};

// ==================== 文件下载进度 SSE ====================

/**
 * 文件下载进度 SSE 事件数据
 * 对应后端 FileProgressInfo 结构
 */
export const FileDownloadProgressInfoSchema = z.object({
    file_id: z.coerce.number(),
    total_downloaded: z.coerce.number(),
    total_size: z.coerce.number(),
    /** 0.0 ~ 100.0 */
    percent: z.coerce.number(),
    /** 下载速度（bytes/sec） */
    speed_bps: z.coerce.number(),
    /** Downloading | Completed | Failed | Queued 等 */
    status: z.string(),
});
export type FileDownloadProgressInfoType = z.infer<typeof FileDownloadProgressInfoSchema>;

/** 下载进度 SSE 端点路径 */
export const USER_APP_FILE_DOWNLOAD_PROGRESS_SSE_URL = "/api/user/app_file/download_progress_sse";

// ==================== 文件关联（lineage）详细查询 ====================

export const UserFileLineageRelatedListParamSchema = z.object({
    id: z.coerce.number(),
    rel_type: z.coerce.number().nullable().optional(),
    storage_type: z.string().nullable().optional(),
    ...LimitParam,
});
export type UserFileLineageRelatedListParamType = z.infer<typeof UserFileLineageRelatedListParamSchema>;

export const UserFileLineageRelatedListResSchema = z.object({
    data: z.array(UserFileItemSchema),
    ...LimitResSchema,
});
export type UserFileLineageRelatedListResType = z.infer<typeof UserFileLineageRelatedListResSchema>;

export const userFileLineageRelatedList = async (
    param: UserFileLineageRelatedListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserFileLineageRelatedListResType>> => {
    const cleanedParam = cleanEmptyStringParams(param, ['storage_type']);
    const { data } = await authApi().post('/api/user/app_file/lineage_related_list', cleanedParam, config);
    return parseResData(data, UserFileLineageRelatedListResSchema);
};


// ==================== 用户文件（/user/file）上传 API ====================
// 与 app_file 类似，但走 /api/user/file 路径，不需要 app_id，用于用户自身文件（如头像）。

export const UserSelfFileUploadByMd5ParamSchema = z.object({
    file_md5: z.string(),
    file_name: z.string(),
    tag_names: z.array(z.string()).optional(),
});
export type UserSelfFileUploadByMd5ParamType = z.infer<typeof UserSelfFileUploadByMd5ParamSchema>;

export const UserSelfFileUploadByMd5ResSchema = z.object({
    matched: BoolSchema,
    id: z.coerce.number().optional(),
    file_key: z.string().optional(),
});
export type UserSelfFileUploadByMd5ResType = z.infer<typeof UserSelfFileUploadByMd5ResSchema>;

export const userSelfFileUploadByMd5 = async (
    param: UserSelfFileUploadByMd5ParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSelfFileUploadByMd5ResType>> => {
    const { data } = await authApi().post('/api/user/file/upload_by_md5', param, config);
    return parseResData(data, UserSelfFileUploadByMd5ResSchema);
};

export const UserSelfFileUploadCreateParamSchema = z.object({
    file_name: z.string(),
    chunks: z.array(UserFileUploadChunkSchema),
    tag_names: z.array(z.string()).optional(),
    storage_type: z.string().optional(),
});
export type UserSelfFileUploadCreateParamType = z.infer<typeof UserSelfFileUploadCreateParamSchema>;

export const UserSelfFileUploadCreateResSchema = z.object({
    id: z.coerce.number(),
    file_id: z.coerce.number(),
    file_name: z.string(),
    status: z.coerce.number(),
    file_key: z.string().optional(),
});
export type UserSelfFileUploadCreateResType = z.infer<typeof UserSelfFileUploadCreateResSchema>;

export const userSelfFileUploadCreate = async (
    param: UserSelfFileUploadCreateParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSelfFileUploadCreateResType>> => {
    const { data } = await authApi().post('/api/user/file/upload_create', param, config);
    return parseResData(data, UserSelfFileUploadCreateResSchema);
};

export const userSelfFileUploadData = async (
    fileRefId: number,
    chunkIndex: number,
    file: Blob,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<{}>> => {
    const formData = new FormData();
    formData.append('id', String(fileRefId));
    formData.append('chunk_index', String(chunkIndex));
    formData.append('file', file);
    const { data } = await authApi().post('/api/user/file/upload_data', formData, { ...config });
    // upload_data 成功时后端不返回 response body，直接检查 status
    if (data?.status) return data as ApiResult<{}>;
    return Promise.reject(data);
};

/**
 * 将 file_key 转为可访问的文件 URL（公开文件通过 /api/user/file/share/{key} 访问）
 */
export const fileKeyToUrl = (fileKey: string, apiBaseUrl: string): string => {
    return `${apiBaseUrl}/api/user/file/share/${fileKey}`;
};

// ==================== 用户自身文件列表（/api/user/file/list） ====================

export const UserSelfFileListParamSchema = z.object({
    url: z.string().nullable().optional(),
    source_url: z.string().nullable().optional(),
    add_time_start: z.coerce.number().nullable().optional(),
    add_time_end: z.coerce.number().nullable().optional(),
    storage_type: z.string().nullable().optional(),
    file_md5: z.string().nullable().optional(),
    status: z.coerce.number().nullable().optional(),
    tag_names: z.array(z.string()).nullable().optional(),
    attr_tag: z.coerce.boolean().nullable().optional(),
    attr_lineage: z.coerce.boolean().nullable().optional(),
    ...LimitParam,
});
export type UserSelfFileListParamType = z.infer<typeof UserSelfFileListParamSchema>;

export const UserSelfFileListResSchema = z.object({
    data: z.array(UserFileItemSchema),
    ...LimitResSchema,
});
export type UserSelfFileListResType = z.infer<typeof UserSelfFileListResSchema>;

export const userSelfFileList = async (
    param: UserSelfFileListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSelfFileListResType>> => {
    const cleanedParam = cleanEmptyStringParams(param, ['url', 'source_url', 'storage_type', 'file_md5']);
    const { data } = await authApi().post('/api/user/file/list', cleanedParam, config);
    return parseResData(data, UserSelfFileListResSchema);
};

// ==================== 用户自身文件删除（/api/user/file/delete） ====================

export const UserSelfFileDeleteParamSchema = z.object({
    file_ref_id: z.coerce.number(),
});
export type UserSelfFileDeleteParamType = z.infer<typeof UserSelfFileDeleteParamSchema>;

export const userSelfFileDelete = async (
    param: UserSelfFileDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<any>> => {
    const { data } = await authApi().post('/api/user/file/delete', param, config);
    return data;
};

// ==================== 用户自身文件标签名列表（/api/user/file/tag_names） ====================

export const UserSelfFileTagNamesParamSchema = z.object({
    tag_name_prefix: z.string().nullable().optional(),
    limit: z.coerce.number().nullable().optional(),
});
export type UserSelfFileTagNamesParamType = z.infer<typeof UserSelfFileTagNamesParamSchema>;

export const UserSelfFileTagNamesResSchema = z.object({
    data: z.array(z.string()),
});
export type UserSelfFileTagNamesResType = z.infer<typeof UserSelfFileTagNamesResSchema>;

export const userSelfFileTagNames = async (
    param: UserSelfFileTagNamesParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSelfFileTagNamesResType>> => {
    const { data } = await authApi().post('/api/user/file/tag_names', param, config);
    return parseResData(data, UserSelfFileTagNamesResSchema);
};
