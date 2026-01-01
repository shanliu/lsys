import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary';
import { AppServiceExterLoginPage } from '@apps/main/features/user/pages/app/detail/service/exter-login-page';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_main/user/app/$appId/service/exter-login')({
    component: AppServiceExterLoginPage,
    errorComponent: PageErrorBoundaryCreater({ variant: 'content' })
});
