import AppDetailIndexPage from '@apps/main/features/user/pages/app/detail/index-page'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/user/app/$appId/')({
  component: AppDetailIndexPage,
})
