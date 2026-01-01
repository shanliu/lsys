import { createFileRoute } from '@tanstack/react-router'
import { UserSwitchPage } from '@apps/main/features/auth/pages/user-switch-page'

export const Route = createFileRoute('/_main/switch-user')({
  component: UserSwitchPage
})
