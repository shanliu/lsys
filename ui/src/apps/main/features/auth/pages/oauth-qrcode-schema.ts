import { z } from 'zod'

// OAuth QrCode 页面搜索参数 Schema
// code 为可选，因为 external-bind-drawer.tsx 生成路径时不需要
// 但 state 和 login_type 为必须参数
export const OAuthQrCodeSearchSchema = z.object({
    code: z.string().optional(),
    state: z.string().min(1, "状态参数不能为空"),
    login_type: z.string().min(1, "登录类型不能为空"),
})

export type OAuthQrCodeSearchType = z.infer<typeof OAuthQrCodeSearchSchema>
