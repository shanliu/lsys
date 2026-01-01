import { createFileRoute } from '@tanstack/react-router'
import { lazy } from 'react'

const SignInAccountPage = lazy(() => import('@apps/main/features/auth/pages/sign-in-account-page'))

export const Route = createFileRoute('/_auth/sign-in/')({
  component: SignInAccountPage,

})
