import { authApi } from "@shared/lib/apis/api_auth";
import { cleanEmptyStringParams, parseResData } from "@shared/lib/apis/utils";
import { DictListSchema } from "@shared/types/apis-dict";
import { ApiResult } from "@shared/types/apis-rest";
import { BoolSchema, LimitParam, LimitResSchema, PageParam, PageResSchema, UnixTimestampSchema, UserDataResSchema } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

// ==================== 字典映射 ====================

export const UserFileMappingParamSchema = z.object({});
export type UserFileMappingParamType = z.infer<typeof UserFileMappingParamSchema>;

export const StorageTypeDictItemSchema = z.object({
    key: z.string(),
    val: z.string(),
    type: z.enum(['local', 'oss']),
});
export type StorageTypeDictItemType = z.infer<typeof StorageTypeDictItemSchema>;

export const UserFileMappingResSchema = z.object({
    min_chunk_size: z.coerce.number(),
    max_upload_size: z.coerce.number(),
    chunk_threshold: z.coerce.number(),
    default_chunk_size: z.coerce.number(),
    storage_type: z.array(StorageTypeDictItemSchema),
    file_source_type: DictListSchema,
    file_status: DictListSchema,
    file_chunk_status: DictListSchema,
    file_user_status: DictListSchema,
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
    attr_tag: z.boolean().nullable().optional(),
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

export const UserFileItemSchema = z.object({
    id: z.coerce.number(),
    file_id: z.coerce.number(),
    file_name: z.string(),
    file_md5: z.string().nullable(),
    file_size: z.coerce.number(),
    storage_type: z.string(),
    status: z.coerce.number(),
    content_type: z.string().nullable(),
    source_url: z.string().nullable(),
    url: z.string().nullable(),
    add_time: UnixTimestampSchema,
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
    file_user_id: z.coerce.number(),
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
    max_concurrency: z.coerce.number().optional(),
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

// ==================== 用户导出类型常量 ====================

// 文件
export const EXPORT_TYPE_USER_FILE_LIST = 'user_file_list';
export const EXPORT_TYPE_USER_FILE_LOG = 'user_file_log';
export const EXPORT_TYPE_USER_FILE_CHUNK = 'user_file_chunk';

// 应用
export const EXPORT_TYPE_USER_APP_REQUEST = 'user_app_request';
export const EXPORT_TYPE_USER_SUB_REQUEST = 'user_sub_request';
export const EXPORT_TYPE_USER_APP_LIST = 'user_app_list';
export const EXPORT_TYPE_USER_PARENT_APP_LIST = 'user_parent_app_list';
export const EXPORT_TYPE_USER_SUB_APP_LIST = 'user_sub_app_list';

// 通知
export const EXPORT_TYPE_APP_NOTIFY_LIST = 'app_notify_list';

// 邮件
export const EXPORT_TYPE_USER_MAILER_MESSAGE_LIST = 'user_mailer_message_list';
export const EXPORT_TYPE_USER_MAILER_MESSAGE_LOG = 'user_mailer_message_log';
export const EXPORT_TYPE_USER_MAILER_TPL_CONFIG = 'user_mailer_tpl_config';
export const EXPORT_TYPE_USER_MAILER_TPL_BODY = 'user_mailer_tpl_body';

// 短信
export const EXPORT_TYPE_USER_SMSER_MESSAGE_LIST = 'user_smser_message_list';
export const EXPORT_TYPE_USER_SMSER_MESSAGE_LOG = 'user_smser_message_log';
export const EXPORT_TYPE_USER_SMSER_TPL_CONFIG = 'user_smser_tpl_config';

// 登录历史
export const EXPORT_TYPE_USER_LOGIN_HISTORY = 'user_login_history';

// RBAC
export const EXPORT_TYPE_USER_RBAC_SYSTEM_AUDIT = 'user_rbac_system_audit';
export const EXPORT_TYPE_USER_RBAC_APP_AUDIT = 'user_rbac_app_audit';
export const EXPORT_TYPE_USER_RBAC_SYSTEM_ROLE_PERM = 'user_rbac_system_role_perm';
export const EXPORT_TYPE_USER_RBAC_APP_ROLE_PERM = 'user_rbac_app_role_perm';
export const EXPORT_TYPE_USER_RBAC_APP_RES = 'user_rbac_app_res';
export const EXPORT_TYPE_USER_RBAC_SYSTEM_ROLE_USER = 'user_rbac_system_role_user';
export const EXPORT_TYPE_USER_RBAC_APP_ROLE_USER = 'user_rbac_app_role_user';
export const EXPORT_TYPE_USER_RBAC_APP_RES_TYPE = 'user_rbac_app_res_type';
export const EXPORT_TYPE_USER_RBAC_APP_RES_TYPE_OP = 'user_rbac_app_res_type_op';
export const EXPORT_TYPE_USER_RBAC_APP_OP = 'user_rbac_app_op';
export const EXPORT_TYPE_USER_RBAC_SYSTEM_ROLE = 'user_rbac_system_role';
export const EXPORT_TYPE_USER_RBAC_APP_ROLE = 'user_rbac_app_role';
export const EXPORT_TYPE_USER_SYSTEM_ROLE_USER_AVAILABLE = 'user_system_role_user_available';
export const EXPORT_TYPE_USER_APP_ROLE_USER_AVAILABLE = 'user_app_role_user_available';

export const UserFileExportListParamSchema = z.object({
    app_id: z.coerce.number().optional(),
    export_type: z.string().optional(),
    status: z.coerce.number().optional(),
    page: z.object({
        page: z.coerce.number().min(1).optional(),
        limit: z.coerce.number().min(1).max(50).optional(),
    }).optional(),
    count_num: z.boolean().optional(),
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

export const UserFileExportTaskSchema = z.object({
    id: z.coerce.number(),
    app_id: z.coerce.number(),
    export_type: z.string(),
    export_params: z.string(),          // JSON 字符串，需前端 JSON.parse 后展示
    status: z.coerce.number(),          // 1=Pending 2=Running 3=Success 4=Failed 5=Deleted
    error_message: z.string().optional(),
    add_time: UnixTimestampSchema,
    change_time: UnixTimestampSchema.nullable().optional(),
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