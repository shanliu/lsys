import { SendLogPage } from '@apps/main/features/admin/pages/sms/send-log-page'
import { SmsLogFilterParamSchema } from '@apps/main/features/admin/pages/sms/send-log-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/sms/send-log')({
    component: SendLogPage,
    validateSearch: SmsLogFilterParamSchema,
})
