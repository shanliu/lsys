import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary'
import { UserBreadcrumbLayout } from '@apps/main/features/user/components/layout/user-breadcrumb-layout'
import { cn } from '@shared/lib/utils'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/user/account')({
  component: UserBreadcrumbLayout,
   errorComponent: PageErrorBoundaryCreater({ variant: 'page' , className: cn("p-6")}),
})
