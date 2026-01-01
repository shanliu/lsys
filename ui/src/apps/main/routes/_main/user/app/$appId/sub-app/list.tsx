import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary';
import { SubAppListPage } from '@apps/main/features/user/pages/app/detail/sub-app/list-page';
import { SubAppListFilterParamSchema } from '@apps/main/features/user/pages/app/detail/sub-app/list-schema';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_main/user/app/$appId/sub-app/list')({
  component: SubAppListPage,
  validateSearch: SubAppListFilterParamSchema,
  errorComponent: PageErrorBoundaryCreater({ variant: 'content' })
});
