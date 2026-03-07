import AppDetailFeatureCollectorListPage from '@apps/main/features/user/pages/app/detail/feature-file/collector-list-page'
import { CollectorListFilterParamSchema } from '@apps/main/features/user/pages/app/detail/feature-file/collector-list-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/user/app/$appId/features-file/collector')({
    component: AppDetailFeatureCollectorListPage,
    validateSearch: CollectorListFilterParamSchema,
})
