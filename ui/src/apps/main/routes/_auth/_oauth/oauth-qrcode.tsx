import OAuthQrCodePage from '@/apps/main/features/auth/pages/oauth-qrcode'
import { OAuthQrCodeSearchSchema } from '@apps/main/features/auth/pages/oauth-qrcode-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_auth/_oauth/oauth-qrcode')({
  validateSearch: OAuthQrCodeSearchSchema,
  component: OAuthQrCodePage,
})
