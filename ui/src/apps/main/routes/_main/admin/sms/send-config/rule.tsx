import { SmsSendConfigRulePage } from '@apps/main/features/admin/pages/sms/send-config-rule-page'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/sms/send-config/rule')({
    component: SmsSendConfigRulePage,
})
