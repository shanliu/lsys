import { PageDataParam } from '@shared/types/base-schema';
import { z } from 'zod';

// 基础过滤器字段 schema
const MailTplFilterBaseSchema = z.object({
    tpl_id: z.string().optional(),
});

// URL 参数 schema，包含分页参数
export const MailTplFilterParamSchema = MailTplFilterBaseSchema.extend(PageDataParam);

// 表单 schema
export const MailTplFilterFormSchema = z.object({
    tpl_id: z.string().optional(),
});

export type MailTplFilterParamType = z.infer<typeof MailTplFilterParamSchema>;


// 表单 Schema
export const MailTplFormSchema = z.object({
    tpl_id: z.string().min(1, "模板ID不能为空"),
    tpl_data: z.string().min(1, "模板内容不能为空"),
});

export type MailTplFormType = z.infer<typeof MailTplFormSchema>
