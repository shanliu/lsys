import { AdminOssConfigPage } from '@/apps/main/features/admin/pages/file/oss-config-page'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/file/oss-config')({
    component: AdminOssConfigPage,
})
