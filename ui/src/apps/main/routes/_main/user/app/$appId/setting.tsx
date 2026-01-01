import { PageErrorBoundaryCreater } from '@apps/main/components/page-error-boundary';
import { AppSettingPage } from '@apps/main/features/user/pages/app/detail/setting-page';
import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

const SettingSearchSchema = z.object({
    secret: z.coerce.number().refine((val) => val === 0 || val === 1, {
        message: "secret must be 0 or 1"
    }).optional().catch(undefined)
});

export const Route = createFileRoute('/_main/user/app/$appId/setting')({
    component: AppSettingPage,
    errorComponent: PageErrorBoundaryCreater({ variant: 'content' }),
    validateSearch: (search) => SettingSearchSchema.parse(search)
});
