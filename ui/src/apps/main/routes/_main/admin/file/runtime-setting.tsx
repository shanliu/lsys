import { AdminRuntimeSettingPage } from '@/apps/main/features/admin/pages/file/runtime-setting-page'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/admin/file/runtime-setting')({
    component: AdminRuntimeSettingPage,
})
