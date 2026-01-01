import AppDetailFeatureSmsTplConfigPage from '@apps/main/features/user/pages/app/detail/feature-sms/tpl-config-page'
import { TplConfigFilterParamSchema } from '@apps/main/features/user/pages/app/detail/feature-sms/tpl-config-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/user/app/$appId/features-sms/tpl-config')({
    component: AppDetailFeatureSmsTplConfigPage,
    validateSearch: TplConfigFilterParamSchema,
})



