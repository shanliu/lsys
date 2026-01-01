import { EmailAdapterConfigPage } from '@apps/main/features/admin/pages/email/adapter-config-page'
import { AdapterConfigParamSchema } from '@apps/main/features/admin/pages/email/adapter-config-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/email/adapter-config')({
    validateSearch: AdapterConfigParamSchema,
    component: EmailAdapterConfigPage
})
