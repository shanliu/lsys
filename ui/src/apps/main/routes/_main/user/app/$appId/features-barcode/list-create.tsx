
import AppDetailFeatureBarCodeListCreatePage from '@apps/main/features/user/pages/app/detail/feature-barcode/list-create-page'
import { BarcodeCreateConfigFilterParamSchema } from '@apps/main/features/user/pages/app/detail/feature-barcode/list-create-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/user/app/$appId/features-barcode/list-create')({
  component: AppDetailFeatureBarCodeListCreatePage,
  validateSearch: BarcodeCreateConfigFilterParamSchema,
})

