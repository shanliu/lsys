import { PageDataParam } from "@shared/types/base-schema";
import { z } from "zod";

// 辅助函数：将空字符串转换为 undefined
const emptyToUndefined = (val: string | undefined) => (val === "" ? undefined : val);

// 基础过滤器字段 schema
const SmsSendConfigTplFilterBaseSchema = z.object({
    tpl: z.string().optional().transform(emptyToUndefined),
});

// URL 参数 schema，包含分页参数
export const SmsSendConfigTplFilterParamSchema = SmsSendConfigTplFilterBaseSchema.extend(PageDataParam);

// 过滤表单 Schema
export const SmsSendConfigTplFilterFormSchema = z.object({
    tpl: z.string().optional(),
});

export type SmsSendConfigTplFilterParamType = z.infer<typeof SmsSendConfigTplFilterParamSchema>;

// SMS 服务提供商类型
export enum SmsProviderType {
    ALI = "ali",
    TENCENT = "ten",
    HUAWEI = "hw",
    JD = "jd",
    NETEASE = "netease",
    CLOOPEN = "cloopen",
}

export const SMS_PROVIDER_LABELS = {
    [SmsProviderType.ALI]: "阿里云短信",
    [SmsProviderType.TENCENT]: "腾讯云短信",
    [SmsProviderType.HUAWEI]: "华为云短信",
    [SmsProviderType.JD]: "京东云短信",
    [SmsProviderType.NETEASE]: "网易云短信",
    [SmsProviderType.CLOOPEN]: "容联云短信",
};

// 阿里云短信配置
export const AliSmsConfigFormSchema = z.object({
    provider_type: z.literal(SmsProviderType.ALI),
    name: z.string().min(1, "配置名称不能为空"),
    tpl_key: z.string().min(1, "模板Key不能为空"),
    ali_config_id: z.coerce.number().min(1, "请选择阿里云配置"),
    aliyun_sms_tpl: z.string().min(1, "阿里云短信模板ID不能为空"),
    aliyun_sign_name: z.string().min(1, "阿里云短信签名不能为空"),
});

// 腾讯云短信配置
export const TencentSmsConfigFormSchema = z.object({
    provider_type: z.literal(SmsProviderType.TENCENT),
    name: z.string().min(1, "配置名称不能为空"),
    tpl_key: z.string().min(1, "模板Key不能为空"),
    config_id: z.coerce.number().min(1, "请选择腾讯云配置"),
    sign_name: z.string().min(1, "短信签名不能为空"),
    template_id: z.string().min(1, "模板ID不能为空"),
    template_map: z.string().min(1, "模板参数映射不能为空"),
});

// 华为云短信配置
export const HuaweiSmsConfigFormSchema = z.object({
    provider_type: z.literal(SmsProviderType.HUAWEI),
    name: z.string().min(1, "配置名称不能为空"),
    tpl_key: z.string().min(1, "模板Key不能为空"),
    hw_config_id: z.coerce.number().min(1, "请选择华为云配置"),
    signature: z.string().min(1, "短信签名不能为空"),
    sender: z.string().min(1, "发送者不能为空"),
    template_id: z.string().min(1, "模板ID不能为空"),
    template_map: z.string().min(1, "模板参数映射不能为空"),
});

// 京东云短信配置
export const JdSmsConfigFormSchema = z.object({
    provider_type: z.literal(SmsProviderType.JD),
    name: z.string().min(1, "配置名称不能为空"),
    tpl_key: z.string().min(1, "模板Key不能为空"),
    config_id: z.coerce.number().min(1, "请选择京东云配置"),
    sign_id: z.string().min(1, "签名ID不能为空"),
    template_id: z.string().min(1, "模板ID不能为空"),
    template_map: z.string().min(1, "模板参数映射不能为空"),
});

// 网易云短信配置
export const NeteaseSmsConfigFormSchema = z.object({
    provider_type: z.literal(SmsProviderType.NETEASE),
    name: z.string().min(1, "配置名称不能为空"),
    tpl_key: z.string().min(1, "模板Key不能为空"),
    config_id: z.coerce.number().min(1, "请选择网易云配置"),
    template_id: z.string().min(1, "模板ID不能为空"),
    template_map: z.string().min(1, "模板参数映射不能为空"),
});

// 容联云短信配置
export const CloopenSmsConfigFormSchema = z.object({
    provider_type: z.literal(SmsProviderType.CLOOPEN),
    name: z.string().min(1, "配置名称不能为空"),
    tpl_key: z.string().min(1, "模板Key不能为空"),
    config_id: z.coerce.number().min(1, "请选择容联云配置"),
    template_id: z.string().min(1, "容联云短信模板ID不能为空"),
    template_map: z.string().min(1, "模板参数映射不能为空"),
});

// 统一的模板配置表单 Schema
export const TplConfigFormSchema = z.discriminatedUnion("provider_type", [
    AliSmsConfigFormSchema,
    TencentSmsConfigFormSchema,
    HuaweiSmsConfigFormSchema,
    JdSmsConfigFormSchema,
    NeteaseSmsConfigFormSchema,
    CloopenSmsConfigFormSchema,
]);

export type TplConfigFormType = z.infer<typeof TplConfigFormSchema>;
