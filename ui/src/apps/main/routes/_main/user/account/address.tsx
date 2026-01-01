import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary';
import { ProfileAddressPage } from '@apps/main/features/user/pages/account/address-page';
import { addressSearchSchema } from '@apps/main/features/user/pages/account/address-schema';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_main/user/account/address')({
  component: ProfileAddressPage,
  errorComponent: PageErrorBoundaryCreater({ variant: 'content' }),
  validateSearch: addressSearchSchema,
});
