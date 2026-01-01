import { createFileRoute } from '@tanstack/react-router'
import { lazy } from 'react'

const SignUpSmsPage = lazy(() => import('@apps/main/features/auth/pages/sign-up-sms-page'))

export const Route = createFileRoute('/_auth/sign-up/sms')({
  component: SignUpSmsPage,
})
