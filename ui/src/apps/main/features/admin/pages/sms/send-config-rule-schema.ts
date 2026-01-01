import { z } from "zod";

// 新增配置表单 Schema
export const SmsSendConfigSendRuleFormSchema = z.object({
    priority: z.coerce.number().min(0, "优先级必须大于等于0"),
    config_type: z.coerce.number().min(1, "请选择配置类型"),
    config_data: z.any(), // 根据 config_type 动态验证
});

export type SmsSendConfigSendRuleFormType = z.infer<typeof SmsSendConfigSendRuleFormSchema>;
