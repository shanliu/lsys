import { createFileRoute } from '@tanstack/react-router'
import { lazy } from 'react'

const SignUpMailPage = lazy(() => import('@apps/main/features/auth/pages/sign-up-mail-page'))

export const Route = createFileRoute('/_auth/sign-up/mail')({
  component: SignUpMailPage,
})
