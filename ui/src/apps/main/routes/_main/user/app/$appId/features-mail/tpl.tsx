import { default as AppDetailFeatureMailTplPage } from '@apps/main/features/user/pages/app/detail/feature-mail/tpl-page'
import { MailTplFilterParamSchema } from '@apps/main/features/user/pages/app/detail/feature-mail/tpl-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/user/app/$appId/features-mail/tpl')({
    component: AppDetailFeatureMailTplPage,
    validateSearch: MailTplFilterParamSchema,
})



