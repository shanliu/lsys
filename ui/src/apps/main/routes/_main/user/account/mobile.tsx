import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary';
import { ProfileMobilePage } from '@apps/main/features/user/pages/account/mobile-page';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_main/user/account/mobile')({
  component: ProfileMobilePage,
  errorComponent: PageErrorBoundaryCreater({ variant: 'content' }),
});
