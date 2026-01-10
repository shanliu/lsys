import { authApi } from "@shared/lib/apis/api_auth";
import { cleanEmptyStringParams, parseResData } from "@shared/lib/apis/utils";
import { DictListSchema } from "@shared/types/apis-dict";
import { ApiResult } from "@shared/types/apis-rest";
import { BoolSchema, LimitParam, LimitResSchema, PageParam, PageResSchema, UnixTimestampSchema } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

// User Sender SMS APIs
export const UserSenderSmsMapResSchema = z.object({
    config_type: DictListSchema,
    log_status: DictListSchema,
    log_type: DictListSchema,
    sms_body_status: DictListSchema,
    sms_config_type: DictListSchema,
    sms_send_status: DictListSchema,
});
export type UserSenderSmsMapResType = z.infer<typeof UserSenderSmsMapResSchema>;

export async function userSenderSmsMapping(
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderSmsMapResType>> {
    const { data } = await authApi().post("/api/user/app_sender/smser/mapping", {}, config);
    return parseResData(data, UserSenderSmsMapResSchema);
}

// SMS Config List
export const UserSenderSmsConfigListParamSchema = z.object({
    id: z.coerce.number().optional().nullable(),
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
});
export type UserSenderSmsConfigListParamType = z.infer<typeof UserSenderSmsConfigListParamSchema>;

export const UserSenderSmsConfigItemSchema = z.object({
    add_time: UnixTimestampSchema,
    app_id: z.coerce.number(),
    config_data: z.any(),
    config_type: z.string(),
    id: z.coerce.number(),
    priority: z.coerce.number(),
});
export type UserSenderSmsConfigItemType = z.infer<typeof UserSenderSmsConfigItemSchema>;

export const UserSenderSmsConfigListResSchema = z.object({
    data: z.array(UserSenderSmsConfigItemSchema),
});
export type UserSenderSmsConfigListResType = z.infer<typeof UserSenderSmsConfigListResSchema>;

export async function userSenderSmsConfigList(
    param: UserSenderSmsConfigListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderSmsConfigListResType>> {
    const { data } = await authApi().post("/api/user/app_sender/smser/config_list", param, config);
    return parseResData(data, UserSenderSmsConfigListResSchema);
}

// Add SMS Config
export const UserSenderSmsConfigAddParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    priority: z.coerce.number().min(1, "优先级必须大于0"),
    config_type: z.coerce.number().min(1, "配置类型必须大于0"),
    config_data: z.any(),
});
export type UserSenderSmsConfigAddParamType = z.infer<typeof UserSenderSmsConfigAddParamSchema>;

export const UserSenderSmsConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type UserSenderSmsConfigAddResType = z.infer<typeof UserSenderSmsConfigAddResSchema>;

export async function userSenderSmsConfigAdd(
    param: UserSenderSmsConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderSmsConfigAddResType>> {
    const { data } = await authApi().post("/api/user/app_sender/smser/config_add", param, config);
    return parseResData(data, UserSenderSmsConfigAddResSchema);
}

// Delete SMS Config
export const UserSenderSmsConfigDelParamSchema = z.object({
    config_id: z.coerce.number().min(1, "配置ID必须大于0"),
});
export type UserSenderSmsConfigDelParamType = z.infer<typeof UserSenderSmsConfigDelParamSchema>;

export async function userSenderSmsConfigDel(
    param: UserSenderSmsConfigDelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app_sender/smser/config_del", param, config);
    return data;
}

// Send SMS Message
export const UserSenderSmsMessageSendParamSchema = z.object({
    mobile: z.array(z.string()),
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    tpl_key: z.string().min(1, "模板标识不能为空"),
    data: z.record(z.any()),
    send_time: z.string().optional().nullable(),
    max_try: z.coerce.number().optional().nullable(),
});
export type UserSenderSmsMessageSendParamType = z.infer<typeof UserSenderSmsMessageSendParamSchema>;

export async function userSenderSmsMessageSend(
    param: UserSenderSmsMessageSendParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app_sender/smser/message_send", param, config);
    return data;
}

// SMS Message List
export const UserSenderSmsMessageListParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    tpl_key: z.string().optional(),
    status: z.coerce.number().nullable().optional(),
    body_id: z.coerce.number().optional().nullable(),
    snid: z.coerce.number().optional().nullable(),
    to_mobile: z.string().optional(),
    ...LimitParam,
});
export type UserSenderSmsMessageListParamType = z.infer<typeof UserSenderSmsMessageListParamSchema>;

export const UserSenderSmsMessageItemSchema = z.object({
    add_time: UnixTimestampSchema,
    app_id: z.coerce.number(),
    expected_time: UnixTimestampSchema,
    id: z.coerce.number(),
    max_try_num: z.coerce.number(),
    mobile: z.string(),
    now_send: BoolSchema,
    on_task: BoolSchema,
    body_status: z.coerce.number().nullable(),
    send_time: UnixTimestampSchema,
    snid: z.coerce.number(),
    status: z.coerce.number(),
    tpl_key: z.string(),
    try_num: z.coerce.number(),
});
export type UserSenderSmsMessageItemType = z.infer<typeof UserSenderSmsMessageItemSchema>;

export const UserSenderSmsMessageListResSchema = z.object({
    data: z.array(UserSenderSmsMessageItemSchema),
    ...LimitResSchema,
});
export type UserSenderSmsMessageListResType = z.infer<typeof UserSenderSmsMessageListResSchema>;

export async function userSenderSmsMessageList(
    param: UserSenderSmsMessageListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderSmsMessageListResType>> {
    const cleanedParam = cleanEmptyStringParams(param, ['tpl_key', 'to_mobile']);
    const { data } = await authApi().post("/api/user/app_sender/smser/message_list", cleanedParam, config);
    return parseResData(data, UserSenderSmsMessageListResSchema);
}

// Cancel SMS Message
export const UserSenderSmsMessageCancelParamSchema = z.object({
    message_id: z.coerce.number().min(1, "消息ID不能为空"),
});
export type UserSenderSmsMessageCancelParamType = z.infer<typeof UserSenderSmsMessageCancelParamSchema>;

export async function userSenderSmsMessageCancel(
    param: UserSenderSmsMessageCancelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app_sender/smser/message_cancel", param, config);
    return data;
}

// View SMS Message
export const UserSenderSmsMessageViewParamSchema = z.object({
    message_id: z.coerce.number().min(1, "消息ID不能为空"),
});
export type UserSenderSmsMessageViewParamType = z.infer<typeof UserSenderSmsMessageViewParamSchema>;

export const UserSenderSmsMessageViewResSchema = z.object({
    body: z.string(),
});
export type UserSenderSmsMessageViewResType = z.infer<typeof UserSenderSmsMessageViewResSchema>;

export async function userSenderSmsMessageView(
    param: UserSenderSmsMessageViewParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderSmsMessageViewResType>> {
    const { data } = await authApi().post("/api/user/app_sender/smser/message_view", param, config);
    return parseResData(data, UserSenderSmsMessageViewResSchema);
}

// SMS Message Logs
export const UserSenderSmsMessageLogsParamSchema = z.object({
    message_id: z.coerce.number().min(1, "消息ID不能为空"),
    ...PageParam,
});
export type UserSenderSmsMessageLogsParamType = z.infer<typeof UserSenderSmsMessageLogsParamSchema>;

export const UserSenderSmsMessageLogItemSchema = z.object({
    app_id: z.coerce.number(),
    create_time: UnixTimestampSchema,
    executor_type: z.string(),
    id: z.coerce.number(),
    log_type: z.coerce.number(),
    message: z.string(),
    sender_message_id: z.coerce.number(),
    sender_type: z.coerce.number(),
    status: z.coerce.number(),
});
export type UserSenderSmsMessageLogItemType = z.infer<typeof UserSenderSmsMessageLogItemSchema>;

export const UserSenderSmsMessageLogsResSchema = z.object({
    data: z.array(UserSenderSmsMessageLogItemSchema),
    ...PageResSchema,
});
export type UserSenderSmsMessageLogsResType = z.infer<typeof UserSenderSmsMessageLogsResSchema>;

export async function userSenderSmsMessageLogs(
    param: UserSenderSmsMessageLogsParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderSmsMessageLogsResType>> {
    const { data } = await authApi().post("/api/user/app_sender/smser/message_logs", param, config);
    return parseResData(data, UserSenderSmsMessageLogsResSchema);
}

export const UserSenderSmsNotifyGetConfigItemSchema = z.object({
    app_id: z.coerce.number(),
    app_name: z.string(),
    call_url: z.string().nullable(),
    change_time: UnixTimestampSchema.nullable().nullable(),
    change_user_id: z.coerce.number().nullable(),
});
export type UserSenderSmsNotifyGetConfigItemType = z.infer<typeof UserSenderSmsNotifyGetConfigItemSchema>;
// Get/Set Notify Config
export const UserSenderSmsNotifyGetConfigResSchema = z.object({
    data: z.array(UserSenderSmsNotifyGetConfigItemSchema),
});
export type UserSenderSmsNotifyGetConfigResType = z.infer<typeof UserSenderSmsNotifyGetConfigResSchema>;

export async function userSenderSmsNotifyGetConfig(
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderSmsNotifyGetConfigResType>> {
    const { data } = await authApi().post("/api/user/app_sender/smser/notify_get_config", {}, config);
    return parseResData(data, UserSenderSmsNotifyGetConfigResSchema);
}

export const UserSenderSmsNotifySetConfigParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    url: z.string().min(1, "回调URL不能为空"),
});
export type UserSenderSmsNotifySetConfigParamType = z.infer<typeof UserSenderSmsNotifySetConfigParamSchema>;

export async function userSenderSmsNotifySetConfig(
    param: UserSenderSmsNotifySetConfigParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app_sender/smser/notify_set_config", param, config);
    return data;
}

// Template Config List
export const UserSenderSmsTplConfigListParamSchema = z.object({
    id: z.coerce.number().optional().nullable(),
    app_id: z.coerce.number().optional().nullable(),
    tpl: z.string().optional(),
    like_tpl: z.string().optional(),
    app_info: BoolSchema.optional(),
    ...PageParam,
});
export type UserSenderSmsTplConfigListParamType = z.infer<typeof UserSenderSmsTplConfigListParamSchema>;

export const UserSenderSmsTplConfigItemSchema = z.object({
    app_client_id: z.coerce.number().optional(),
    app_id: z.coerce.number(),
    app_name: z.string().optional(),
    change_time: UnixTimestampSchema,
    change_user_id: z.coerce.number(),
    config_data: z.object({
        sign_name: z.string().optional(),
        template_id: z.string().optional().nullable(),
        template_map: z.string().optional(),
        aliyun_sms_tpl: z.string().optional(),
        aliyun_sign_name: z.string().optional(),
        signature: z.string().optional(),
        sender: z.string().optional(),
    }).optional(),
    id: z.coerce.number(),
    name: z.string(),
    setting_id: z.coerce.number(),
    setting_key: z.string(),
    setting_name: z.string(),
    tpl_key: z.string(),
    user_id: z.coerce.number(),
});
export type UserSenderSmsTplConfigItemType = z.infer<typeof UserSenderSmsTplConfigItemSchema>;

export const UserSenderSmsTplConfigListResSchema = z.object({
    data: z.array(UserSenderSmsTplConfigItemSchema),
    ...PageResSchema,
});
export type UserSenderSmsTplConfigListResType = z.infer<typeof UserSenderSmsTplConfigListResSchema>;

export async function userSenderSmsTplConfigList(
    param: UserSenderSmsTplConfigListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderSmsTplConfigListResType>> {
    const cleanedParam = cleanEmptyStringParams(param, ['tpl', 'like_tpl']);
    const { data } = await authApi().post("/api/user/app_sender/smser/tpl_config_list", cleanedParam, config);
    return parseResData(data, UserSenderSmsTplConfigListResSchema);
}

// Delete Template Config
export const UserSenderSmsTplConfigDelParamSchema = z.object({
    config_id: z.coerce.number().min(1, "配置ID必须大于0"),
});
export type UserSenderSmsTplConfigDelParamType = z.infer<typeof UserSenderSmsTplConfigDelParamSchema>;

export async function userSenderSmsTplConfigDel(
    param: UserSenderSmsTplConfigDelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult> {
    const { data } = await authApi().post("/api/user/app_sender/smser/tpl_config_del", param, config);
    return data;
}

// Aliyun SMS Config
export const UserSenderSmsAliConfigListParamSchema = z.object({
    ids: z.array(z.coerce.number()).nullable(),
});
export type UserSenderSmsAliConfigListParamType = z.infer<typeof UserSenderSmsAliConfigListParamSchema>;

export const UserSenderSmsAliConfigItemSchema = z.object({
    app_id: z.union([z.coerce.number(), z.string()]),
    id: z.coerce.number(),
    name: z.string(),
});
export type UserSenderSmsAliConfigItemType = z.infer<typeof UserSenderSmsAliConfigItemSchema>;

export const UserSenderSmsAliConfigListResSchema = z.object({
    data: z.array(UserSenderSmsAliConfigItemSchema),
});
export type UserSenderSmsAliConfigListResType = z.infer<typeof UserSenderSmsAliConfigListResSchema>;

export async function userSenderSmsAliConfigList(
    param: UserSenderSmsAliConfigListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderSmsAliConfigListResType>> {
    const { data } = await authApi().post("/api/user/app_sender/smser/ali_config_list", param, config);
    return parseResData(data, UserSenderSmsAliConfigListResSchema);
}

export const UserSenderSmsAliAppConfigAddParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    name: z.string().min(1, "配置名称不能为空"),
    ali_config_id: z.coerce.number().min(1, "阿里云配置ID必须大于0"),
    tpl_key: z.string().min(1, "模板标识不能为空"),
    aliyun_sms_tpl: z.string().min(1, "阿里云短信模板ID不能为空"),
    aliyun_sign_name: z.string().min(1, "阿里云短信签名不能为空"),
});
export type UserSenderSmsAliAppConfigAddParamType = z.infer<typeof UserSenderSmsAliAppConfigAddParamSchema>;

export const UserSenderSmsAliAppConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type UserSenderSmsAliAppConfigAddResType = z.infer<typeof UserSenderSmsAliAppConfigAddResSchema>;

export async function userSenderSmsAliAppConfigAdd(
    param: UserSenderSmsAliAppConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderSmsAliAppConfigAddResType>> {
    const { data } = await authApi().post("/api/user/app_sender/smser/ali_app_config_add", param, config);
    return parseResData(data, UserSenderSmsAliAppConfigAddResSchema);
}

// Tencent Cloud SMS Config
export const UserSenderSmsTenConfigListParamSchema = z.object({
    ids: z.array(z.coerce.number()).nullable(),
});
export type UserSenderSmsTenConfigListParamType = z.infer<typeof UserSenderSmsTenConfigListParamSchema>;

export const UserSenderSmsTenConfigItemSchema = z.object({
    app_id: z.union([z.coerce.number(), z.string()]),
    id: z.coerce.number(),
    name: z.string(),
});
export type UserSenderSmsTenConfigItemType = z.infer<typeof UserSenderSmsTenConfigItemSchema>;

export const UserSenderSmsTenConfigListResSchema = z.object({
    data: z.array(UserSenderSmsTenConfigItemSchema),
});
export type UserSenderSmsTenConfigListResType = z.infer<typeof UserSenderSmsTenConfigListResSchema>;

export async function userSenderSmsTenConfigList(
    param: UserSenderSmsTenConfigListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderSmsTenConfigListResType>> {
    const { data } = await authApi().post("/api/user/app_sender/smser/ten_config_list", param, config);
    return parseResData(data, UserSenderSmsTenConfigListResSchema);
}

export const UserSenderSmsTenAppConfigAddParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    name: z.string().min(1, "配置名称不能为空"),
    config_id: z.coerce.number().min(1, "腾讯云配置ID必须大于0"),
    tpl_key: z.string().min(1, "模板标识不能为空"),
    sign_name: z.string().min(1, "短信签名不能为空"),
    template_id: z.string().min(1, "模板ID不能为空"),
    template_map: z.string().min(1, "模板参数映射不能为空"),
});
export type UserSenderSmsTenAppConfigAddParamType = z.infer<typeof UserSenderSmsTenAppConfigAddParamSchema>;

export const UserSenderSmsTenAppConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type UserSenderSmsTenAppConfigAddResType = z.infer<typeof UserSenderSmsTenAppConfigAddResSchema>;

export async function userSenderSmsTenAppConfigAdd(
    param: UserSenderSmsTenAppConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderSmsTenAppConfigAddResType>> {
    const { data } = await authApi().post("/api/user/app_sender/smser/ten_app_config_add", param, config);
    return parseResData(data, UserSenderSmsTenAppConfigAddResSchema);
}

// Huawei Cloud SMS Config
export const UserSenderSmsHwConfigListParamSchema = z.object({
    ids: z.array(z.coerce.number()).nullable(),
});
export type UserSenderSmsHwConfigListParamType = z.infer<typeof UserSenderSmsHwConfigListParamSchema>;

export const UserSenderSmsHwConfigItemSchema = z.object({
    app_id: z.union([z.coerce.number(), z.string()]),
    id: z.coerce.number(),
    name: z.string(),
});
export type UserSenderSmsHwConfigItemType = z.infer<typeof UserSenderSmsHwConfigItemSchema>;

export const UserSenderSmsHwConfigListResSchema = z.object({
    data: z.array(UserSenderSmsHwConfigItemSchema),
});
export type UserSenderSmsHwConfigListResType = z.infer<typeof UserSenderSmsHwConfigListResSchema>;

export async function userSenderSmsHwConfigList(
    param: UserSenderSmsHwConfigListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderSmsHwConfigListResType>> {
    const { data } = await authApi().post("/api/user/app_sender/smser/hw_config_list", param, config);
    return parseResData(data, UserSenderSmsHwConfigListResSchema);
}

export const UserSenderSmsHwAppConfigAddParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    name: z.string().min(1, "配置名称不能为空"),
    hw_config_id: z.coerce.number().min(1, "华为云配置ID必须大于0"),
    tpl_key: z.string().min(1, "模板标识不能为空"),
    signature: z.string().min(1, "短信签名不能为空"),
    sender: z.string().min(1, "发送者不能为空"),
    template_id: z.string().min(1, "模板ID不能为空"),
    template_map: z.string().min(1, "模板参数映射不能为空"),
});
export type UserSenderSmsHwAppConfigAddParamType = z.infer<typeof UserSenderSmsHwAppConfigAddParamSchema>;

export const UserSenderSmsHwAppConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type UserSenderSmsHwAppConfigAddResType = z.infer<typeof UserSenderSmsHwAppConfigAddResSchema>;

export async function userSenderSmsHwAppConfigAdd(
    param: UserSenderSmsHwAppConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderSmsHwAppConfigAddResType>> {
    const { data } = await authApi().post("/api/user/app_sender/smser/hw_app_config_add", param, config);
    return parseResData(data, UserSenderSmsHwAppConfigAddResSchema);
}

// JD Cloud SMS Config
export const UserSenderSmsJdConfigListParamSchema = z.object({
    ids: z.array(z.coerce.number()).nullable(),
});
export type UserSenderSmsJdConfigListParamType = z.infer<typeof UserSenderSmsJdConfigListParamSchema>;

export const UserSenderSmsJdConfigItemSchema = z.object({
    app_id: z.union([z.coerce.number(), z.string()]),
    id: z.coerce.number(),
    name: z.string(),
});
export type UserSenderSmsJdConfigItemType = z.infer<typeof UserSenderSmsJdConfigItemSchema>;

export const UserSenderSmsJdConfigListResSchema = z.object({
    data: z.array(UserSenderSmsJdConfigItemSchema),
});
export type UserSenderSmsJdConfigListResType = z.infer<typeof UserSenderSmsJdConfigListResSchema>;

export async function userSenderSmsJdConfigList(
    param: UserSenderSmsJdConfigListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderSmsJdConfigListResType>> {
    const { data } = await authApi().post("/api/user/app_sender/smser/jd_config_list", param, config);
    return parseResData(data, UserSenderSmsJdConfigListResSchema);
}

export const UserSenderSmsJdAppConfigAddParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    name: z.string().min(1, "配置名称不能为空"),
    config_id: z.coerce.number().min(1, "京东云配置ID必须大于0"),
    tpl_key: z.string().min(1, "模板标识不能为空"),
    sign_id: z.coerce.number().min(1, "签名ID不能为空"),
    template_id: z.string().min(1, "模板ID不能为空"),
    template_map: z.string().min(1, "模板参数映射不能为空"),
});
export type UserSenderSmsJdAppConfigAddParamType = z.infer<typeof UserSenderSmsJdAppConfigAddParamSchema>;

export const UserSenderSmsJdAppConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type UserSenderSmsJdAppConfigAddResType = z.infer<typeof UserSenderSmsJdAppConfigAddResSchema>;

export async function userSenderSmsJdAppConfigAdd(
    param: UserSenderSmsJdAppConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderSmsJdAppConfigAddResType>> {
    const { data } = await authApi().post("/api/user/app_sender/smser/jd_app_config_add", param, config);
    return parseResData(data, UserSenderSmsJdAppConfigAddResSchema);
}

// NetEase Cloud SMS Config
export const UserSenderSmsNeteaseConfigListParamSchema = z.object({
    ids: z.array(z.coerce.number()).nullable(),
});
export type UserSenderSmsNeteaseConfigListParamType = z.infer<typeof UserSenderSmsNeteaseConfigListParamSchema>;

export const UserSenderSmsNeteaseConfigItemSchema = z.object({
    app_id: z.union([z.coerce.number(), z.string()]),
    id: z.coerce.number(),
    name: z.string(),
});
export type UserSenderSmsNeteaseConfigItemType = z.infer<typeof UserSenderSmsNeteaseConfigItemSchema>;

export const UserSenderSmsNeteaseConfigListResSchema = z.object({
    data: z.array(UserSenderSmsNeteaseConfigItemSchema),
});
export type UserSenderSmsNeteaseConfigListResType = z.infer<typeof UserSenderSmsNeteaseConfigListResSchema>;

export async function userSenderSmsNeteaseConfigList(
    param: UserSenderSmsNeteaseConfigListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderSmsNeteaseConfigListResType>> {
    const { data } = await authApi().post("/api/user/app_sender/smser/netease_config_list", param, config);
    return parseResData(data, UserSenderSmsNeteaseConfigListResSchema);
}

export const UserSenderSmsNeteaseAppConfigAddParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    name: z.string().min(1, "配置名称不能为空"),
    config_id: z.coerce.number().min(1, "网易云配置ID必须大于0"),
    tpl_key: z.string().min(1, "模板标识不能为空"),
    template_id: z.string().min(1, "模板ID不能为空"),
    template_map: z.string().min(1, "模板参数映射不能为空"),
});
export type UserSenderSmsNeteaseAppConfigAddParamType = z.infer<typeof UserSenderSmsNeteaseAppConfigAddParamSchema>;

export const UserSenderSmsNeteaseAppConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type UserSenderSmsNeteaseAppConfigAddResType = z.infer<typeof UserSenderSmsNeteaseAppConfigAddResSchema>;

export async function userSenderSmsNeteaseAppConfigAdd(
    param: UserSenderSmsNeteaseAppConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderSmsNeteaseAppConfigAddResType>> {
    const { data } = await authApi().post("/api/user/app_sender/smser/netease_app_config_add", param, config);
    return parseResData(data, UserSenderSmsNeteaseAppConfigAddResSchema);
}

// Cloopen SMS Config
export const UserSenderSmsCLoopenConfigListParamSchema = z.object({
    ids: z.array(z.coerce.number()).nullable(),
});
export type UserSenderSmsCLoopenConfigListParamType = z.infer<typeof UserSenderSmsCLoopenConfigListParamSchema>;

export const UserSenderSmsCLoopenConfigItemSchema = z.object({
    app_id: z.union([z.coerce.number(), z.string()]),
    id: z.coerce.number(),
    name: z.string(),
});
export type UserSenderSmsCLoopenConfigItemType = z.infer<typeof UserSenderSmsCLoopenConfigItemSchema>;

export const UserSenderSmsCLoopenConfigListResSchema = z.object({
    data: z.array(UserSenderSmsCLoopenConfigItemSchema),
});
export type UserSenderSmsCLoopenConfigListResType = z.infer<typeof UserSenderSmsCLoopenConfigListResSchema>;

export async function userSenderSmsCLoopenConfigList(
    param: UserSenderSmsCLoopenConfigListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderSmsCLoopenConfigListResType>> {
    const { data } = await authApi().post("/api/user/app_sender/smser/cloopen_config_list", param, config);
    return parseResData(data, UserSenderSmsCLoopenConfigListResSchema);
}

export const UserSenderSmsCLoopenAppConfigAddParamSchema = z.object({
    app_id: z.coerce.number().min(1, "应用ID必须大于0"),
    name: z.string().min(1, "配置名称不能为空"),
    config_id: z.coerce.number().min(1, "容联云配置ID必须大于0"),
    tpl_key: z.string().min(1, "模板标识不能为空"),
    template_id: z.string().min(1, "容联云短信模板ID不能为空"),
    template_map: z.string().min(1, "模板参数映射不能为空"),
});
export type UserSenderSmsCLoopenAppConfigAddParamType = z.infer<typeof UserSenderSmsCLoopenAppConfigAddParamSchema>;

export const UserSenderSmsCLoopenAppConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type UserSenderSmsCLoopenAppConfigAddResType = z.infer<typeof UserSenderSmsCLoopenAppConfigAddResSchema>;

export async function userSenderSmsCLoopenAppConfigAdd(
    param: UserSenderSmsCLoopenAppConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderSmsCLoopenAppConfigAddResType>> {
    const { data } = await authApi().post("/api/user/app_sender/smser/cloopen_app_config_add", param, config);
    return parseResData(data, UserSenderSmsCLoopenAppConfigAddResSchema);
}

