import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary';
import { AccountChangePasswordPage } from '@apps/main/features/user/pages/account/change-password-page';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_main/user/account/change-password')({
  component: AccountChangePasswordPage,
  errorComponent: PageErrorBoundaryCreater({ variant: 'content' }),
});
