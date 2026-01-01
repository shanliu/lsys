import { AppRequestPage } from '@apps/main/features/admin/pages/app/request-page'
import { AdminAppRequestFilterParamSchema } from '@apps/main/features/admin/pages/app/request-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/app/request')({
    component: AppRequestPage,
    validateSearch: AdminAppRequestFilterParamSchema,
})
