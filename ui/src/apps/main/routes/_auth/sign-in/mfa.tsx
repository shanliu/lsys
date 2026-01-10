import { SignInSearchSchema } from '@apps/main/features/auth/pages/sign-in-schema'
import { createFileRoute } from '@tanstack/react-router'
import { lazy } from 'react'

const SignInMfaPage = lazy(() => import('@apps/main/features/auth/pages/sign-in-mfa-page'))

export const Route = createFileRoute('/_auth/sign-in/mfa')({
    component: SignInMfaPage,
    validateSearch: SignInSearchSchema,
})
