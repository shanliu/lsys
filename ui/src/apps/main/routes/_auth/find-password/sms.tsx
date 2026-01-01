import { FindPasswordSmsPage } from '@apps/main/features/auth/pages/find-password-sms-page'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_auth/find-password/sms')({
  component: FindPasswordSmsPage,
})
