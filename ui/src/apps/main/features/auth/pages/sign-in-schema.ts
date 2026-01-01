import { z } from 'zod'

export const SignInSearchSchema = z.object({
    redirect_uri: z.string().optional(),
    from: z.string().optional(),
})

export type SignInSearchType = z.infer<typeof SignInSearchSchema>
