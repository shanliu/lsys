import { SignUpLayout } from '@apps/main/features/auth/components/layout/sign-up-layout'
import { redirectIfLoggedIn } from '@apps/main/lib/route-guards'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_auth/sign-up')({
  component: SignUpLayout,
  beforeLoad: ({ location }) => redirectIfLoggedIn(location)
})

