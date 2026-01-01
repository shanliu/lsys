import { createFileRoute } from '@tanstack/react-router'
import { AuthLayout } from '@apps/main/features/auth/components/layout/auth-layout'

export const Route = createFileRoute('/_auth')({
  component: AuthLayout,
})
