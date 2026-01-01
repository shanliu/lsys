import { ChangeLogPage } from '@apps/main/features/admin/pages/user/change-log-page'
import { UserChangeLogFilterParamSchema } from '@apps/main/features/admin/pages/user/change-log-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/user/change-log')({
    component: ChangeLogPage,
    validateSearch: UserChangeLogFilterParamSchema,
})
