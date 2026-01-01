import { createFileRoute } from '@tanstack/react-router'
import { lazy } from 'react'

const SignInMailPage = lazy(() => import('@apps/main/features/auth/pages/sign-in-mail-page'))

export const Route = createFileRoute('/_auth/sign-in/mail')({
  component: SignInMailPage,
})
