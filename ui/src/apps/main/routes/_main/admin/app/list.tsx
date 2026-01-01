import { AppListPage, AdminAppListFilterParamSchema } from '@apps/main/features/admin/pages/app/list-page'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/app/list')({
    component: AppListPage,
    validateSearch: AdminAppListFilterParamSchema,
})
