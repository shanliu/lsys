
import AppRequestPromptPage from '@apps/main/features/user/pages/app/request-prompt-page'
import { createFileRoute } from '@tanstack/react-router'
import { z } from 'zod'

const RequestPromptSearchSchema = z.object({
  type: z.string().optional(),
})

export const Route = createFileRoute('/_main/user/app/$appId/request-prompt')({
  component: AppRequestPromptPage,
  validateSearch: RequestPromptSearchSchema,
})
