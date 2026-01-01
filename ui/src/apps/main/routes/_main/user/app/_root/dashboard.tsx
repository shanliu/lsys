
import { createFileRoute } from '@tanstack/react-router'
import AppIndexPage from '@apps/main/features/user/pages/app/dashboard-page'

export const Route = createFileRoute('/_main/user/app/_root/dashboard')({
  component: AppIndexPage,
})
