import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary';
import { ProfileExternalPage } from '@apps/main/features/user/pages/account/external-page';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_main/user/account/external')({
  component: ProfileExternalPage,
  errorComponent: PageErrorBoundaryCreater({ variant: 'content' }),
});
