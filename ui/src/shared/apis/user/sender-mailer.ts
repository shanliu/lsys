import { authApi } from "@shared/lib/apis/api_auth";
import { cleanEmptyStringParams, parseResData } from "@shared/lib/apis/utils";
import { DictListSchema } from "@shared/types/apis-dict";
import { ApiResult } from "@shared/types/apis-rest";
import { BoolSchema, LimitParam, LimitRes, PageParam, PageRes, UnixTimestampSchema } from "@shared/types/base-schema";
import { AxiosRequestConfig } from "axios";
import z from "zod";

// 122. 添加邮件发送配置
export const UserSenderMailerConfigAddParamSchema = z.object({
    app_id: z.coerce.number(),
    priority: z.coerce.number(),
    config_type: z.coerce.number(),
    config_data: z.any(),
});
export type UserSenderMailerConfigAddParamType = z.infer<typeof UserSenderMailerConfigAddParamSchema>;

export const UserSenderMailerConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type UserSenderMailerConfigAddResType = z.infer<typeof UserSenderMailerConfigAddResSchema>;

export const userSenderMailerConfigAdd = async (
    param: UserSenderMailerConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderMailerConfigAddResType>> => {
    const { data } = await authApi().post('/api/user/app_sender/mailer/config_add', param, config);
    return data;
};

// 123. 删除邮件发送配置
export const UserSenderMailerConfigDelParamSchema = z.object({
    config_id: z.coerce.number(),
});
export type UserSenderMailerConfigDelParamType = z.infer<typeof UserSenderMailerConfigDelParamSchema>;

export const UserSenderMailerConfigDelResSchema = z.object({});
export type UserSenderMailerConfigDelResType = z.infer<typeof UserSenderMailerConfigDelResSchema>;

export const userSenderMailerConfigDel = async (
    param: UserSenderMailerConfigDelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderMailerConfigDelResType>> => {
    const { data } = await authApi().post('/api/user/app_sender/mailer/config_del', param, config);
    return data;
};

// 124. 获取邮件发送配置列表
export const UserSenderMailerConfigListParamSchema = z.object({
    id: z.coerce.number().nullable().optional(),
    app_id: z.coerce.number(),
});
export type UserSenderMailerConfigListParamType = z.infer<typeof UserSenderMailerConfigListParamSchema>;

export const UserSenderMailerConfigItemSchema = z.object({
    add_time: UnixTimestampSchema,
    app_id: z.coerce.number(),
    config_data: z.any(),
    config_type: z.string(),
    id: z.coerce.number(),
    priority: z.string(),
});
export type UserSenderMailerConfigItem = z.infer<typeof UserSenderMailerConfigItemSchema>;

export const UserSenderMailerConfigListResSchema = z.object({
    data: z.array(UserSenderMailerConfigItemSchema),
});
export type UserSenderMailerConfigListResType = z.infer<typeof UserSenderMailerConfigListResSchema>;

export const userSenderMailerConfigList = async (
    param: UserSenderMailerConfigListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderMailerConfigListResType>> => {
    const { data } = await authApi().post('/api/user/app_sender/mailer/config_list', param, config);
    return data;
};

// 125. 获取邮件发送状态映射
export const UserSenderMailerMappingParamSchema = z.object({});
export type UserSenderMailerMappingParamType = z.infer<typeof UserSenderMailerMappingParamSchema>;

export const UserSenderMailerMappingResSchema = z.object({
    config_type: DictListSchema,
    log_status: DictListSchema,
    log_type: DictListSchema,
    mail_body_status: DictListSchema,
    mail_config_type: DictListSchema,
    mail_send_status: DictListSchema,
});
export type UserSenderMailerMappingResType = z.infer<typeof UserSenderMailerMappingResSchema>;

export const userSenderMailerMapping = async (
    param: UserSenderMailerMappingParamType = {},
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderMailerMappingResType>> => {
    const { data } = await authApi().post('/api/user/app_sender/mailer/mapping', param, config);
    return data;
};

// 126. 取消邮件发送
export const UserSenderMailerMessageCancelParamSchema = z.object({
    message_id: z.coerce.number(),
});
export type UserSenderMailerMessageCancelParamType = z.infer<typeof UserSenderMailerMessageCancelParamSchema>;

export const UserSenderMailerMessageCancelResSchema = z.object({});
export type UserSenderMailerMessageCancelResType = z.infer<typeof UserSenderMailerMessageCancelResSchema>;

export const userSenderMailerMessageCancel = async (
    param: UserSenderMailerMessageCancelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderMailerMessageCancelResType>> => {
    const { data } = await authApi().post('/api/user/app_sender/mailer/message_cancel', param, config);
    return data;
};

// 127. 获取邮件发送消息列表
export const UserSenderMailerMessageListParamSchema = z.object({
    app_id: z.coerce.number(),
    tpl_id: z.string().nullable().optional(),
    status: z.coerce.number().nullable().optional(),
    body_id: z.coerce.number().nullable().optional(),
    snid: z.string().nullable().optional(),
    to_mail: z.string().nullable().optional(),
    ...LimitParam,
});
export type UserSenderMailerMessageListParamType = z.infer<typeof UserSenderMailerMessageListParamSchema>;

export const UserSenderMailerMessageItemSchema = z.object({
    add_time: UnixTimestampSchema,
    app_id: z.coerce.number(),
    expected_time: UnixTimestampSchema,
    id: z.coerce.number(),
    max_try_num: z.coerce.number(),
    now_send: BoolSchema,
    on_task: BoolSchema,
    send_time: UnixTimestampSchema,
    snid: z.string(),
    status: z.coerce.number(),
    body_status: z.coerce.number().nullable(),
    to_mail: z.string(),
    tpl_key: z.string(),
    try_num: z.coerce.number(),
});
export type UserSenderMailerMessageItemType = z.infer<typeof UserSenderMailerMessageItemSchema>;

export const UserSenderMailerMessageListResSchema = z.object({
    data: z.array(UserSenderMailerMessageItemSchema),
    ...LimitRes,
});
export type UserSenderMailerMessageListResType = z.infer<typeof UserSenderMailerMessageListResSchema>;

export const userSenderMailerMessageList = async (
    param: UserSenderMailerMessageListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderMailerMessageListResType>> => {
    const cleanedParam = cleanEmptyStringParams(param, ['tpl_id', 'snid', 'to_mail']);
    const { data } = await authApi().post('/api/user/app_sender/mailer/message_list', cleanedParam, config);
    return parseResData(data, UserSenderMailerMessageListResSchema);
};

// 128. 获取邮件发送日志列表
export const UserSenderMailerMessageLogsParamSchema = z.object({
    message_id: z.coerce.number(),
    ...PageParam,
});
export type UserSenderMailerMessageLogsParamType = z.infer<typeof UserSenderMailerMessageLogsParamSchema>;

export const UserSenderMailerMessageLogItemSchema = z.object({
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
export type UserSenderMailerMessageLogItemType = z.infer<typeof UserSenderMailerMessageLogItemSchema>;

export const UserSenderMailerMessageLogsResSchema = z.object({
    data: z.array(UserSenderMailerMessageLogItemSchema),
    ...PageRes,
});
export type UserSenderMailerMessageLogsResType = z.infer<typeof UserSenderMailerMessageLogsResSchema>;

export const userSenderMailerMessageLogs = async (
    param: UserSenderMailerMessageLogsParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderMailerMessageLogsResType>> => {
    const { data } = await authApi().post('/api/user/app_sender/mailer/message_logs', param, config);
    return parseResData(data, UserSenderMailerMessageLogsResSchema);
};

// 129. 发送邮件消息
export const UserSenderMailerMessageSendParamSchema = z.object({
    app_id: z.coerce.number(),
    tpl_key: z.string(),
    to: z.array(z.string()),
    data: z.record(z.any()),
    reply: z.string().nullable().optional(),
    send_time: z.string().optional().nullable(),
    max_try: z.coerce.number().optional().nullable(),
});
export type UserSenderMailerMessageSendParamType = z.infer<typeof UserSenderMailerMessageSendParamSchema>;

export const UserSenderMailerMessageSendResSchema = z.object({});
export type UserSenderMailerMessageSendResType = z.infer<typeof UserSenderMailerMessageSendResSchema>;

export const userSenderMailerMessageSend = async (
    param: UserSenderMailerMessageSendParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderMailerMessageSendResType>> => {
    const { data } = await authApi().post('/api/user/app_sender/mailer/message_send', param, config);
    return data;
};

// 130. 查看邮件消息内容
export const UserSenderMailerMessageViewParamSchema = z.object({
    message_id: z.coerce.number(),
});
export type UserSenderMailerMessageViewParamType = z.infer<typeof UserSenderMailerMessageViewParamSchema>;

export const UserSenderMailerMessageViewResSchema = z.object({
    body: z.string(),
});
export type UserSenderMailerMessageViewResType = z.infer<typeof UserSenderMailerMessageViewResSchema>;

export const userSenderMailerMessageView = async (
    param: UserSenderMailerMessageViewParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderMailerMessageViewResType>> => {
    const { data } = await authApi().post('/api/user/app_sender/mailer/message_view', param, config);
    return data;
};

// 131. 添加SMTP发送配置
export const UserSenderMailerSmtpConfigAddParamSchema = z.object({
    app_id: z.coerce.number(),
    smtp_config_id: z.coerce.number(),
    name: z.string(),
    tpl_key: z.string(),
    from_email: z.string(),
    reply_email: z.string().optional(),
    subject_tpl_id: z.string(),
    body_tpl_id: z.string(),
});
export type UserSenderMailerSmtpConfigAddParamType = z.infer<typeof UserSenderMailerSmtpConfigAddParamSchema>;

export const UserSenderMailerSmtpConfigAddResSchema = z.object({
    id: z.coerce.number(),
});
export type UserSenderMailerSmtpConfigAddResType = z.infer<typeof UserSenderMailerSmtpConfigAddResSchema>;

export const userSenderMailerSmtpConfigAdd = async (
    param: UserSenderMailerSmtpConfigAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderMailerSmtpConfigAddResType>> => {
    const { data } = await authApi().post('/api/user/app_sender/mailer/smtp_config_add', param, config);
    return data;
};

// 132. 获取SMTP配置列表
export const UserSenderMailerSmtpConfigListParamSchema = z.object({
    ids: z.array(z.coerce.number()).nullable().optional(),
});
export type UserSenderMailerSmtpConfigListParamType = z.infer<typeof UserSenderMailerSmtpConfigListParamSchema>;

export const UserSenderMailerSmtpConfigItemSchema = z.object({
    email: z.string(),
    id: z.coerce.number(),
    name: z.string(),
    user: z.string(),
});
export type UserSenderMailerSmtpConfigItemType = z.infer<typeof UserSenderMailerSmtpConfigItemSchema>;

export const UserSenderMailerSmtpConfigListResSchema = z.object({
    data: z.array(UserSenderMailerSmtpConfigItemSchema),
});
export type UserSenderMailerSmtpConfigListResType = z.infer<typeof UserSenderMailerSmtpConfigListResSchema>;

export const userSenderMailerSmtpConfigList = async (
    param: UserSenderMailerSmtpConfigListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderMailerSmtpConfigListResType>> => {
    const { data } = await authApi().post('/api/user/app_sender/mailer/smtp_config_list', param, config);
    return data;
};

// 133. 添加邮件模板内容
export const UserSenderMailerTplBodyAddParamSchema = z.object({
    app_id: z.coerce.number(),
    tpl_id: z.string(),
    tpl_data: z.string(),
});
export type UserSenderMailerTplBodyAddParamType = z.infer<typeof UserSenderMailerTplBodyAddParamSchema>;

export const UserSenderMailerTplBodyAddResSchema = z.object({
    id: z.coerce.number(),
});
export type UserSenderMailerTplBodyAddResType = z.infer<typeof UserSenderMailerTplBodyAddResSchema>;

export const userSenderMailerTplBodyAdd = async (
    param: UserSenderMailerTplBodyAddParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderMailerTplBodyAddResType>> => {
    const { data } = await authApi().post('/api/user/app_sender/mailer/tpl_body_add', param, config);
    return data;
};

// 134. 删除邮件模板内容
export const UserSenderMailerTplBodyDelParamSchema = z.object({
    id: z.coerce.number(),
});
export type UserSenderMailerTplBodyDelParamType = z.infer<typeof UserSenderMailerTplBodyDelParamSchema>;

export const UserSenderMailerTplBodyDelResSchema = z.object({});
export type UserSenderMailerTplBodyDelResType = z.infer<typeof UserSenderMailerTplBodyDelResSchema>;

export const userSenderMailerTplBodyDel = async (
    param: UserSenderMailerTplBodyDelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderMailerTplBodyDelResType>> => {
    const { data } = await authApi().post('/api/user/app_sender/mailer/tpl_body_del', param, config);
    return data;
};

// 135. 编辑邮件模板内容
export const UserSenderMailerTplBodyEditParamSchema = z.object({
    id: z.coerce.number(),
    tpl_data: z.string(),
});
export type UserSenderMailerTplBodyEditParamType = z.infer<typeof UserSenderMailerTplBodyEditParamSchema>;

export const UserSenderMailerTplBodyEditResSchema = z.object({});
export type UserSenderMailerTplBodyEditResType = z.infer<typeof UserSenderMailerTplBodyEditResSchema>;

export const userSenderMailerTplBodyEdit = async (
    param: UserSenderMailerTplBodyEditParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderMailerTplBodyEditResType>> => {
    const { data } = await authApi().post('/api/user/app_sender/mailer/tpl_body_edit', param, config);
    return data;
};

// 136. 获取邮件模板内容列表
export const UserSenderMailerTplBodyListParamSchema = z.object({
    id: z.coerce.number().nullable().optional(),
    app_id: z.coerce.number(),
    tpl_id: z.string().nullable().optional(),
    tpl_id_like: z.string().optional().nullable(),
    ...PageParam,
});
export type UserSenderMailerTplBodyListParamType = z.infer<typeof UserSenderMailerTplBodyListParamSchema>;

export const UserSenderMailerTplBodyItemSchema = z.object({
    change_time: UnixTimestampSchema,
    change_user_id: z.coerce.number(),
    id: z.coerce.number(),
    sender_type: z.coerce.number(),
    status: z.coerce.number(),
    tpl_data: z.string(),
    tpl_id: z.string(),
    user_id: z.coerce.number(),
});
export type UserSenderMailerTplBodyItemType = z.infer<typeof UserSenderMailerTplBodyItemSchema>;

export const UserSenderMailerTplBodyListResSchema = z.object({
    data: z.array(UserSenderMailerTplBodyItemSchema),
    ...PageRes,
});
export type UserSenderMailerTplBodyListResType = z.infer<typeof UserSenderMailerTplBodyListResSchema>;

export const userSenderMailerTplBodyList = async (
    param: UserSenderMailerTplBodyListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderMailerTplBodyListResType>> => {
    const cleanedParam = cleanEmptyStringParams(param, ['tpl_id', 'tpl_id_like']);
    const { data } = await authApi().post('/api/user/app_sender/mailer/tpl_body_list', cleanedParam, config);
    return data;
};

// 137. 删除邮件模板配置
export const UserSenderMailerTplConfigDelParamSchema = z.object({
    tpl_config_id: z.coerce.number(),
});
export type UserSenderMailerTplConfigDelParamType = z.infer<typeof UserSenderMailerTplConfigDelParamSchema>;

export const UserSenderMailerTplConfigDelResSchema = z.object({
    num: z.string(),
});
export type UserSenderMailerTplConfigDelResType = z.infer<typeof UserSenderMailerTplConfigDelResSchema>;

export const userSenderMailerTplConfigDel = async (
    param: UserSenderMailerTplConfigDelParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderMailerTplConfigDelResType>> => {
    const { data } = await authApi().post('/api/user/app_sender/mailer/tpl_config_del', param, config);
    return data;
};

// 138. 获取邮件模板配置列表
export const UserSenderMailerTplConfigListParamSchema = z.object({
    id: z.coerce.number().nullable().optional(),
    app_id: z.coerce.number(),
    tpl: z.string().nullable().optional(),
    like_tpl: z.string().nullable().optional(),
    app_info: BoolSchema.nullable().optional(),
    ...PageParam,
});
export type UserSenderMailerTplConfigListParamType = z.infer<typeof UserSenderMailerTplConfigListParamSchema>;

export const UserSenderMailerTplConfigItemSchema = z.object({
    app_id: z.coerce.number(),
    change_time: UnixTimestampSchema,
    change_user_id: z.coerce.number(),
    config_data: z.object({
        body_tpl_id: z.string(),
        from_email: z.string(),
        reply_email: z.string(),
        subject_tpl_id: z.string(),
    }),
    id: z.coerce.number(),
    name: z.string(),
    setting_id: z.coerce.number(),
    setting_key: z.string(),
    setting_name: z.string(),
    tpl_key: z.string(),
    user_id: z.coerce.number(),
});
export type UserSenderMailerTplConfigItemType = z.infer<typeof UserSenderMailerTplConfigItemSchema>;

export const UserSenderMailerTplConfigListResSchema = z.object({
    data: z.array(UserSenderMailerTplConfigItemSchema),
    ...PageRes,
});
export type UserSenderMailerTplConfigListResType = z.infer<typeof UserSenderMailerTplConfigListResSchema>;

export const userSenderMailerTplConfigList = async (
    param: UserSenderMailerTplConfigListParamType,
    config?: AxiosRequestConfig<any>
): Promise<ApiResult<UserSenderMailerTplConfigListResType>> => {
    const cleanedParam = cleanEmptyStringParams(param, ['tpl', 'like_tpl']);
    const { data } = await authApi().post('/api/user/app_sender/mailer/tpl_config_list', cleanedParam, config);
    return data;
};


