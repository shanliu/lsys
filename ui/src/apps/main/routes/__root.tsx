import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary'
import { RootLayout } from '@apps/main/features/common/components/layout/root-layout'
import { QueryClient } from '@tanstack/react-query'
import { createRootRouteWithContext } from '@tanstack/react-router'
export const Route = createRootRouteWithContext<{
  queryClient: QueryClient
}>()({
  component: RootLayout,
  // notFoundComponent: NotFoundError,
  errorComponent: PageErrorBoundaryCreater({ variant: 'content' }),
})
