import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary';
import { AppServiceSubAppPage } from '@apps/main/features/user/pages/app/detail/service/sub-app-page';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_main/user/app/$appId/service/sub-app')({
    component: AppServiceSubAppPage,
    errorComponent: PageErrorBoundaryCreater({ variant: 'content' })
});
