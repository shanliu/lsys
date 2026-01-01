import AppDetailFeatureMailConfigPage from '@apps/main/features/user/pages/app/detail/feature-mail/config-page'
import { MailConfigFilterParamSchema } from '@apps/main/features/user/pages/app/detail/feature-mail/config-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/user/app/$appId/features-mail/config')({
  component: AppDetailFeatureMailConfigPage,
  validateSearch: MailConfigFilterParamSchema,
})
