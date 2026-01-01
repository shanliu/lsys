import { ResPage } from '@apps/main/features/admin/pages/rbac/res-page'
import { ResListFilterParamSchema } from '@apps/main/features/user/pages/app/detail/feature-rbac/res-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/rbac/res')({
    component: ResPage,
    validateSearch: (search) => ResListFilterParamSchema.parse(search),
})
