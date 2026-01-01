import AppDetailFeatureMailSendPage from '@apps/main/features/user/pages/app/detail/feature-mail/send-page'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/user/app/$appId/features-mail/send')({
  component: AppDetailFeatureMailSendPage,
})

