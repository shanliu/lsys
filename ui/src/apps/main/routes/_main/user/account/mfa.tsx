import { MfaPage } from '@apps/main/features/user/pages/account/mfa-page';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_main/user/account/mfa')({
    component: MfaPage,
});
