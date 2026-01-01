import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary'
import { AdminBreadcrumbLayout } from '@apps/main/features/admin/components/layout/admin-breadcrumb-layout'
import { cn } from '@shared/lib/utils'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/user')({
 component: AdminBreadcrumbLayout,
  errorComponent: PageErrorBoundaryCreater({ variant: 'page' , className: cn("p-6")}),
})
