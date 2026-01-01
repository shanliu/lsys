import AppDetailFeatureSmsSendPage from '@apps/main/features/user/pages/app/detail/feature-sms/send-page'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/user/app/$appId/features-sms/send')({
  component: AppDetailFeatureSmsSendPage,
})

