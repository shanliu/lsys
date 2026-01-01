import { z } from 'zod'

// OAuth 页面搜索参数 Schema - 强制约束所有参数
export const OAuthSearchSchema = z.object({
    scope: z.string().min(1, "授权范围不能为空"),
    client_id: z.string().min(1, "客户端ID不能为空"),
    redirect_uri: z.string().min(1, "回调地址不能为空"),
    state: z.string().optional().default(""),
})

export type OAuthSearchType = z.infer<typeof OAuthSearchSchema>
