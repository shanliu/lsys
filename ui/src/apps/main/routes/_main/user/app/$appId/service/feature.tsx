import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary';
import { AppServiceFeaturePage } from '@apps/main/features/user/pages/app/detail/service/feature-page';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_main/user/app/$appId/service/feature')({
    component: AppServiceFeaturePage,
    errorComponent: PageErrorBoundaryCreater({ variant: 'content' })
});
