import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary';
import { AccountSetInfoPage } from '@apps/main/features/user/pages/account/set-info-page';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_main/user/account/set-info')({
  component: AccountSetInfoPage,
  errorComponent: PageErrorBoundaryCreater({ variant: 'content' }),
});
