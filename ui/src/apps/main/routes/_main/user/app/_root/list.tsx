
import { AppListFilterParamSchema } from '@apps/main/features/user/pages/app/list-page'
import { createFileRoute } from '@tanstack/react-router'
import AppListPage from '@apps/main/features/user/pages/app/list-page'

export const Route = createFileRoute('/_main/user/app/_root/list')({
  component: AppListPage,
  validateSearch: AppListFilterParamSchema,
})
