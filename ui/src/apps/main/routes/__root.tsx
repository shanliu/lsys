import { PageErrorBoundaryCreater } from '@/apps/main/components/local/page-error-boundary'
import { RootLayout } from '@apps/main//components/layout/root-layout'
import { QueryClient } from '@tanstack/react-query'
import { createRootRouteWithContext } from '@tanstack/react-router'
export const Route = createRootRouteWithContext<{
  queryClient: QueryClient
}>()({
  component: RootLayout,
  // notFoundComponent: NotFoundError,
  errorComponent: PageErrorBoundaryCreater({ variant: 'content' }),
})
