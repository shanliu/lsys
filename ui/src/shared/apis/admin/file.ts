import { authApi } from "@shared/lib/apis/api_auth";
import { cleanEmptyStringParams, parseResData } from "@shared/lib/apis/utils";
import { DictListSchema } from "@shared/types/apis-dict";
import { ApiResult } from "@shared/types/apis-rest";
import {
  LimitParam,
  LimitResSchema,
  PageResSchema,
  UnixTimestampSchema,
} from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

/**
 * 管理员文件管理 API
 * 对应后端: /api/system/file/
 */

// ==================== 字典映射 ====================

export const StorageTypeDictItemSchema = z.object({
  key: z.string(),
  val: z.string(),
  type: z.enum(["local", "oss"]),
});
export type StorageTypeDictItemType = z.infer<typeof StorageTypeDictItemSchema>;

export const AdminFileMappingResSchema = z.object({
  storage_type: z.array(StorageTypeDictItemSchema),
  file_source_type: DictListSchema,
  file_status: DictListSchema,
  file_chunk_status: DictListSchema,
  file_user_status: DictListSchema,
  file_tag_status: DictListSchema,
});
export type AdminFileMappingResType = z.infer<typeof AdminFileMappingResSchema>;

export const adminFileMapping = async (
  config?: AxiosRequestConfig<any>,
): Promise<ApiResult<AdminFileMappingResType>> => {
  const { data } = await authApi().post("/api/system/file/mapping", {}, config);
  return parseResData(data, AdminFileMappingResSchema);
};

// ==================== 文件列表（管理员视角） ====================

export const AdminFileListParamSchema = z.object({
  user_id: z.coerce.number().nullable().optional(),
  url: z.string().nullable().optional(),
  source_url: z.string().nullable().optional(),
  add_time_start: z.coerce.number().nullable().optional(),
  add_time_end: z.coerce.number().nullable().optional(),
  status: z.coerce.number().nullable().optional(),
  storage_type: z.string().nullable().optional(),
  file_md5: z.string().nullable().optional(),
  tag_names: z.array(z.string()).nullable().optional(),
  ...LimitParam,
});
export type AdminFileListParamType = z.infer<typeof AdminFileListParamSchema>;

export const AdminFileTagSchema = z.object({
  tag_name: z.string(),
  add_time: UnixTimestampSchema,
});
export type AdminFileTagType = z.infer<typeof AdminFileTagSchema>;

export const AdminFileItemSchema = z.object({
  id: z.coerce.number(),
  file_id: z.coerce.number(),
  file_name: z.string(),
  file_md5: z.string().nullable(),
  file_size: z.coerce.number(),
  storage_type: z.string(),
  status: z.coerce.number(),
  content_type: z.string().nullable(),
  source_url: z.string().nullable(),
  file_url: z.string().nullable(),
  add_time: UnixTimestampSchema,
  user_id: z.coerce.number(),
  from_user_id: z.coerce.number().nullable().optional(),
  copy_file_id: z.coerce.number().nullable().optional(),
  // 本地存储属性
  local_id: z.coerce.number().nullable().optional(),
  source_type: z.string().nullable().optional(),
  source_name: z.string().nullable().optional(),
  local_path: z.string().nullable().optional(),
  file_chunk_total: z.coerce.number().nullable().optional(),
  file_chunk_succ: z.coerce.number().nullable().optional(),
  file_chunk_size: z.coerce.number().nullable().optional(),
  // OSS 存储属性
  oss_id: z.coerce.number().nullable().optional(),
  object_key: z.string().nullable().optional(),
  object_url: z.string().nullable().optional(),
  bucket: z.string().nullable().optional(),
  region: z.string().nullable().optional(),
  oss_size: z.coerce.number().nullable().optional(),
  // 标签
  tags: z.array(AdminFileTagSchema).nullable().optional(),
});
export type AdminFileItemType = z.infer<typeof AdminFileItemSchema>;

export const AdminFileListResSchema = z.object({
  data: z.array(AdminFileItemSchema),
  ...LimitResSchema,
});
export type AdminFileListResType = z.infer<typeof AdminFileListResSchema>;

export const adminFileList = async (
  param: AdminFileListParamType,
  config?: AxiosRequestConfig<any>,
): Promise<ApiResult<AdminFileListResType>> => {
  const cleanedParam = cleanEmptyStringParams(param, [
    "url",
    "source_url",
    "storage_type",
    "file_md5",
  ]);
  const { data } = await authApi().post(
    "/api/system/file/list",
    cleanedParam,
    config,
  );
  return parseResData(data, AdminFileListResSchema);
};

// ==================== 文件删除 ====================

export const AdminFileDeleteParamSchema = z.object({
  file_user_id: z.coerce.number(),
});
export type AdminFileDeleteParamType = z.infer<
  typeof AdminFileDeleteParamSchema
>;

export const adminFileDelete = async (
  param: AdminFileDeleteParamType,
  config?: AxiosRequestConfig<any>,
): Promise<ApiResult<any>> => {
  const { data } = await authApi().post(
    "/api/system/file/delete",
    param,
    config,
  );
  return data;
};

// ==================== OSS 配置管理 ====================

// --- 列表 ---
export const AdminOssConfigListParamSchema = z.object({
  page: z.coerce.number().optional(),
  limit: z.coerce.number().optional(),
  count_num: z.boolean().optional(),
});
export type AdminOssConfigListParamType = z.infer<
  typeof AdminOssConfigListParamSchema
>;

export const AdminOssConfigItemSchema = z.object({
  id: z.coerce.number(),
  name: z.string(),
  config_key: z.string(),
  provider_type: z.string(),
  provider_config: z.record(z.unknown()),
  change_user_id: z.coerce.number(),
  change_time: UnixTimestampSchema,
});
export type AdminOssConfigItemType = z.infer<typeof AdminOssConfigItemSchema>;

export const AdminOssConfigListResSchema = z.object({
  data: z.array(AdminOssConfigItemSchema),
  ...PageResSchema,
});
export type AdminOssConfigListResType = z.infer<
  typeof AdminOssConfigListResSchema
>;

export const adminOssConfigList = async (
  param: AdminOssConfigListParamType,
  config?: AxiosRequestConfig<any>,
): Promise<ApiResult<AdminOssConfigListResType>> => {
  const { data } = await authApi().post(
    "/api/system/file/oss_config_list",
    param,
    config,
  );
  return parseResData(data, AdminOssConfigListResSchema);
};

// --- 详情 ---
export const AdminOssConfigDetailParamSchema = z.object({
  id: z.coerce.number(),
});
export type AdminOssConfigDetailParamType = z.infer<
  typeof AdminOssConfigDetailParamSchema
>;

export const adminOssConfigDetail = async (
  param: AdminOssConfigDetailParamType,
  config?: AxiosRequestConfig<any>,
): Promise<ApiResult<AdminOssConfigItemType>> => {
  const { data } = await authApi().post(
    "/api/system/file/oss_config_detail",
    param,
    config,
  );
  return parseResData(data, AdminOssConfigItemSchema);
};

// --- 新增 ---
export const AdminOssConfigAddParamSchema = z.object({
  name: z.string().min(1, "名称不能为空"),
  config_key: z
    .string()
    .min(1, "配置标识不能为空")
    .max(32, "配置标识最长32字符")
    .regex(
      /^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$/,
      "只允许小写字母、数字、连字符，不能以连字符开头或结尾",
    ),
  provider_type: z.string().min(1, "厂商类型不能为空"),
  provider_config: z.record(z.unknown()),
});
export type AdminOssConfigAddParamType = z.infer<
  typeof AdminOssConfigAddParamSchema
>;

export const AdminOssConfigAddResSchema = z.object({
  id: z.coerce.number(),
});
export type AdminOssConfigAddResType = z.infer<
  typeof AdminOssConfigAddResSchema
>;

export const adminOssConfigAdd = async (
  param: AdminOssConfigAddParamType,
  config?: AxiosRequestConfig<any>,
): Promise<ApiResult<AdminOssConfigAddResType>> => {
  const { data } = await authApi().post(
    "/api/system/file/oss_config_add",
    param,
    config,
  );
  return parseResData(data, AdminOssConfigAddResSchema);
};

// --- 修改 ---
export const AdminOssConfigEditParamSchema = z.object({
  id: z.coerce.number(),
  name: z.string().min(1, "名称不能为空"),
  provider_config: z.record(z.unknown()),
});
export type AdminOssConfigEditParamType = z.infer<
  typeof AdminOssConfigEditParamSchema
>;

export const adminOssConfigEdit = async (
  param: AdminOssConfigEditParamType,
  config?: AxiosRequestConfig<any>,
): Promise<ApiResult<any>> => {
  const { data } = await authApi().post(
    "/api/system/file/oss_config_edit",
    param,
    config,
  );
  return data;
};

// --- 删除 ---
export const AdminOssConfigDeleteParamSchema = z.object({
  id: z.coerce.number(),
});
export type AdminOssConfigDeleteParamType = z.infer<
  typeof AdminOssConfigDeleteParamSchema
>;

export const adminOssConfigDelete = async (
  param: AdminOssConfigDeleteParamType,
  config?: AxiosRequestConfig<any>,
): Promise<ApiResult<any>> => {
  const { data } = await authApi().post(
    "/api/system/file/oss_config_delete",
    param,
    config,
  );
  return data;
};
