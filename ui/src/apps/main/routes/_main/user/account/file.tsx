import { PageErrorBoundaryCreater } from '@/apps/main/components/local/page-error-boundary';
import { AccountFilePage, FileFilterParamSchema } from '@apps/main/features/user/pages/account/file-page';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_main/user/account/file')({
    validateSearch: FileFilterParamSchema,
    component: AccountFilePage,
    errorComponent: PageErrorBoundaryCreater({ variant: 'content' }),
});
