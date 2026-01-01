import { PageDataParam } from '@shared/types/base-schema';
import { z } from 'zod';

// 基础过滤器字段 schema
const EmailSendConfigTplBodyFilterBaseSchema = z.object({
    tpl_id: z.string().optional(),
});

// URL 参数 schema，包含分页参数
export const EmailSendConfigTplBodyFilterParamSchema = EmailSendConfigTplBodyFilterBaseSchema.extend(PageDataParam);

// 表单 schema
export const EmailSendConfigTplBodyFilterFormSchema = z.object({
    tpl_id: z.string().optional(),
});

export type EmailSendConfigTplBodyFilterParamType = z.infer<typeof EmailSendConfigTplBodyFilterParamSchema>;

// 表单 Schema
export const EmailSendConfigTplBodyFormSchema = z.object({
    tpl_id: z.string().min(1, "模板ID不能为空"),
    tpl_data: z.string().min(1, "模板内容不能为空"),
});

export type EmailSendConfigTplBodyFormType = z.infer<typeof EmailSendConfigTplBodyFormSchema>;
