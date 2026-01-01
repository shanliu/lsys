import AppDetailFeatureMailTplConfigPage from '@apps/main/features/user/pages/app/detail/feature-mail/tpl-config-page'
import { TplConfigFilterParamSchema } from '@apps/main/features/user/pages/app/detail/feature-mail/tpl-config-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/user/app/$appId/features-mail/tpl-config')({
    component: AppDetailFeatureMailTplConfigPage,
    validateSearch: TplConfigFilterParamSchema,
})



