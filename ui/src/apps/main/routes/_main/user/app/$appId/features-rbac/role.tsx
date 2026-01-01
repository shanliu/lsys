import AppDetailFeatureRbacRolePage, { RoleListFilterParamSchema } from '@apps/main/features/user/pages/app/detail/feature-rbac/role-page'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/user/app/$appId/features-rbac/role')({
  component: AppDetailFeatureRbacRolePage,
  validateSearch: RoleListFilterParamSchema,
})
