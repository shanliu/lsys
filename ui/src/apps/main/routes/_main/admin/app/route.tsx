import { createFileRoute } from '@tanstack/react-router'

import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary'
import { AdminBreadcrumbLayout } from '@apps/main/features/admin/components/layout/admin-breadcrumb-layout'
import { cn } from '@shared/lib/utils'

export const Route = createFileRoute('/_main/admin/app')({
  component: AdminBreadcrumbLayout,
  errorComponent: PageErrorBoundaryCreater({ variant: 'page', className: cn("p-4") }),
})
