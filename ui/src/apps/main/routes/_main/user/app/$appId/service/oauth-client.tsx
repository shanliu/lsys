import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary';
import { AppServiceOauthClientPage } from '@apps/main/features/user/pages/app/detail/service/oauth-client-page';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_main/user/app/$appId/service/oauth-client')({
    component: AppServiceOauthClientPage,
    errorComponent: PageErrorBoundaryCreater({ variant: 'content' })
});
