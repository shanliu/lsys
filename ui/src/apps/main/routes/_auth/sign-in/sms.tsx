import { createFileRoute } from '@tanstack/react-router'
import { lazy } from 'react'

const SignInSmsPage = lazy(() => import('@apps/main/features/auth/pages/sign-in-sms-page'))

export const Route = createFileRoute('/_auth/sign-in/sms')({
  component: SignInSmsPage,
})
