import { LoginLogPage } from '@apps/main/features/admin/pages/user/login-log-page'
import { LoginLogFilterParamSchema } from '@apps/main/features/admin/pages/user/login-log-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/user/login-log')({
    component: LoginLogPage,
    validateSearch: LoginLogFilterParamSchema,
})
