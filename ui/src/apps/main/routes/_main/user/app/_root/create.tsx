import AppCreatePage from '@apps/main/features/user/pages/app/create-page'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/user/app/_root/create')({
  component: AppCreatePage,
})
