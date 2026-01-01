import { ProfileEmailPage } from '@apps/main/features/user/pages/account/email-page';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_main/user/account/email')({
  component: ProfileEmailPage,
});
