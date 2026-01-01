import { ConfigPage } from '@apps/main/features/admin/pages/config/config-page'
import { ConfigParamSchema } from '@apps/main/features/admin/pages/config/config-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/config/')({
    validateSearch: ConfigParamSchema,
    component: ConfigPage
})
