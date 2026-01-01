import AppDetailFeatureRbacAuditPage from '@apps/main/features/user/pages/app/detail/feature-rbac/audit-page'
import { AuditListFilterParamSchema } from '@apps/main/features/user/pages/app/detail/feature-rbac/audit-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute(
  '/_main/user/app/$appId/features-rbac/audit',
)({
  component: AppDetailFeatureRbacAuditPage,
  validateSearch: AuditListFilterParamSchema,
})

