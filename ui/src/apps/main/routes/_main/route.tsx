import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary'
import { MainLayout } from '@apps/main/features/common/components/layout/main-layout'
import { validateAuthAndRedirect } from '@apps/main/lib/route-guards'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main')({
  component: MainLayout,
  errorComponent: PageErrorBoundaryCreater({ variant: 'content' }),
  beforeLoad: ({ location }) => validateAuthAndRedirect(location)
})
