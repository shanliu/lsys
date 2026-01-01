import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary';
import { AppServiceSubAppOauthServerPage } from '@apps/main/features/user/pages/app/detail/service/sub-app-oauth-server-page';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_main/user/app/$appId/service/sub-app-oauth-server')({
    component: AppServiceSubAppOauthServerPage,
    errorComponent: PageErrorBoundaryCreater({ variant: 'content' })
});
