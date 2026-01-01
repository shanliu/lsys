import { isDomain } from "@shared/lib/utils";
import { PageDataParam } from "@shared/types/base-schema";
import { z } from "zod";

// 基础过滤器字段 schema
const MailConfigFilterBaseSchema = z.object({
    config_type: z.string().optional(),
});

// URL 参数 schema，包含分页参数
export const MailConfigFilterParamSchema = MailConfigFilterBaseSchema.extend(PageDataParam);

// 过滤表单 Schema
export const MailConfigFilterFormSchema = z.object({
    config_type: z.string().optional(),
});

export type MailConfigFilterParamType = z.infer<typeof MailConfigFilterParamSchema>;

// Config Type 2: 频率限制数据结构
const ConfigType2DataSchema = z.object({
    range_time: z.coerce.number().min(1, "时间范围必须大于0"),
    max_send: z.coerce.number().min(1, "最大发送数量必须大于0"),
});

// Config Type 3: 每次最大发送数量
const ConfigType3DataSchema = z.coerce.number().min(1, "数量必须大于0");

// Config Type 4: 指定模板不检测限制
const ConfigType4DataSchema = z.string().min(1, "模板不能为空").refine(
    (val) => !/^\s*$/.test(val),
    "模板不能为空白字符"
);

// Config Type 20: 指定邮箱屏蔽
const ConfigType20DataSchema = z.string().email("请提供正确的邮箱地址");

// Config Type 21: 指定域名屏蔽
const ConfigType21DataSchema = z.string().refine(
    (val) => isDomain(val, true),
    "请提供正确的域名"
);

// 新增配置表单 Schema
export const MailConfigFormSchema = z.object({
    priority: z.coerce.number().min(0, "优先级必须大于等于0"),
    config_type: z.coerce.number().min(1, "请选择配置类型"),
    config_data: z.any(), // 根据 config_type 动态验证
}).superRefine((data, ctx) => {
    // 根据 config_type 验证 config_data
    switch (data.config_type) {
        case 1:
            // 关闭功能，不需要配置数据
            data.config_data = '';
            break;
        case 2:
            // 频率限制
            try {
                ConfigType2DataSchema.parse(data.config_data);
            } catch (err: any) {
                ctx.addIssue({
                    code: z.ZodIssueCode.custom,
                    message: err.errors?.[0]?.message || "请设置限制数量跟时间",
                    path: ['config_data'],
                });
            }
            break;
        case 3:
            // 每次最大发送数量
            try {
                ConfigType3DataSchema.parse(data.config_data);
            } catch (err: any) {
                ctx.addIssue({
                    code: z.ZodIssueCode.custom,
                    message: err.errors?.[0]?.message || "数量必须大于0",
                    path: ['config_data'],
                });
            }
            break;
        case 4:
            // 指定模板不检测限制
            try {
                ConfigType4DataSchema.parse(data.config_data);
            } catch (err: any) {
                ctx.addIssue({
                    code: z.ZodIssueCode.custom,
                    message: err.errors?.[0]?.message || "模板不能为空",
                    path: ['config_data'],
                });
            }
            break;
        case 20:
            // 指定邮箱屏蔽
            try {
                ConfigType20DataSchema.parse(data.config_data);
            } catch (err: any) {
                ctx.addIssue({
                    code: z.ZodIssueCode.custom,
                    message: err.errors?.[0]?.message || "请提供正确邮箱",
                    path: ['config_data'],
                });
            }
            break;
        case 21:
            // 指定域名屏蔽
            try {
                ConfigType21DataSchema.parse(data.config_data);
            } catch (err: any) {
                ctx.addIssue({
                    code: z.ZodIssueCode.custom,
                    message: err.errors?.[0]?.message || "请提供正确域名",
                    path: ['config_data'],
                });
            }
            break;
        default:
            ctx.addIssue({
                code: z.ZodIssueCode.custom,
                message: "不支持的配置类型",
                path: ['config_type'],
            });
    }
});

export type MailConfigFormType = z.infer<typeof MailConfigFormSchema>;
