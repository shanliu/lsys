import { SmsAdapterConfigPage } from '@apps/main/features/admin/pages/sms/adapter-config-page'
import { AdapterConfigParamSchema } from '@apps/main/features/admin/pages/sms/adapter-config-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/sms/adapter-config')({
    validateSearch: AdapterConfigParamSchema,
    component: SmsAdapterConfigPage
})
