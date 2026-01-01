
import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary'
import { UserLayout } from '@apps/main/features/user/components/layout/user-layout'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/user')({
  component: UserLayout,
  errorComponent: PageErrorBoundaryCreater({ variant: 'layout' }),
})
