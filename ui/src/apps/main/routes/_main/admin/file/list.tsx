import { AdminFileListPage } from '@apps/main/features/admin/pages/file/file-list-page'
import { AdminFileListFilterParamSchema } from '@apps/main/features/admin/pages/file/file-list-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/file/list')({
    validateSearch: AdminFileListFilterParamSchema,
    component: AdminFileListPage,
})
