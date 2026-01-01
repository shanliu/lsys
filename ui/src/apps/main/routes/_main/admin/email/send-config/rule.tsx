import { EmailSendConfigRulePage } from '@apps/main/features/admin/pages/email/send-config-rule-page'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/email/send-config/rule')({
    component: EmailSendConfigRulePage,
})
