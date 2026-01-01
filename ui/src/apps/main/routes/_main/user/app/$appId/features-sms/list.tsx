import AppDetailFeatureSmsListPage from '@apps/main/features/user/pages/app/detail/feature-sms/list-page'
import { SmsListFilterParamSchema } from '@apps/main/features/user/pages/app/detail/feature-sms/list-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/user/app/$appId/features-sms/list')({
  component: AppDetailFeatureSmsListPage,
  validateSearch: SmsListFilterParamSchema,
})
