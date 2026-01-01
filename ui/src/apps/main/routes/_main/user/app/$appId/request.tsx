import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary';
import { AppRequestPage } from '@apps/main/features/user/pages/app/detail/request-page';
import { AppRequestListFilterParamSchema } from '@apps/main/features/user/pages/app/detail/request-schema';

import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_main/user/app/$appId/request')({
    component: AppRequestPage,
    validateSearch: AppRequestListFilterParamSchema,
    errorComponent: PageErrorBoundaryCreater({ variant: 'content' })
});
