import AppDetailFeatureMailListPage from '@apps/main/features/user/pages/app/detail/feature-mail/list-page'
import { MailListFilterParamSchema } from '@apps/main/features/user/pages/app/detail/feature-mail/list-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/user/app/$appId/features-mail/list')({
  component: AppDetailFeatureMailListPage,
  validateSearch: MailListFilterParamSchema,
})

