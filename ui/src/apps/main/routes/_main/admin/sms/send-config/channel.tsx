import { SmsSendConfigTplConfigPage } from '@apps/main/features/admin/pages/sms/send-config-tpl-config-page'
import { SmsSendConfigTplFilterParamSchema } from '@apps/main/features/admin/pages/sms/send-config-tpl-config-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/sms/send-config/channel')({
    component: SmsSendConfigTplConfigPage,
    validateSearch: SmsSendConfigTplFilterParamSchema,
})
