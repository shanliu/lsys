import AppDetailFeatureSmsConfigPage from '@apps/main/features/user/pages/app/detail/feature-sms/config-page'
import { SmsConfigFilterParamSchema } from '@apps/main/features/user/pages/app/detail/feature-sms/config-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/user/app/$appId/features-sms/config')({
  component: AppDetailFeatureSmsConfigPage,
  validateSearch: SmsConfigFilterParamSchema,
})
