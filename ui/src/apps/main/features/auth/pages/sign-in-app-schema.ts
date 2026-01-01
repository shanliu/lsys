import { z } from 'zod'

// 应用登录页面搜索参数 Schema - 强制约束必需参数
export const SignInAppSearchSchema = z.object({
    client_id: z
        .string()
        .min(2, "客户端ID不能为空")
        .max(128, "客户端ID长度不能超过128个字符")
        .regex(/^[a-zA-Z0-9_.-]+$/, "客户端ID只能包含字母、数字、下划线、连字符和点"),
    code: z.string().min(16, "授权码不能为空"),
    redirect_uri: z.string().optional(),
})

export type SignInAppSearchType = z.infer<typeof SignInAppSearchSchema>
