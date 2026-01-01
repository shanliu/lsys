import { default as IndexPage } from '@apps/main/features/common/pages/index-page'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/(root)/')({
  component: IndexPage,
})

