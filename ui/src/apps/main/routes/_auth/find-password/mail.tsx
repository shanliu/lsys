import { FindPasswordMailPage } from '@apps/main/features/auth/pages/find-password-mail-page'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_auth/find-password/mail')({
  component: FindPasswordMailPage,
})
