import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary';
import { AccountLoginLogPage, LoginLogFilterParamSchema } from '@apps/main/features/user/pages/account/login-log-page';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_main/user/account/login-log')({
  validateSearch: LoginLogFilterParamSchema,
  component: AccountLoginLogPage,
  errorComponent: PageErrorBoundaryCreater({ variant: 'content' }),
});
