import { EmailSendConfigTplBodyPage } from '@apps/main/features/admin/pages/email/send-config-tpl-body-page'
import { EmailSendConfigTplBodyFilterParamSchema } from '@apps/main/features/admin/pages/email/send-config-tpl-body-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/email/send-config/template')({
    component: EmailSendConfigTplBodyPage,
    validateSearch: EmailSendConfigTplBodyFilterParamSchema,
})
