// system\sender_smser\hw_config_add.md
// system\sender_smser\hw_config_del.md
// system\sender_smser\hw_config_edit.md
// system\sender_smser\hw_config_list.md
// system\sender_smser\ali_config_add.md
// system\sender_smser\ali_config_del.md
// system\sender_smser\ali_config_edit.md
// system\sender_smser\ali_config_list.md
// system\sender_smser\cloopen_config_add.md
// system\sender_smser\cloopen_config_del.md
// system\sender_smser\cloopen_config_edit.md
// system\sender_smser\cloopen_config_list.md
// system\sender_smser\jd_config_add.md
// system\sender_smser\jd_config_del.md
// system\sender_smser\jd_config_edit.md
// system\sender_smser\jd_config_list.md
// system\sender_smser\netease_config_add.md
// system\sender_smser\netease_config_del.md
// system\sender_smser\netease_config_edit.md
// system\sender_smser\netease_config_list.md
// system\sender_smser\tencent_config_add.md
// system\sender_smser\tencent_config_del.md
// system\sender_smser\tencent_config_edit.md
// system\sender_smser\tencent_config_list.md
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error';
import { Route } from '@apps/main/routes/_main/admin/sms/adapter-config';
import { SmsAdapterConfigAliyunSmsPage } from './adapter-config-aliyun-page';
import { SmsAdapterConfigCloopenSmsPage } from './adapter-config-cloopen-page';
import { SmsAdapterConfigHuaweiSmsPage } from './adapter-config-huawei-page';
import { SmsAdapterConfigJdSmsPage } from './adapter-config-jd-page';
import { SmsAdapterConfigNeteaseSmsPage } from './adapter-config-netease-page';
import { SmsAdapterConfigTencentSmsPage } from './adapter-config-tencent-page';

export function SmsAdapterConfigPage() {
    const router = Route.useSearch();
    
    const renderContent = () => {
        switch (router.type) {
            case "aliyun": return <SmsAdapterConfigAliyunSmsPage />
            case "huawei": return <SmsAdapterConfigHuaweiSmsPage />
            case "cloopen": return <SmsAdapterConfigCloopenSmsPage />
            case "jd": return <SmsAdapterConfigJdSmsPage />
            case "netease": return <SmsAdapterConfigNeteaseSmsPage />
            case "tencent": return <SmsAdapterConfigTencentSmsPage />
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
