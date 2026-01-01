import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary';
import { AccountSetNamePage } from '@apps/main/features/user/pages/account/set-name-page';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_main/user/account/set-name')({
  component: AccountSetNamePage,
  errorComponent: PageErrorBoundaryCreater({ variant: 'content' }),
});
