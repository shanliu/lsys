import { EmailSendConfigTplConfigPage } from '@apps/main/features/admin/pages/email/send-config-tpl-config-page'
import { EmailSendConfigTplConfigFilterParamSchema } from '@apps/main/features/admin/pages/email/send-config-tpl-config-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/email/send-config/channel')({
    component: EmailSendConfigTplConfigPage,
    validateSearch: EmailSendConfigTplConfigFilterParamSchema,
})
