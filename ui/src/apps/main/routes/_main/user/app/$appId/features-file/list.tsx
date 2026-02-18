import AppDetailFeatureFileListPage from '@apps/main/features/user/pages/app/detail/feature-file/list-page'
import { FileListFilterParamSchema } from '@apps/main/features/user/pages/app/detail/feature-file/list-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/user/app/$appId/features-file/list')({
    component: AppDetailFeatureFileListPage,
    validateSearch: FileListFilterParamSchema,
})
