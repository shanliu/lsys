import { SignInLayout } from '@apps/main/features/auth/components/layout/sign-in-layout'
import { SignInSearchSchema } from '@apps/main/features/auth/pages/sign-in-schema'
import { redirectIfLoggedIn } from '@apps/main/lib/route-guards'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_auth/sign-in')({
  component: SignInLayout,
  validateSearch: SignInSearchSchema,
  beforeLoad: ({ location }) => redirectIfLoggedIn(location)
})
