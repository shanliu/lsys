import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary';
import { AppNotifyPage } from '@apps/main/features/user/pages/app/detail/notify-page';
import { AppNotifyListFilterParamSchema } from '@apps/main/features/user/pages/app/detail/notify-schema';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_main/user/app/$appId/notify')({
    component: AppNotifyPage,
    validateSearch: AppNotifyListFilterParamSchema,
    errorComponent: PageErrorBoundaryCreater({ variant: 'content' })
});
