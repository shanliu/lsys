import { SignInAppSearchSchema } from '@apps/main/features/auth/pages/sign-in-app-schema'
import { createFileRoute } from '@tanstack/react-router'
import { lazy } from 'react'

const SignInAppPage = lazy(() => import('@apps/main/features/auth/pages/sign-in-app-page'))

export const Route = createFileRoute('/_auth/sign-in/app')({
    component: SignInAppPage,
    validateSearch: SignInAppSearchSchema,
})
