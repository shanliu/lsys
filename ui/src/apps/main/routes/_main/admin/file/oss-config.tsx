
import { AdminOssConfigPage } from '@/apps/main/features/admin/pages/file/oss-config-page'
import { AdminFileListFilterParamSchema } from '@apps/main/features/admin/pages/file/file-list-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/file/oss-config')({
    validateSearch: AdminFileListFilterParamSchema,
    component: AdminOssConfigPage,
})
