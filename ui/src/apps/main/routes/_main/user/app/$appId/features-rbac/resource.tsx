import AppDetailFeatureRbacResPage, { ResListFilterParamSchema } from '@apps/main/features/user/pages/app/detail/feature-rbac/res-page'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/user/app/$appId/features-rbac/resource')({
    component: AppDetailFeatureRbacResPage,
    validateSearch: ResListFilterParamSchema,
})
