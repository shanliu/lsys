import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error';
import { Route } from '@apps/main/routes/_main/admin/email/adapter-config';
import { EmailAdapterConfigSmtpPage } from '@apps/main/features/admin/pages/email/adapter-config-smtp-page';

export function EmailAdapterConfigPage() {
    const search = Route.useSearch();
    
    const renderContent = () => {
        switch (search.type) {
            case "smtp": return <EmailAdapterConfigSmtpPage />
            default:
                return <CenteredError variant='page' error={"类型不支持"} />
        }
    };

    return (
        <div className="m-4 md:m-6">
            {renderContent()}
        </div>
    );
}
