import { authApi } from "@shared/lib/apis/api_auth";
import { parseResData } from "@shared/lib/apis/utils";
import { DictListSchema } from "@shared/types/apis-dict";
import { ApiResult } from "@shared/types/apis-rest";
import { PageParam, PageRes, UnixTimestampSchema } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

// User App Barcode APIs
export const UserBarcodeMapResSchema = z.object({
    create_status:DictListSchema,
    parse_status: DictListSchema,
    barcode_type: DictListSchema,
});
export type UserBarcodeMapResType = z.infer<typeof UserBarcodeMapResSchema>;

export async function userBarcodeMapping(
    param: {},
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserBarcodeMapResType>> {
    const { data } = await authApi().post("/api/user/app_barcode/mapping", param, config);
    return parseResData(data, UserBarcodeMapResSchema);
}

// Barcode Create Config List
export const UserBarcodeCreateConfigListParamSchema = z.object({
    id: z.coerce.number().optional().nullable(),
    app_id: z.coerce.number().optional().nullable(),
    barcode_type: z.string().optional().nullable(),
    ...PageParam,
});
export type UserBarcodeCreateConfigListParamType = z.infer<typeof UserBarcodeCreateConfigListParamSchema>;

export const UserBarcodeCreateConfigItemSchema = z.object({
    id: z.coerce.number(),
    app_id: z.coerce.number(),
    barcode_type: z.string(),
    status: z.coerce.number(),
    image_format: z.string(),
    image_width: z.coerce.number(),
    image_height: z.coerce.number(),
    margin: z.coerce.number(),
    image_color: z.string(),
    image_background: z.string(),
    change_time: UnixTimestampSchema,
    url: z.string(),
});
export type UserBarcodeCreateConfigItemType = z.infer<typeof UserBarcodeCreateConfigItemSchema>;

export const UserBarcodeCreateConfigListResSchema = z.object({
    data: z.array(UserBarcodeCreateConfigItemSchema),
    ...PageRes,
});
export type UserBarcodeCreateConfigListResType = z.infer<typeof UserBarcodeCreateConfigListResSchema>;

export async function userBarcodeCreateConfigList(
    param: UserBarcodeCreateConfigListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserBarcodeCreateConfigListResType>> {
    if (param?.barcode_type=="")delete param.barcode_type;
    const { data } = await authApi().post("/api/user/app_barcode/create_config_list", param, config);
    return parseResData(data, UserBarcodeCreateConfigListResSchema);
}

// Add Barcode Create Config
export const UserBarcodeCreateConfigAddParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    barcode_type: z.string().min(1, "条码类型不能为空"),
    status: z.coerce.number().min(1, "状态必须大于0"),
    image_format: z.string().min(1, "图片格式不能为空"),
    image_width: z.coerce.number().min(1, "图片宽度必须大于0"),
    image_height: z.coerce.number().min(1, "图片高度必须大于0"),
    margin: z.coerce.number().min(0, "边距不能小于0"),
    image_color: z.string().min(1, "前景色不能为空"),
    image_background: z.string().min(1, "背景色不能为空"),
});
export type UserBarcodeCreateConfigAddParamType = z.infer<typeof UserBarcodeCreateConfigAddParamSchema>;

export const UserBarcodeCreateConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type UserBarcodeCreateConfigAddResType = z.infer<typeof UserBarcodeCreateConfigAddResSchema>;

export async function userBarcodeCreateConfigAdd(
    param: UserBarcodeCreateConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserBarcodeCreateConfigAddResType>> {
    const { data } = await authApi().post("/api/user/app_barcode/create_config_add", param, config);
    return parseResData(data, UserBarcodeCreateConfigAddResSchema);
}

// Edit Barcode Create Config
export const UserBarcodeCreateConfigEditParamSchema = z.object({
    id: z.coerce.number().min(1, "配置ID必须大于0"),
    barcode_type: z.string().min(1, "条码类型不能为空"),
    status: z.coerce.number().min(0, "状态不能小于0"),
    image_format: z.string().min(1, "图片格式不能为空"),
    image_width: z.coerce.number().min(1, "图片宽度必须大于0"),
    image_height: z.coerce.number().min(1, "图片高度必须大于0"),
    margin: z.coerce.number().min(0, "边距不能小于0"),
    image_color: z.string().min(1, "前景色不能为空"),
    image_background: z.string().min(1, "背景色不能为空"),
});
export type UserBarcodeCreateConfigEditParamType = z.infer<typeof UserBarcodeCreateConfigEditParamSchema>;

export async function userBarcodeCreateConfigEdit(
    param: UserBarcodeCreateConfigEditParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app_barcode/create_config_edit", param, config);
    return data;
}

// Delete Barcode Create Config
export const UserBarcodeCreateConfigDeleteParamSchema = z.object({
    id: z.coerce.number().min(1, "配置ID必须大于0"),
});
export type UserBarcodeCreateConfigDeleteParamType = z.infer<typeof UserBarcodeCreateConfigDeleteParamSchema>;

export async function userBarcodeCreateConfigDelete(
    param: UserBarcodeCreateConfigDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app_barcode/create_config_delete", param, config);
    return data;
}

// Parse Record List
export const UserBarcodeParseRecordListParamSchema = z.object({
    id: z.coerce.number().optional().nullable(),
    app_id: z.coerce.number().optional().nullable(),
    barcode_type: z.string().optional(),
    ...PageParam,
});
export type UserBarcodeParseRecordListParamType = z.infer<typeof UserBarcodeParseRecordListParamSchema>;

export const UserBarcodeParseRecordItemSchema = z.object({
    id: z.coerce.number(),
    app_id: z.coerce.number(),
    bar_type: z.string(),
    text: z.string(),
    status: z.coerce.number(),
    error: z.string(),
    hash: z.string(),
    create_time: UnixTimestampSchema,
});
export type UserBarcodeParseRecordItemType = z.infer<typeof UserBarcodeParseRecordItemSchema>;

export const UserBarcodeParseRecordListResSchema = z.object({
    data: z.array(UserBarcodeParseRecordItemSchema),
    ...PageRes,
});
export type UserBarcodeParseRecordListResType = z.infer<typeof UserBarcodeParseRecordListResSchema>;

export async function userBarcodeParseRecordList(
    param: UserBarcodeParseRecordListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserBarcodeParseRecordListResType>> {
    if (param?.barcode_type=="")delete param.barcode_type;
    const { data } = await authApi().post("/api/user/app_barcode/parse_record_list", param, config);
    return parseResData(data, UserBarcodeParseRecordListResSchema);
}

// Delete Parse Record
export const UserBarcodeParseRecordDeleteParamSchema = z.object({
    id: z.coerce.number().min(1, "记录ID必须大于0"),
});
export type UserBarcodeParseRecordDeleteParamType = z.infer<typeof UserBarcodeParseRecordDeleteParamSchema>;

export async function userBarcodeParseRecordDelete(
    param: UserBarcodeParseRecordDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app_barcode/parse_record_delete", param, config);
    return data;
}


