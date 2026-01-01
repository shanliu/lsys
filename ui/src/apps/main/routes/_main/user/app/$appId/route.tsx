import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary'
import { AppDetailLayout } from '@apps/main/features/user/components/layout/app-detail-layout'
import {
  parseAppIdParams,
  stringifyAppIdParams,
  validateAppIdBeforeLoad
} from '@apps/main/features/user/components/layout/app-detail-utils'
import { cn } from '@shared/lib/utils'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/user/app/$appId')({
  params: {
    parse: parseAppIdParams,
    stringify: stringifyAppIdParams
  },
  beforeLoad: validateAppIdBeforeLoad,
  component: AppDetailLayout,
  errorComponent: PageErrorBoundaryCreater({ variant: 'page' , className: cn("p-6")}),
})
