import { redirect } from '@tanstack/react-router'
import { z } from 'zod'

// 定义路径参数的验证模式
const appParamsSchema = z.object({
    appId: z.coerce.number().int().positive("应用ID必须是正整数")
})

/**
 * AppId 参数解析函数
 * 统一处理 appId 参数的验证和错误处理
 */
export const parseAppIdParams = (params: Record<string, unknown>) => {
    try {
        return appParamsSchema.parse(params)
    } catch {
        // 如果 appId 无效，重定向到应用列表页面
        throw redirect({
            to: '/user/app',
            replace: true
        })
    }
}

/**
 * AppId 参数字符串化函数
 */
export const stringifyAppIdParams = (params: { appId: number }) => ({
    appId: params.appId.toString()
})

/**
 * AppId 路由前置验证函数
 */
export const validateAppIdBeforeLoad = ({ params }: { params: { appId: number } }) => {
    // 二次验证确保 appId 有效
    if (!params.appId || params.appId <= 0) {
        throw redirect({
            to: '/user/app',
            replace: true
        })
    }
}
