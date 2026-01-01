import { RolePage, RoleListFilterParamSchema } from '@apps/main/features/admin/pages/rbac/role-page'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/rbac/role')({
    component: RolePage,
    validateSearch: (search) => RoleListFilterParamSchema.parse(search),
})
