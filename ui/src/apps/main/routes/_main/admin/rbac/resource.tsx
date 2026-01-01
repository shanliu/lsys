import { ResPage, ResListFilterParamSchema } from '@apps/main/features/admin/pages/rbac/res-page'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/rbac/resource')({
    component: ResPage,
    validateSearch: (search) => ResListFilterParamSchema.parse(search),
})
