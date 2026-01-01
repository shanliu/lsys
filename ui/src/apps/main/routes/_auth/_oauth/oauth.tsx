import { OAuthPage } from '@apps/main/features/auth/pages/oauth-page'
import { OAuthSearchSchema } from '@apps/main/features/auth/pages/oauth-schema'
import { userStore } from '@shared/lib/auth'
import { createFileRoute, redirect } from '@tanstack/react-router'

export const Route = createFileRoute('/_auth/_oauth/oauth')({
    validateSearch: OAuthSearchSchema,
    beforeLoad: () => {
        // console.log('OAuth Route beforeLoad called', userStore.getState())
        // 检查用户是否登录，未登录则重定向到登录页
        if (!userStore.getState().isLoggedIn()) {
            // 获取当前完整URL作为 redirect_uri
            const currentUrl = window.location.href
            throw redirect({
                to: '/sign-in',
                search: { redirect_uri: currentUrl },
                replace: true
            })
        }
    },
    component: OAuthPage,
})
