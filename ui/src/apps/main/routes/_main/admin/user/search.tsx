import { UserSearchPage } from '@apps/main/features/admin/pages/user/search-page'
import { UserAccountFilterParamSchema } from '@apps/main/features/admin/pages/user/search-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/user/search')({
  component: UserSearchPage,
  validateSearch: UserAccountFilterParamSchema,
})
