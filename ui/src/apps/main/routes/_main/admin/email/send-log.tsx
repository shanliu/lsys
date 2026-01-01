import { SendLogPage } from '@apps/main/features/admin/pages/email/send-log-page'
import { EmailLogFilterParamSchema } from '@apps/main/features/admin/pages/email/send-log-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/email/send-log')({
    component: SendLogPage,
    validateSearch: EmailLogFilterParamSchema,
})
