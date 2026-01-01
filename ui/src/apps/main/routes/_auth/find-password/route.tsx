import { FindPasswordLayout } from '@apps/main/features/auth/components/layout/find-passwrod-layout'
import { redirectIfLoggedIn } from '@apps/main/lib/route-guards'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_auth/find-password')({
  component: FindPasswordLayout,
  beforeLoad: ({ location }) => redirectIfLoggedIn(location)
})
