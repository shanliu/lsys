import { AuditLogPage } from '@apps/main/features/admin/pages/rbac/audit-log-page'
import { RbacAuditLogFilterParamSchema } from '@apps/main/features/admin/pages/rbac/audit-log-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/rbac/audit-log')({
    component: AuditLogPage,
    validateSearch: RbacAuditLogFilterParamSchema,
})
