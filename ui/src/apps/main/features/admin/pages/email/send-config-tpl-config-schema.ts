import { PageDataParam } from "@shared/types/base-schema";
import { z } from "zod";

// 基础过滤器字段 schema
const EmailSendConfigTplConfigFilterBaseSchema = z.object({
    tpl: z.string().optional(),
});

// URL 参数 schema，包含分页参数
export const EmailSendConfigTplConfigFilterParamSchema = EmailSendConfigTplConfigFilterBaseSchema.extend(PageDataParam);

// 过滤表单 Schema
export const EmailSendConfigTplConfigFilterFormSchema = z.object({
    tpl: z.string().optional(),
});

export type EmailSendConfigTplConfigFilterParamType = z.infer<typeof EmailSendConfigTplConfigFilterParamSchema>;

// 邮件渠道类型
export enum EmailChannelType {
    SMTP = "smtp",
}

// 邮件渠道选项
export const EMAIL_CHANNEL_OPTIONS = [
    { value: EmailChannelType.SMTP, label: "SMTP渠道" },
];

// 渠道选择 Schema
export const EmailSendConfigTplConfigChannelSchema = z.object({
    channel: z.enum([EmailChannelType.SMTP], { message: "请选择邮件渠道" }).default(EmailChannelType.SMTP),
});

export type EmailSendConfigTplConfigChannelType = z.infer<typeof EmailSendConfigTplConfigChannelSchema>;

// SMTP 渠道表单 Schema
export const EmailSendConfigTplConfigSmtpFormSchema = z.object({
    channel: z.enum([EmailChannelType.SMTP]),
    smtp_config_id: z.coerce.number().min(1, "请选择SMTP配置"),
    name: z.string().min(1, "名称不能为空"),
    tpl_key: z.string().min(1, "模板Key不能为空"),
    from_email: z.string().email("请输入有效的邮箱地址"),
    reply_email: z.string().optional(),
    subject_tpl_id: z.string().min(1, "主题模板ID不能为空"),
    body_tpl_id: z.string().min(1, "正文模板ID不能为空"),
});

export type EmailSendConfigTplConfigSmtpFormType = z.infer<typeof EmailSendConfigTplConfigSmtpFormSchema>;

// 通用配置表单类型（渠道的所有可能类型的联合）
export type EmailSendConfigTplConfigFormType = 
    | EmailSendConfigTplConfigSmtpFormType;
