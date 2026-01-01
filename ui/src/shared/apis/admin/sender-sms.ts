import { authApi } from "@shared/lib/apis/api_auth";
import { cleanEmptyStringParams, parseResData } from "@shared/lib/apis/utils";
import { DictListSchema } from "@shared/types/apis-dict";
import { ApiResult } from "@shared/types/apis-rest";
import { BoolSchema, LimitParam, LimitRes, PageParam, PageRes, UnixTimestampSchema } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

// Sender SMS APIs
export const SystemSenderSmsMapResSchema = z.object({
    config_type: DictListSchema,
    log_status: DictListSchema,
    log_type: DictListSchema,
    sms_body_status: DictListSchema,
    sms_config_type: DictListSchema,
    sms_send_status: DictListSchema,
});
export type SystemSenderSmsMapResType = z.infer<typeof SystemSenderSmsMapResSchema>;

export async function systemSenderSmsMapping(
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderSmsMapResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/mapping", {}, config);
    return parseResData(data, SystemSenderSmsMapResSchema);
}

// SMS Config List

export const SystemSenderSmsConfigItemSchema = z.object({
    id: z.coerce.number(),
    priority: z.coerce.number(),
    config_type: z.string(),
    config_data: z.string(),
    add_time: UnixTimestampSchema,
});
export type SystemSenderSmsConfigItemType = z.infer<typeof SystemSenderSmsConfigItemSchema>;

export const SystemSenderSmsConfigListResSchema = z.object({
    data: z.array(SystemSenderSmsConfigItemSchema),
});
export type SystemSenderSmsConfigListResType = z.infer<typeof SystemSenderSmsConfigListResSchema>;

export async function systemSenderSmsConfigList(
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderSmsConfigListResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/config_list", {}, config);
    return parseResData(data, SystemSenderSmsConfigListResSchema);
}

// Add SMS Config
export const SystemSenderSmsConfigAddParamSchema = z.object({
    priority: z.coerce.number(),
    config_type: z.coerce.number(),
    config_data: z.coerce.number(),
});
export type SystemSenderSmsConfigAddParamType = z.infer<typeof SystemSenderSmsConfigAddParamSchema>;

export const SystemSenderSmsConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type SystemSenderSmsConfigAddResType = z.infer<typeof SystemSenderSmsConfigAddResSchema>;

export async function systemSenderSmsConfigAdd(
    param: SystemSenderSmsConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderSmsConfigAddResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/config_add", param, config);
    return parseResData(data, SystemSenderSmsConfigAddResSchema);
}

// Delete SMS Config
export const SystemSenderSmsConfigDelParamSchema = z.object({
    config_id: z.coerce.number(),
});
export type SystemSenderSmsConfigDelParamType = z.infer<typeof SystemSenderSmsConfigDelParamSchema>;

export async function systemSenderSmsConfigDel(
    param: SystemSenderSmsConfigDelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/sender/smser/config_del", param, config);
    return data;
}

// SMS Message List
export const SystemSenderSmsMessageListParamSchema = z.object({
    tpl_key: z.string().optional(),
    status: z.coerce.number().optional().nullable(),
    body_id: z.coerce.number().optional().nullable(),
    snid: z.string().optional(),
    mobile: z.string().optional(),
    ...LimitParam,
});
export type SystemSenderSmsMessageListParamType = z.infer<typeof SystemSenderSmsMessageListParamSchema>;

export const SystemSenderSmsMessageItemSchema = z.object({
    id: z.coerce.number(),
    app_id: z.coerce.number(),
    mobile: z.string(),
    snid: z.string(),
    tpl_key: z.string(),
    status: z.coerce.number(),
    body_status: z.coerce.number().nullable(),
    add_time: UnixTimestampSchema,
    expected_time: UnixTimestampSchema,
    send_time: UnixTimestampSchema,
    max_try_num: z.coerce.number(),
    try_num: z.coerce.number(),
    now_send: z.coerce.number(),
    on_task: z.coerce.number(),
});
export type SystemSenderSmsMessageItemType = z.infer<typeof SystemSenderSmsMessageItemSchema>;

export const SystemSenderSmsMessageListResSchema = z.object({
    data: z.array(SystemSenderSmsMessageItemSchema),
    ...LimitRes,
});
export type SystemSenderSmsMessageListResType = z.infer<typeof SystemSenderSmsMessageListResSchema>;

export async function systemSenderSmsMessageList(
    param: SystemSenderSmsMessageListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderSmsMessageListResType>> {
    const cleanedParam = cleanEmptyStringParams(param, ['tpl_key', 'snid', 'mobile']);
    const { data } = await authApi().post("/api/system/sender/smser/message_list", cleanedParam, config);
    return parseResData(data, SystemSenderSmsMessageListResSchema);
}

// Cancel SMS Message
export const SystemSenderSmsMessageCancelParamSchema = z.object({
    message_id: z.coerce.number(),
});
export type SystemSenderSmsMessageCancelParamType = z.infer<typeof SystemSenderSmsMessageCancelParamSchema>;

export async function systemSenderSmsMessageCancel(
    param: SystemSenderSmsMessageCancelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/sender/smser/message_cancel", param, config);
    return data;
}

// View SMS Message
export const SystemSenderSmsMessageViewParamSchema = z.object({
    message_id: z.coerce.number(),
});
export type SystemSenderSmsMessageViewParamType = z.infer<typeof SystemSenderSmsMessageViewParamSchema>;

export const SystemSenderSmsMessageViewResSchema = z.object({
    body: z.string(),
});
export type SystemSenderSmsMessageViewResType = z.infer<typeof SystemSenderSmsMessageViewResSchema>;

export async function systemSenderSmsMessageView(
    param: SystemSenderSmsMessageViewParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderSmsMessageViewResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/message_view", param, config);
    return parseResData(data, SystemSenderSmsMessageViewResSchema);
}

// SMS Message Logs
export const SystemSenderSmsMessageLogsParamSchema = z.object({
    message_id: z.coerce.number(),
    ...PageParam,
});
export type SystemSenderSmsMessageLogsParamType = z.infer<typeof SystemSenderSmsMessageLogsParamSchema>;

export const SystemSenderSmsMessageLogItemSchema = z.object({
    id: z.coerce.number(),
    app_id: z.coerce.number(),
    sender_message_id: z.coerce.number(),
    sender_type: z.string(),
    executor_type: z.string(),
    log_type: z.string(),
    status: z.string(),
    message: z.string(),
    create_time: UnixTimestampSchema,
});
export type SystemSenderSmsMessageLogItemType = z.infer<typeof SystemSenderSmsMessageLogItemSchema>;

export const SystemSenderSmsMessageLogsResSchema = z.object({
    data: z.array(SystemSenderSmsMessageLogItemSchema),
    ...PageRes,
});
export type SystemSenderSmsMessageLogsResType = z.infer<typeof SystemSenderSmsMessageLogsResSchema>;

export async function systemSenderSmsMessageLogs(
    param: SystemSenderSmsMessageLogsParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderSmsMessageLogsResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/message_logs", param, config);
    return parseResData(data, SystemSenderSmsMessageLogsResSchema);
}

// Template Config APIs
export const SystemSenderSmsTplConfigListParamSchema = z.object({
    id: z.coerce.number().optional().nullable(),
    tpl: z.string().optional(),
    app_info: BoolSchema.optional(),
    ...PageParam,
});
export type SystemSenderSmsTplConfigListParamType = z.infer<typeof SystemSenderSmsTplConfigListParamSchema>;

export const SystemSenderSmsTplConfigItemSchema = z.object({
    id: z.coerce.number(),
    name: z.string(),
    app_id: z.coerce.number(),
    app_name: z.string().optional(),
    app_client_id: z.string().optional(),
    user_id: z.coerce.number(),
    setting_id: z.coerce.number(),
    setting_key: z.string(),
    setting_name: z.string(),
    tpl_key: z.string(),
    config_data: z.object({
        // 通用字段
        template_id: z.string().optional(),
        template_map: z.string().optional(),
        sign_name: z.string().optional(),
        signature: z.string().optional(),
        sender: z.string().optional(),
        // 阿里云短信配置
        aliyun_sms_tpl: z.string().optional(),
        aliyun_sign_name: z.string().optional(),
        // 兼容邮件配置格式
        body_tpl_id: z.string().optional(),
        from_email: z.string().optional(),
        reply_email: z.string().optional(),
        subject_tpl_id: z.string().optional(),
    }).passthrough(),
    change_time: UnixTimestampSchema,
    change_user_id: z.coerce.number(),
});
export type SystemSenderSmsTplConfigItemType = z.infer<typeof SystemSenderSmsTplConfigItemSchema>;

export const SystemSenderSmsTplConfigListResSchema = z.object({
    data: z.array(SystemSenderSmsTplConfigItemSchema),
    ...PageRes,
});
export type SystemSenderSmsTplConfigListResType = z.infer<typeof SystemSenderSmsTplConfigListResSchema>;

export async function systemSenderSmsTplConfigList(
    param: SystemSenderSmsTplConfigListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderSmsTplConfigListResType>> {
    const cleanedParam = cleanEmptyStringParams(param, ['tpl']);
    const { data } = await authApi().post("/api/system/sender/smser/tpl_config_list", cleanedParam, config);
    return parseResData(data, SystemSenderSmsTplConfigListResSchema);
}

export const SystemSenderSmsTplConfigDeleteParamSchema = z.object({
    tpl_config_id: z.coerce.number(),
});
export type SystemSenderSmsTplConfigDeleteParamType = z.infer<typeof SystemSenderSmsTplConfigDeleteParamSchema>;

export const SystemSenderSmsTplConfigDeleteResSchema = z.object({
    num: z.string(),
});
export type SystemSenderSmsTplConfigDeleteResType = z.infer<typeof SystemSenderSmsTplConfigDeleteResSchema>;

export async function systemSenderSmsTplConfigDelete(
    param: SystemSenderSmsTplConfigDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderSmsTplConfigDeleteResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/tpl_config_delete", param, config);
    return parseResData(data, SystemSenderSmsTplConfigDeleteResSchema);
}

// ===== 阿里云短信配置 =====
// 阿里云短信配置添加
export const AliSmsConfigAddParamSchema = z.object({
    name: z.string(),
    access_id: z.string(),
    access_secret: z.string(),
    region: z.string(),
    callback_key: z.string().optional(),
    limit: z.coerce.number().optional().nullable(),
});
export type AliSmsConfigAddParamType = z.infer<typeof AliSmsConfigAddParamSchema>;

export const AliSmsConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type AliSmsConfigAddResType = z.infer<typeof AliSmsConfigAddResSchema>;

export async function aliSmsConfigAdd(
    param: AliSmsConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AliSmsConfigAddResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/ali_config_add", param, config);
    return parseResData(data, AliSmsConfigAddResSchema);
}

// 阿里云短信配置删除
export const AliSmsConfigDelParamSchema = z.object({
    id: z.coerce.number(),
});
export type AliSmsConfigDelParamType = z.infer<typeof AliSmsConfigDelParamSchema>;

export const AliSmsConfigDelResSchema = z.object({
    num: z.string(),
});
export type AliSmsConfigDelResType = z.infer<typeof AliSmsConfigDelResSchema>;

export async function aliSmsConfigDel(
    param: AliSmsConfigDelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AliSmsConfigDelResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/ali_config_del", param, config);
    return parseResData(data, AliSmsConfigDelResSchema);
}

// 阿里云短信配置列表
export const AliSmsConfigListParamSchema = z.object({
    ids: z.array(z.coerce.number()).optional(),
});
export type AliSmsConfigListParamType = z.infer<typeof AliSmsConfigListParamSchema>;

export const AliSmsConfigItemSchema = z.object({
    id: z.coerce.number(),
    name: z.string(),
    access_id: z.string(),
    access_secret: z.string(),
    hide_access_id: z.string(),
    region: z.string(),
    callback_key: z.string(),
    callback_url: z.string(),
    limit: z.string(),
    change_time: UnixTimestampSchema,
    change_user_id: z.coerce.number(),
});


export type AliSmsConfigItemType = z.infer<typeof AliSmsConfigItemSchema>

export const AliSmsConfigListResSchema = z.object({
    data: z.array(AliSmsConfigItemSchema),
});
export type AliSmsConfigListResType = z.infer<typeof AliSmsConfigListResSchema>;

export async function aliSmsConfigList(
    param: AliSmsConfigListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AliSmsConfigListResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/ali_config_list", param, config);
    return parseResData(data, AliSmsConfigListResSchema);
}

// 阿里云短信配置编辑
export const AliSmsConfigEditParamSchema = z.object({
    id: z.coerce.number(),
    name: z.string(),
    access_id: z.string(),
    access_secret: z.string(),
    region: z.string(),
    callback_key: z.string().optional(),
    limit: z.coerce.number().optional().nullable(),
});
export type AliSmsConfigEditParamType = z.infer<typeof AliSmsConfigEditParamSchema>;

export const AliSmsConfigEditResSchema = z.object({
    num: z.string(),
});
export type AliSmsConfigEditResType = z.infer<typeof AliSmsConfigEditResSchema>;

export async function aliSmsConfigEdit(
    param: AliSmsConfigEditParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AliSmsConfigEditResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/ali_config_edit", param, config);
    return parseResData(data, AliSmsConfigEditResSchema);
}

// 阿里云短信模板配置添加
export const AliSmsTplConfigAddParamSchema = z.object({
    name: z.string(),
    ali_config_id: z.coerce.number(),
    tpl_key: z.string(),
    aliyun_sms_tpl: z.string(),
    aliyun_sign_name: z.string(),
});
export type AliSmsTplConfigAddParamType = z.infer<typeof AliSmsTplConfigAddParamSchema>;

export const AliSmsTplConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type AliSmsTplConfigAddResType = z.infer<typeof AliSmsTplConfigAddResSchema>;

export async function aliSmsTplConfigAdd(
    param: AliSmsTplConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<AliSmsTplConfigAddResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/ali_tpl_config_add", param, config);
    return parseResData(data, AliSmsTplConfigAddResSchema);
}

// ===== 容联云短信配置 =====
// 容联云短信配置添加
export const CloopenSmsConfigAddParamSchema = z.object({
    name: z.string(),
    account_sid: z.string(),
    account_token: z.string(),
    sms_app_id: z.string(),
    limit: z.coerce.number().optional().nullable(),
    callback_key: z.string().optional(),
});
export type CloopenSmsConfigAddParamType = z.infer<typeof CloopenSmsConfigAddParamSchema>;

export const CloopenSmsConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type CloopenSmsConfigAddResType = z.infer<typeof CloopenSmsConfigAddResSchema>;

export async function cloopenSmsConfigAdd(
    param: CloopenSmsConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<CloopenSmsConfigAddResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/cloopen_config_add", param, config);
    return parseResData(data, CloopenSmsConfigAddResSchema);
}

// 容联云短信配置删除
export const CloopenSmsConfigDelParamSchema = z.object({
    id: z.coerce.number(),
});
export type CloopenSmsConfigDelParamType = z.infer<typeof CloopenSmsConfigDelParamSchema>;

export const CloopenSmsConfigDelResSchema = z.object({
    num: z.string(),
});
export type CloopenSmsConfigDelResType = z.infer<typeof CloopenSmsConfigDelResSchema>;

export async function cloopenSmsConfigDel(
    param: CloopenSmsConfigDelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<CloopenSmsConfigDelResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/cloopen_config_del", param, config);
    return parseResData(data, CloopenSmsConfigDelResSchema);
}

// 容联云短信配置编辑
export const CloopenSmsConfigEditParamSchema = z.object({
    id: z.coerce.number(),
    name: z.string(),
    account_sid: z.string(),
    account_token: z.string(),
    sms_app_id: z.string(),
    limit: z.coerce.number().optional().nullable(),
    callback_key: z.string().optional(),
});
export type CloopenSmsConfigEditParamType = z.infer<typeof CloopenSmsConfigEditParamSchema>;

export const CloopenSmsConfigEditResSchema = z.object({
    num: z.string(),
});
export type CloopenSmsConfigEditResType = z.infer<typeof CloopenSmsConfigEditResSchema>;

export async function cloopenSmsConfigEdit(
    param: CloopenSmsConfigEditParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<CloopenSmsConfigEditResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/cloopen_config_edit", param, config);
    return parseResData(data, CloopenSmsConfigEditResSchema);
}

// 容联云短信配置列表
export const CloopenSmsConfigListParamSchema = z.object({
    ids: z.array(z.coerce.number()).optional(),
});
export type CloopenSmsConfigListParamType = z.infer<typeof CloopenSmsConfigListParamSchema>;

export const CloopenSmsConfigItemSchema = z.object({
    id: z.coerce.number(),
    name: z.string(),
    account_sid: z.string(),
    account_token: z.string(),
    hide_account_sid: z.string(),
    sms_app_id: z.string(),
    callback_key: z.string(),
    callback_url: z.string(),
    limit: z.string(),
    change_time: UnixTimestampSchema,
    change_user_id: z.coerce.number(),
});
export type CloopenSmsConfigItemType = z.infer<typeof CloopenSmsConfigItemSchema>;

export const CloopenSmsConfigListResSchema = z.object({
    data: z.array(CloopenSmsConfigItemSchema),
});
export type CloopenSmsConfigListResType = z.infer<typeof CloopenSmsConfigListResSchema>;

export async function cloopenSmsConfigList(
    param: CloopenSmsConfigListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<CloopenSmsConfigListResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/cloopen_config_list", param, config);
    return parseResData(data, CloopenSmsConfigListResSchema);
}

// 容联云短信模板配置添加
export const CloopenSmsTplConfigAddParamSchema = z.object({
    name: z.string(),
    config_id: z.coerce.number(),
    tpl_key: z.string(),
    template_id: z.string(),
    template_map: z.string(),
});
export type CloopenSmsTplConfigAddParamType = z.infer<typeof CloopenSmsTplConfigAddParamSchema>;

export const CloopenSmsTplConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type CloopenSmsTplConfigAddResType = z.infer<typeof CloopenSmsTplConfigAddResSchema>;

export async function cloopenSmsTplConfigAdd(
    param: CloopenSmsTplConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<CloopenSmsTplConfigAddResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/cloopen_tpl_config_add", param, config);
    return parseResData(data, CloopenSmsTplConfigAddResSchema);
}

// ===== 华为云短信配置 =====
// 华为云短信配置添加
export const HwSmsConfigAddParamSchema = z.object({
    name: z.string(),
    url: z.string(),
    app_key: z.string(),
    app_secret: z.string(),
    callback_key: z.string().optional(),
    limit: z.coerce.number().optional().nullable(),
});
export type HwSmsConfigAddParamType = z.infer<typeof HwSmsConfigAddParamSchema>;

export const HwSmsConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type HwSmsConfigAddResType = z.infer<typeof HwSmsConfigAddResSchema>;

export async function hwSmsConfigAdd(
    param: HwSmsConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<HwSmsConfigAddResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/hw_config_add", param, config);
    return parseResData(data, HwSmsConfigAddResSchema);
}

// 华为云短信配置删除
export const HwSmsConfigDelParamSchema = z.object({
    id: z.coerce.number(),
});
export type HwSmsConfigDelParamType = z.infer<typeof HwSmsConfigDelParamSchema>;

export const HwSmsConfigDelResSchema = z.object({
    num: z.string(),
});
export type HwSmsConfigDelResType = z.infer<typeof HwSmsConfigDelResSchema>;

export async function hwSmsConfigDel(
    param: HwSmsConfigDelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<HwSmsConfigDelResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/hw_config_del", param, config);
    return parseResData(data, HwSmsConfigDelResSchema);
}

// 华为云短信配置编辑
export const HwSmsConfigEditParamSchema = z.object({
    id: z.coerce.number(),
    name: z.string(),
    url: z.string(),
    app_key: z.string(),
    app_secret: z.string(),
    callback_key: z.string().optional(),
    limit: z.coerce.number().optional().nullable(),
});
export type HwSmsConfigEditParamType = z.infer<typeof HwSmsConfigEditParamSchema>;

export const HwSmsConfigEditResSchema = z.object({
    num: z.string(),
});
export type HwSmsConfigEditResType = z.infer<typeof HwSmsConfigEditResSchema>;

export async function hwSmsConfigEdit(
    param: HwSmsConfigEditParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<HwSmsConfigEditResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/hw_config_edit", param, config);
    return parseResData(data, HwSmsConfigEditResSchema);
}

// 华为云短信配置列表
export const HwSmsConfigListParamSchema = z.object({
    ids: z.array(z.coerce.number()).optional(),
});
export type HwSmsConfigListParamType = z.infer<typeof HwSmsConfigListParamSchema>;

export const HwSmsConfigItemSchema = z.object({
    id: z.coerce.number(),
    name: z.string(),
    url: z.string(),
    app_key: z.string(),
    app_secret: z.string(),
    hide_app_key: z.string(),
    callback_key: z.string(),
    callback_url: z.string(),
    limit: z.string(),
    change_time: UnixTimestampSchema,
    change_user_id: z.coerce.number(),
});
export type HwSmsConfigItemType = z.infer<typeof HwSmsConfigItemSchema>;

export const HwSmsConfigListResSchema = z.object({
    data: z.array(HwSmsConfigItemSchema),
});
export type HwSmsConfigListResType = z.infer<typeof HwSmsConfigListResSchema>;

export async function hwSmsConfigList(
    param: HwSmsConfigListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<HwSmsConfigListResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/hw_config_list", param, config);
    return parseResData(data, HwSmsConfigListResSchema);
}

// 华为云短信模板配置添加
export const HwSmsTplConfigAddParamSchema = z.object({
    name: z.string(),
    hw_config_id: z.coerce.number(),
    tpl_key: z.string(),
    signature: z.string(),
    sender: z.string(),
    template_id: z.string(),
    template_map: z.string(),
});
export type HwSmsTplConfigAddParamType = z.infer<typeof HwSmsTplConfigAddParamSchema>;

export const HwSmsTplConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type HwSmsTplConfigAddResType = z.infer<typeof HwSmsTplConfigAddResSchema>;

export async function hwSmsTplConfigAdd(
    param: HwSmsTplConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<HwSmsTplConfigAddResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/hw_tpl_config_add", param, config);
    return parseResData(data, HwSmsTplConfigAddResSchema);
}

// ===== 京东云短信配置 =====
// 京东云短信配置添加
export const JdSmsConfigAddParamSchema = z.object({
    name: z.string(),
    region: z.string(),
    access_key: z.string(),
    access_secret: z.string(),
    limit: z.coerce.number().optional().nullable(),
});
export type JdSmsConfigAddParamType = z.infer<typeof JdSmsConfigAddParamSchema>;

export const JdSmsConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type JdSmsConfigAddResType = z.infer<typeof JdSmsConfigAddResSchema>;

export async function jdSmsConfigAdd(
    param: JdSmsConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<JdSmsConfigAddResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/jd_config_add", param, config);
    return parseResData(data, JdSmsConfigAddResSchema);
}

// 京东云短信配置删除
export const JdSmsConfigDelParamSchema = z.object({
    id: z.coerce.number(),
});
export type JdSmsConfigDelParamType = z.infer<typeof JdSmsConfigDelParamSchema>;

export const JdSmsConfigDelResSchema = z.object({
    num: z.string(),
});
export type JdSmsConfigDelResType = z.infer<typeof JdSmsConfigDelResSchema>;

export async function jdSmsConfigDel(
    param: JdSmsConfigDelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<JdSmsConfigDelResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/jd_config_del", param, config);
    return parseResData(data, JdSmsConfigDelResSchema);
}

// 京东云短信配置编辑
export const JdSmsConfigEditParamSchema = z.object({
    id: z.coerce.number(),
    name: z.string(),
    region: z.string(),
    access_key: z.string(),
    access_secret: z.string(),
    limit: z.coerce.number().optional().nullable(),
});
export type JdSmsConfigEditParamType = z.infer<typeof JdSmsConfigEditParamSchema>;

export const JdSmsConfigEditResSchema = z.object({
    num: z.string(),
});
export type JdSmsConfigEditResType = z.infer<typeof JdSmsConfigEditResSchema>;

export async function jdSmsConfigEdit(
    param: JdSmsConfigEditParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<JdSmsConfigEditResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/jd_config_edit", param, config);
    return parseResData(data, JdSmsConfigEditResSchema);
}

// 京东云短信配置列表
export const JdSmsConfigListParamSchema = z.object({
    ids: z.array(z.coerce.number()).optional(),
});
export type JdSmsConfigListParamType = z.infer<typeof JdSmsConfigListParamSchema>;

export const JdSmsConfigItemSchema = z.object({
    id: z.coerce.number(),
    name: z.string(),
    region: z.string(),
    access_key: z.string(),
    access_secret: z.string(),
    hide_access_key: z.string(),
    limit: z.string(),
    change_time: UnixTimestampSchema,
    change_user_id: z.coerce.number(),
});
export type JdSmsConfigItemType = z.infer<typeof JdSmsConfigItemSchema>;

export const JdSmsConfigListResSchema = z.object({
    data: z.array(JdSmsConfigItemSchema),
});
export type JdSmsConfigListResType = z.infer<typeof JdSmsConfigListResSchema>;

export async function jdSmsConfigList(
    param: JdSmsConfigListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<JdSmsConfigListResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/jd_config_list", param, config);
    return parseResData(data, JdSmsConfigListResSchema);
}

// 京东云短信模板配置添加
export const JdSmsTplConfigAddParamSchema = z.object({
    name: z.string(),
    config_id: z.coerce.number(),
    tpl_key: z.string(),
    sign_id: z.string(),
    template_id: z.string(),
    template_map: z.string(),
});
export type JdSmsTplConfigAddParamType = z.infer<typeof JdSmsTplConfigAddParamSchema>;

export const JdSmsTplConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type JdSmsTplConfigAddResType = z.infer<typeof JdSmsTplConfigAddResSchema>;

export async function jdSmsTplConfigAdd(
    param: JdSmsTplConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<JdSmsTplConfigAddResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/jd_tpl_config_add", param, config);
    return parseResData(data, JdSmsTplConfigAddResSchema);
}

// ===== 网易云短信配置 =====
// 网易云短信配置添加
export const NeteaseSmsConfigAddParamSchema = z.object({
    name: z.string(),
    access_key: z.string(),
    access_secret: z.string(),
    limit: z.coerce.number().optional().nullable(),
});
export type NeteaseSmsConfigAddParamType = z.infer<typeof NeteaseSmsConfigAddParamSchema>;

export const NeteaseSmsConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type NeteaseSmsConfigAddResType = z.infer<typeof NeteaseSmsConfigAddResSchema>;

export async function neteaseSmsConfigAdd(
    param: NeteaseSmsConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<NeteaseSmsConfigAddResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/netease_config_add", param, config);
    return parseResData(data, NeteaseSmsConfigAddResSchema);
}

// 网易云短信配置删除
export const NeteaseSmsConfigDelParamSchema = z.object({
    id: z.coerce.number(),
});
export type NeteaseSmsConfigDelParamType = z.infer<typeof NeteaseSmsConfigDelParamSchema>;

export const NeteaseSmsConfigDelResSchema = z.object({
    num: z.string(),
});
export type NeteaseSmsConfigDelResType = z.infer<typeof NeteaseSmsConfigDelResSchema>;

export async function neteaseSmsConfigDel(
    param: NeteaseSmsConfigDelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<NeteaseSmsConfigDelResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/netease_config_del", param, config);
    return parseResData(data, NeteaseSmsConfigDelResSchema);
}

// 网易云短信配置编辑
export const NeteaseSmsConfigEditParamSchema = z.object({
    id: z.coerce.number(),
    name: z.string(),
    access_key: z.string(),
    access_secret: z.string(),
    limit: z.coerce.number().optional().nullable(),
});
export type NeteaseSmsConfigEditParamType = z.infer<typeof NeteaseSmsConfigEditParamSchema>;

export const NeteaseSmsConfigEditResSchema = z.object({
    num: z.string(),
});
export type NeteaseSmsConfigEditResType = z.infer<typeof NeteaseSmsConfigEditResSchema>;

export async function neteaseSmsConfigEdit(
    param: NeteaseSmsConfigEditParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<NeteaseSmsConfigEditResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/netease_config_edit", param, config);
    return parseResData(data, NeteaseSmsConfigEditResSchema);
}

// 网易云短信配置列表
export const NeteaseSmsConfigListParamSchema = z.object({
    ids: z.array(z.coerce.number()).optional(),
});
export type NeteaseSmsConfigListParamType = z.infer<typeof NeteaseSmsConfigListParamSchema>;

export const NeteaseSmsConfigItemSchema = z.object({
    id: z.coerce.number(),
    name: z.string(),
    access_key: z.string(),
    access_secret: z.string(),
    hide_access_key: z.string(),
    callback_url: z.string(),
    limit: z.string(),
    change_time: UnixTimestampSchema,
    change_user_id: z.coerce.number(),
});
export type NeteaseSmsConfigItemType = z.infer<typeof NeteaseSmsConfigItemSchema>;

export const NeteaseSmsConfigListResSchema = z.object({
    data: z.array(NeteaseSmsConfigItemSchema),
});
export type NeteaseSmsConfigListResType = z.infer<typeof NeteaseSmsConfigListResSchema>;

export async function neteaseSmsConfigList(
    param: NeteaseSmsConfigListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<NeteaseSmsConfigListResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/netease_config_list", param, config);
    return parseResData(data, NeteaseSmsConfigListResSchema);
}

// 网易云短信模板配置添加
export const NeteaseSmsTplConfigAddParamSchema = z.object({
    name: z.string(),
    config_id: z.coerce.number(),
    tpl_key: z.string(),
    template_id: z.string(),
    template_map: z.string(),
});
export type NeteaseSmsTplConfigAddParamType = z.infer<typeof NeteaseSmsTplConfigAddParamSchema>;

export const NeteaseSmsTplConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type NeteaseSmsTplConfigAddResType = z.infer<typeof NeteaseSmsTplConfigAddResSchema>;

export async function neteaseSmsTplConfigAdd(
    param: NeteaseSmsTplConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<NeteaseSmsTplConfigAddResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/netease_tpl_config_add", param, config);
    return parseResData(data, NeteaseSmsTplConfigAddResSchema);
}

// ===== 腾讯云短信配置 =====
// 腾讯云短信配置添加
export const TencentSmsConfigAddParamSchema = z.object({
    name: z.string(),
    region: z.string(),
    secret_id: z.string(),
    secret_key: z.string(),
    sms_app_id: z.string(),
    callback_key: z.string().optional(),
    limit: z.coerce.number().optional().nullable(),
});
export type TencentSmsConfigAddParamType = z.infer<typeof TencentSmsConfigAddParamSchema>;

export const TencentSmsConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type TencentSmsConfigAddResType = z.infer<typeof TencentSmsConfigAddResSchema>;

export async function tencentSmsConfigAdd(
    param: TencentSmsConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<TencentSmsConfigAddResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/tencent_config_add", param, config);
    return parseResData(data, TencentSmsConfigAddResSchema);
}

// 腾讯云短信配置删除
export const TencentSmsConfigDelParamSchema = z.object({
    id: z.coerce.number(),
});
export type TencentSmsConfigDelParamType = z.infer<typeof TencentSmsConfigDelParamSchema>;

export const TencentSmsConfigDelResSchema = z.object({
    num: z.string(),
});
export type TencentSmsConfigDelResType = z.infer<typeof TencentSmsConfigDelResSchema>;

export async function tencentSmsConfigDel(
    param: TencentSmsConfigDelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<TencentSmsConfigDelResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/tencent_config_del", param, config);
    return parseResData(data, TencentSmsConfigDelResSchema);
}

// 腾讯云短信配置编辑
export const TencentSmsConfigEditParamSchema = z.object({
    id: z.coerce.number(),
    name: z.string(),
    region: z.string(),
    secret_id: z.string(),
    secret_key: z.string(),
    sms_app_id: z.string(),
    callback_key: z.string().optional(),
    limit: z.coerce.number().optional().nullable(),
});
export type TencentSmsConfigEditParamType = z.infer<typeof TencentSmsConfigEditParamSchema>;

export const TencentSmsConfigEditResSchema = z.object({
    num: z.string(),
});
export type TencentSmsConfigEditResType = z.infer<typeof TencentSmsConfigEditResSchema>;

export async function tencentSmsConfigEdit(
    param: TencentSmsConfigEditParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<TencentSmsConfigEditResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/tencent_config_edit", param, config);
    return parseResData(data, TencentSmsConfigEditResSchema);
}

// 腾讯云短信配置列表
export const TencentSmsConfigListParamSchema = z.object({
    ids: z.array(z.coerce.number()).optional(),
});
export type TencentSmsConfigListParamType = z.infer<typeof TencentSmsConfigListParamSchema>;

export const TencentSmsConfigItemSchema = z.object({
    id: z.coerce.number(),
    name: z.string(),
    region: z.string(),
    secret_id: z.string(),
    secret_key: z.string(),
    hide_secret_id: z.string(),
    sms_app_id: z.string(),
    callback_key: z.string(),
    callback_url: z.string(),
    limit: z.string(),
    change_time: UnixTimestampSchema,
    change_user_id: z.coerce.number(),
});
export type TencentSmsConfigItemType = z.infer<typeof TencentSmsConfigItemSchema>;

export const TencentSmsConfigListResSchema = z.object({
    data: z.array(TencentSmsConfigItemSchema),
});
export type TencentSmsConfigListResType = z.infer<typeof TencentSmsConfigListResSchema>;

export async function tencentSmsConfigList(
    param: TencentSmsConfigListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<TencentSmsConfigListResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/tencent_config_list", param, config);
    return parseResData(data, TencentSmsConfigListResSchema);
}

// 腾讯云短信模板配置添加
export const TencentSmsTplConfigAddParamSchema = z.object({
    name: z.string(),
    config_id: z.coerce.number(),
    tpl_key: z.string(),
    sign_name: z.string(),
    template_id: z.string(),
    template_map: z.string(),
});
export type TencentSmsTplConfigAddParamType = z.infer<typeof TencentSmsTplConfigAddParamSchema>;

export const TencentSmsTplConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type TencentSmsTplConfigAddResType = z.infer<typeof TencentSmsTplConfigAddResSchema>;

export async function tencentSmsTplConfigAdd(
    param: TencentSmsTplConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<TencentSmsTplConfigAddResType>> {
    const { data } = await authApi().post("/api/system/sender/smser/tencent_tpl_config_add", param, config);
    return parseResData(data, TencentSmsTplConfigAddResSchema);
}

