import { authApi } from "@shared/lib/apis/api_auth";
import { cleanEmptyStringParams, parseResData } from "@shared/lib/apis/utils";
import { DictListSchema } from "@shared/types/apis-dict";
import { ApiResult } from "@shared/types/apis-rest";
import { BoolSchema, LimitParam, LimitRes, PageParam, PageRes, UnixTimestampSchema } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

// Sender Mailer APIs
export const SystemSenderMailerMapResSchema = z.object({
    config_type: DictListSchema,
    log_status: DictListSchema,
    log_type: DictListSchema,
    mail_body_status: DictListSchema,
    mail_config_type: DictListSchema,
    mail_send_status: DictListSchema,
});
export type SystemSenderMailerMapResType = z.infer<typeof SystemSenderMailerMapResSchema>;

export async function systemSenderMailerMapping(
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderMailerMapResType>> {
    const { data } = await authApi().post("/api/system/sender/mailer/mapping", {}, config);
    return parseResData(data, SystemSenderMailerMapResSchema);
}
export const SystemSenderMailerConfigListItemSchema = z.object({
    id: z.coerce.number(),
    add_time: UnixTimestampSchema,
    config_data: z.string(),
    config_type: z.string(),
    priority: z.coerce.number(),
});
export type SystemSenderMailerConfigListItemType = z.infer<typeof SystemSenderMailerConfigListItemSchema>;
// Mailer Config List - 根据文档应该没有参数，只返回配置列表
export const SystemSenderMailerConfigListResSchema = z.object({
    data: z.array(SystemSenderMailerConfigListItemSchema),
});
export type SystemSenderMailerConfigListResType = z.infer<typeof SystemSenderMailerConfigListResSchema>;

export async function systemSenderMailerConfigList(
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderMailerConfigListResType>> {
    const { data } = await authApi().post("/api/system/sender/mailer/config_list", {}, config);
    return parseResData(data, SystemSenderMailerConfigListResSchema);
}

// Add Mailer Config
export const SystemSenderMailerConfigAddParamSchema = z.object({
    priority: z.coerce.number(),
    config_type: z.coerce.number(),
    config_data: z.coerce.number(),
});
export type SystemSenderMailerConfigAddParamType = z.infer<typeof SystemSenderMailerConfigAddParamSchema>;

export const SystemSenderMailerConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type SystemSenderMailerConfigAddResType = z.infer<typeof SystemSenderMailerConfigAddResSchema>;

export async function systemSenderMailerConfigAdd(
    param: SystemSenderMailerConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderMailerConfigAddResType>> {
    const { data } = await authApi().post("/api/system/sender/mailer/config_add", param, config);
    return parseResData(data, SystemSenderMailerConfigAddResSchema);
}

// Delete Mailer Config
export const SystemSenderMailerConfigDelParamSchema = z.object({
    config_id: z.coerce.number(),
});
export type SystemSenderMailerConfigDelParamType = z.infer<typeof SystemSenderMailerConfigDelParamSchema>;

export async function systemSenderMailerConfigDel(
    param: SystemSenderMailerConfigDelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/sender/mailer/config_del", param, config);
    return data;
}

// SMTP Config APIs
export const SystemSenderMailerSmtpConfigListParamSchema = z.object({
    ids: z.array(z.coerce.number()).optional(),
});
export type SystemSenderMailerSmtpConfigListParamType = z.infer<typeof SystemSenderMailerSmtpConfigListParamSchema>;

export const SystemSenderMailerSmtpConfigItemSchema = z.object({
    id: z.coerce.number(),
    name: z.string(),
    host: z.string(),
    port: z.coerce.number(),
    timeout: z.coerce.number(),
    email: z.string(),
    user: z.string(),
    password: z.string(),
    hide_password: z.string(),
    hide_user: z.string(),
    tls_domain: z.string(),
    change_time: UnixTimestampSchema,
    change_user_id: z.coerce.number(),
});
export type SystemSenderMailerSmtpConfigItemType = z.infer<typeof SystemSenderMailerSmtpConfigItemSchema>;

export const SystemSenderMailerSmtpConfigListResSchema = z.object({
    data: z.array(SystemSenderMailerSmtpConfigItemSchema),
});
export type SystemSenderMailerSmtpConfigListResType = z.infer<typeof SystemSenderMailerSmtpConfigListResSchema>;

export async function systemSenderMailerSmtpConfigList(
    param: SystemSenderMailerSmtpConfigListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderMailerSmtpConfigListResType>> {
    const { data } = await authApi().post("/api/system/sender/mailer/smtp_config_list", param, config);
    return parseResData(data, SystemSenderMailerSmtpConfigListResSchema);
}

// Add SMTP Config
export const SystemSenderMailerSmtpConfigAddParamSchema = z.object({
    name: z.string(),
    host: z.string(),
    port: z.coerce.number(),
    timeout: z.coerce.number(),
    email: z.string(),
    user: z.string(),
    password: z.string(),
    tls_domain: z.string().optional(),
    branch_limit: z.coerce.number(),
});
export type SystemSenderMailerSmtpConfigAddParamType = z.infer<typeof SystemSenderMailerSmtpConfigAddParamSchema>;

export const SystemSenderMailerSmtpConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type SystemSenderMailerSmtpConfigAddResType = z.infer<typeof SystemSenderMailerSmtpConfigAddResSchema>;

export async function systemSenderMailerSmtpConfigAdd(
    param: SystemSenderMailerSmtpConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderMailerSmtpConfigAddResType>> {
    const { data } = await authApi().post("/api/system/sender/mailer/smtp_config_add", param, config);
    return parseResData(data, SystemSenderMailerSmtpConfigAddResSchema);
}

// Check SMTP Config
export const SystemSenderMailerSmtpConfigCheckParamSchema = z.object({
    host: z.string(),
    port: z.coerce.number(),
    timeout: z.coerce.number(),
    email: z.string(),
    user: z.string(),
    password: z.string(),
    tls_domain: z.string().optional(),
});
export type SystemSenderMailerSmtpConfigCheckParamType = z.infer<typeof SystemSenderMailerSmtpConfigCheckParamSchema>;

export const SystemSenderMailerSmtpConfigCheckResSchema = z.object({
    status: z.coerce.number(),
});
export type SystemSenderMailerSmtpConfigCheckResType = z.infer<typeof SystemSenderMailerSmtpConfigCheckResSchema>;

export async function systemSenderMailerSmtpConfigCheck(
    param: SystemSenderMailerSmtpConfigCheckParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderMailerSmtpConfigCheckResType>> {
    const { data } = await authApi().post("/api/system/sender/mailer/smtp_config_check", param, config);
    return parseResData(data, SystemSenderMailerSmtpConfigCheckResSchema);
}

// Edit SMTP Config
export const SystemSenderMailerSmtpConfigEditParamSchema = z.object({
    id: z.coerce.number(),
    name: z.string(),
    host: z.string(),
    port: z.coerce.number(),
    timeout: z.coerce.number(),
    email: z.string(),
    user: z.string(),
    password: z.string(),
    tls_domain: z.string().optional(),
    branch_limit: z.coerce.number(),
});
export type SystemSenderMailerSmtpConfigEditParamType = z.infer<typeof SystemSenderMailerSmtpConfigEditParamSchema>;

export const SystemSenderMailerSmtpConfigEditResSchema = z.object({
    num: z.coerce.number(),
});
export type SystemSenderMailerSmtpConfigEditResType = z.infer<typeof SystemSenderMailerSmtpConfigEditResSchema>;

export async function systemSenderMailerSmtpConfigEdit(
    param: SystemSenderMailerSmtpConfigEditParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderMailerSmtpConfigEditResType>> {
    const { data } = await authApi().post("/api/system/sender/mailer/smtp_config_edit", param, config);
    return parseResData(data, SystemSenderMailerSmtpConfigEditResSchema);
}

// Delete SMTP Config
export const SystemSenderMailerSmtpConfigDelParamSchema = z.object({
    id: z.coerce.number(),
});
export type SystemSenderMailerSmtpConfigDelParamType = z.infer<typeof SystemSenderMailerSmtpConfigDelParamSchema>;

export const SystemSenderMailerSmtpConfigDelResSchema = z.object({
    num: z.coerce.number(),
});
export type SystemSenderMailerSmtpConfigDelResType = z.infer<typeof SystemSenderMailerSmtpConfigDelResSchema>;

export async function systemSenderMailerSmtpConfigDel(
    param: SystemSenderMailerSmtpConfigDelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderMailerSmtpConfigDelResType>> {
    const { data } = await authApi().post("/api/system/sender/mailer/smtp_config_del", param, config);
    return parseResData(data, SystemSenderMailerSmtpConfigDelResSchema);
}

// Message APIs
export const SystemSenderMailerMessageListParamSchema = z.object({
    tpl_key: z.string().optional(),
    status: z.coerce.number().optional().nullable(),
    body_id: z.coerce.number().optional().nullable(),
    snid: z.string().optional(),
    to_mail: z.string().optional(),
    ...LimitParam,
});
export type SystemSenderMailerMessageListParamType = z.infer<typeof SystemSenderMailerMessageListParamSchema>;

export const SystemSenderMailerMessageItemSchema = z.object({
    id: z.coerce.number(),
    add_time: UnixTimestampSchema,
    app_id: z.coerce.number(),
    expected_time: UnixTimestampSchema,
    max_try_num: z.coerce.number(),
    now_send: z.coerce.number(),
    on_task: z.coerce.number(),
    body_status: z.coerce.number().nullable(),
    send_time: UnixTimestampSchema,
    snid: z.string(),
    status: z.coerce.number(),
    to_mail: z.string(),
    tpl_key: z.string(),
    try_num: z.string(),
});
export type SystemSenderMailerMessageItemType = z.infer<typeof SystemSenderMailerMessageItemSchema>;

export const SystemSenderMailerMessageListResSchema = z.object({
    data: z.array(SystemSenderMailerMessageItemSchema),
    ...LimitRes,
});
export type SystemSenderMailerMessageListResType = z.infer<typeof SystemSenderMailerMessageListResSchema>;

export async function systemSenderMailerMessageList(
    param: SystemSenderMailerMessageListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderMailerMessageListResType>> {
    const cleanedParam = cleanEmptyStringParams(param, ['tpl_key', 'snid', 'to_mail']);
    const { data } = await authApi().post("/api/system/sender/mailer/message_list", cleanedParam, config);
    return parseResData(data, SystemSenderMailerMessageListResSchema);
}

// Cancel Message
export const SystemSenderMailerMessageCancelParamSchema = z.object({
    message_id: z.coerce.number(),
});
export type SystemSenderMailerMessageCancelParamType = z.infer<typeof SystemSenderMailerMessageCancelParamSchema>;

export async function systemSenderMailerMessageCancel(
    param: SystemSenderMailerMessageCancelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/sender/mailer/message_cancel", param, config);
    return data;
}

// View Message
export const SystemSenderMailerMessageViewParamSchema = z.object({
    message_id: z.coerce.number(),
});
export type SystemSenderMailerMessageViewParamType = z.infer<typeof SystemSenderMailerMessageViewParamSchema>;

export const SystemSenderMailerMessageViewResSchema = z.object({
    body: z.string(),
});
export type SystemSenderMailerMessageViewResType = z.infer<typeof SystemSenderMailerMessageViewResSchema>;

export async function systemSenderMailerMessageView(
    param: SystemSenderMailerMessageViewParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderMailerMessageViewResType>> {
    const { data } = await authApi().post("/api/system/sender/mailer/message_view", param, config);
    return parseResData(data, SystemSenderMailerMessageViewResSchema);
}

// Message Logs
export const SystemSenderMailerMessageLogsParamSchema = z.object({
    message_id: z.coerce.number(),
    ...PageParam,
});
export type SystemSenderMailerMessageLogsParamType = z.infer<typeof SystemSenderMailerMessageLogsParamSchema>;

export const SystemSenderMailerMessageLogItemSchema = z.object({
    id: z.coerce.number(),
    app_id: z.coerce.number(),
    create_time: UnixTimestampSchema,
    executor_type: z.string(),
    log_type: z.string(),
    message: z.string(),
    sender_message_id: z.coerce.number(),
    sender_type: z.string(),
    status: z.string().optional(),
});
export type SystemSenderMailerMessageLogItemType = z.infer<typeof SystemSenderMailerMessageLogItemSchema>;

export const SystemSenderMailerMessageLogsResSchema = z.object({
    data: z.array(SystemSenderMailerMessageLogItemSchema),
    ...PageRes,
});
export type SystemSenderMailerMessageLogsResType = z.infer<typeof SystemSenderMailerMessageLogsResSchema>;

export async function systemSenderMailerMessageLogs(
    param: SystemSenderMailerMessageLogsParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderMailerMessageLogsResType>> {
    const { data } = await authApi().post("/api/system/sender/mailer/message_logs", param, config);
    return parseResData(data, SystemSenderMailerMessageLogsResSchema);
}

// SMTP Template Config APIs
export const SystemSenderMailerSmtpTplConfigAddParamSchema = z.object({
    smtp_config_id: z.coerce.number(),
    name: z.string(),
    tpl_key: z.string(),
    from_email: z.string(),
    reply_email: z.string(),
    subject_tpl_id: z.string(),
    body_tpl_id: z.string(),
});
export type SystemSenderMailerSmtpTplConfigAddParamType = z.infer<typeof SystemSenderMailerSmtpTplConfigAddParamSchema>;

export const SystemSenderMailerSmtpTplConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type SystemSenderMailerSmtpTplConfigAddResType = z.infer<typeof SystemSenderMailerSmtpTplConfigAddResSchema>;

export async function systemSenderMailerSmtpTplConfigAdd(
    param: SystemSenderMailerSmtpTplConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderMailerSmtpTplConfigAddResType>> {
    const { data } = await authApi().post("/api/system/sender/mailer/smtp_tpl_config_add", param, config);
    return parseResData(data, SystemSenderMailerSmtpTplConfigAddResSchema);
}

// Template Body APIs
export const SystemSenderMailerTplBodyAddParamSchema = z.object({
    tpl_id: z.string(),
    tpl_data: z.string(),
});
export type SystemSenderMailerTplBodyAddParamType = z.infer<typeof SystemSenderMailerTplBodyAddParamSchema>;

export const SystemSenderMailerTplBodyAddResSchema = z.object({
    id: z.coerce.number(),
});
export type SystemSenderMailerTplBodyAddResType = z.infer<typeof SystemSenderMailerTplBodyAddResSchema>;

export async function systemSenderMailerTplBodyAdd(
    param: SystemSenderMailerTplBodyAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderMailerTplBodyAddResType>> {
    const { data } = await authApi().post("/api/system/sender/mailer/tpl_body_add", param, config);
    return parseResData(data, SystemSenderMailerTplBodyAddResSchema);
}

// Delete Template Body
export const SystemSenderMailerTplBodyDelParamSchema = z.object({
    id: z.coerce.number(),
});
export type SystemSenderMailerTplBodyDelParamType = z.infer<typeof SystemSenderMailerTplBodyDelParamSchema>;

export async function systemSenderMailerTplBodyDel(
    param: SystemSenderMailerTplBodyDelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/sender/mailer/tpl_body_del", param, config);
    return data;
}

// Edit Template Body
export const SystemSenderMailerTplBodyEditParamSchema = z.object({
    id: z.coerce.number(),
    tpl_data: z.string(),
});
export type SystemSenderMailerTplBodyEditParamType = z.infer<typeof SystemSenderMailerTplBodyEditParamSchema>;

export async function systemSenderMailerTplBodyEdit(
    param: SystemSenderMailerTplBodyEditParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/system/sender/mailer/tpl_body_edit", param, config);
    return data;
}

// Template Body List
export const SystemSenderMailerTplBodyListParamSchema = z.object({
    id: z.coerce.number().optional().nullable(),
    tpl_id: z.string().optional(),
    tpl_id_like: z.string().optional().nullable(),
    ...PageParam,
});
export type SystemSenderMailerTplBodyListParamType = z.infer<typeof SystemSenderMailerTplBodyListParamSchema>;

export const SystemSenderMailerTplBodyItemSchema = z.object({
    id: z.coerce.number(),
    change_time: UnixTimestampSchema,
    change_user_id: z.coerce.number(),
    sender_type: z.string(),
    status: z.coerce.number(),
    tpl_data: z.string(),
    tpl_id: z.string(),
    user_id: z.coerce.number(),
});
export type SystemSenderMailerTplBodyItemType = z.infer<typeof SystemSenderMailerTplBodyItemSchema>;

export const SystemSenderMailerTplBodyListResSchema = z.object({
    data: z.array(SystemSenderMailerTplBodyItemSchema),
    ...PageRes,
});
export type SystemSenderMailerTplBodyListResType = z.infer<typeof SystemSenderMailerTplBodyListResSchema>;

export async function systemSenderMailerTplBodyList(
    param: SystemSenderMailerTplBodyListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderMailerTplBodyListResType>> {
    const cleanedParam = cleanEmptyStringParams(param, ['tpl_id', 'tpl_id_like']);
    const { data } = await authApi().post("/api/system/sender/mailer/tpl_body_list", cleanedParam, config);
    return parseResData(data, SystemSenderMailerTplBodyListResSchema);
}

// Template Config APIs
export const SystemSenderMailerTplConfigDeleteParamSchema = z.object({
    tpl_config_id: z.coerce.number(),
});
export type SystemSenderMailerTplConfigDeleteParamType = z.infer<typeof SystemSenderMailerTplConfigDeleteParamSchema>;

export const SystemSenderMailerTplConfigDeleteResSchema = z.object({
    num: z.string(),
});
export type SystemSenderMailerTplConfigDeleteResType = z.infer<typeof SystemSenderMailerTplConfigDeleteResSchema>;

export async function systemSenderMailerTplConfigDelete(
    param: SystemSenderMailerTplConfigDeleteParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderMailerTplConfigDeleteResType>> {
    const { data } = await authApi().post("/api/system/sender/mailer/tpl_config_delete", param, config);
    return parseResData(data, SystemSenderMailerTplConfigDeleteResSchema);
}

// Template Config List
export const SystemSenderMailerTplConfigListParamSchema = z.object({
    id: z.coerce.number().optional().nullable(),
    tpl: z.string().optional(),
    app_info: BoolSchema.optional(),
    ...PageParam,
});
export type SystemSenderMailerTplConfigListParamType = z.infer<typeof SystemSenderMailerTplConfigListParamSchema>;

export const SystemSenderMailerTplConfigItemSchema = z.object({
    id: z.coerce.number(),
    app_client_id: z.coerce.number().optional(),
    app_id: z.coerce.number(),
    app_name: z.string().optional(),
    change_time: UnixTimestampSchema,
    change_user_id: z.coerce.number(),
    config_data: z.object({
        body_tpl_id: z.string(),
        from_email: z.string(),
        reply_email: z.string().optional(),
        subject_tpl_id: z.string(),
    }),
    name: z.string(),
    setting_id: z.coerce.number(),
    setting_key: z.string(),
    setting_name: z.string(),
    tpl_key: z.string(),
    user_id: z.coerce.number(),
});
export type SystemSenderMailerTplConfigItemType = z.infer<typeof SystemSenderMailerTplConfigItemSchema>;

export const SystemSenderMailerTplConfigListResSchema = z.object({
    data: z.array(SystemSenderMailerTplConfigItemSchema),
    ...PageRes,
});
export type SystemSenderMailerTplConfigListResType = z.infer<typeof SystemSenderMailerTplConfigListResSchema>;

export async function systemSenderMailerTplConfigList(
    param: SystemSenderMailerTplConfigListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<SystemSenderMailerTplConfigListResType>> {
    const cleanedParam = cleanEmptyStringParams(param, ['tpl']);
    const { data } = await authApi().post("/api/system/sender/mailer/tpl_config_list", cleanedParam, config);
    return parseResData(data, SystemSenderMailerTplConfigListResSchema);
}


