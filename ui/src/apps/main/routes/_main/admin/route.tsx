import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary'
import { AdminLayout } from '@apps/main/features/admin/components/layout/admin-layout'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin')({
  component: AdminLayout,
  errorComponent: PageErrorBoundaryCreater({ variant: 'layout' }),
})
