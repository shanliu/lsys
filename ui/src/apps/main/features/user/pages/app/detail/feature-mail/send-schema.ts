import { EmailSchema } from "@shared/types/base-schema"
import { z } from "zod"

// 表单验证 Schema
export const MailSendFormSchema = z.object({
    tpl_key: z.string().min(1, "请选择发送模板"),
    to: z.array(EmailSchema).min(1, "请至少添加一个收件人"),
    data: z.string().refine((val) => {
        if (!val.trim()) return true // 允许为空
        try {
            JSON.parse(val)
            return true
        } catch {
            return false
        }
    }, "请输入有效的 JSON 格式"),
    reply: EmailSchema.optional().or(z.literal("")),
    send_time: z.date().optional(),
    max_try: z.number().min(0).optional(),
})

export type MailSendFormType = z.infer<typeof MailSendFormSchema>
