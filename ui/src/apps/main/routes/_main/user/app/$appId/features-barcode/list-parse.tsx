import AppDetailFeatureBarCodeListParsePage from '@apps/main/features/user/pages/app/detail/feature-barcode/list-parse-page'
import { BarcodeParseRecordFilterParamSchema } from '@apps/main/features/user/pages/app/detail/feature-barcode/list-parse-schema'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_main/user/app/$appId/features-barcode/list-parse')({
  component: AppDetailFeatureBarCodeListParsePage,
  validateSearch: BarcodeParseRecordFilterParamSchema,
})
