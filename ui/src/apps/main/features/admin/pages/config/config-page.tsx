import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error';
import { Route } from '@apps/main/routes/_main/admin/config';
import { OAuthConfigPage } from './oauth-config-page';
import { SiteConfigPage } from './site-config-page';

export function ConfigPage() {
    const router = Route.useSearch();
    
    const renderContent = () => {
        switch (router.type) {
            case "site": return <SiteConfigPage />
            case "oauth": return <OAuthConfigPage />
            default:
                return <CenteredError variant='page' error={"类型不支持"} />
        }
    };

    return (
        <div className="h-full flex flex-col  px-4 mt-4">
            {renderContent()}
        </div>
    );
}
