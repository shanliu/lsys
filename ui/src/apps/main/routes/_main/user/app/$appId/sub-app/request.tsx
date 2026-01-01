import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary';
import { SubAppRequestPage } from '@apps/main/features/user/pages/app/detail/sub-app/request-page';
import { SubAppRequestFilterParamSchema } from '@apps/main/features/user/pages/app/detail/sub-app/request-schema';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_main/user/app/$appId/sub-app/request')({
  component: SubAppRequestPage,
  validateSearch: SubAppRequestFilterParamSchema,
  errorComponent: PageErrorBoundaryCreater({ variant: 'content' })
});
