import z from "zod"

// Define AppSecretItem schema based on API response
export const AppSecretItemSchema = z.object({
    secret_data: z.string(),
    time_out: z.date().nullable() // 实际过期时间，null表示永不过期
})

export type AppSecretItem = z.infer<typeof AppSecretItemSchema>

// Define OAuthSecretItem schema based on API response
export const OAuthSecretItemSchema = z.object({
    secret_data: z.string(),
    time_out: z.date().nullable() // 实际过期时间，null表示永不过期
})

export type OAuthSecretItem = z.infer<typeof OAuthSecretItemSchema>

// Secret form schema
export const SecretFormSchema = z.object({
    secret: z.string().optional(),
    secret_timeout: z.coerce.number().min(0, "超时时间不能小于0")
})

export type SecretFormData = z.infer<typeof SecretFormSchema>

