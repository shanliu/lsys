import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary'
import { AdminSettingLayout } from '@apps/main/features/admin/components/layout/admin-setting-layout'
import { cn } from '@shared/lib/utils'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/config')({
  component: AdminSettingLayout,
  errorComponent: PageErrorBoundaryCreater({ variant: 'page', className: cn("p-6") }),
})
