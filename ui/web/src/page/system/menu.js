
import KeyIcon from '@mui/icons-material/Key';
import SettingsIcon from '@mui/icons-material/Settings';
import ApiIcon from '@mui/icons-material/Api';
import ManageAccountsIcon from '@mui/icons-material/ManageAccounts';

export const Menus = [
    {
        url: "/system/setting",
        icon: SettingsIcon,
        text: "全局设置",
        rbac: [{
            key: "admin-setting",
            access: [{
                name: "admin-setting"
            }]
        }]
    },
    {
        url: "/system/app",
        icon: ApiIcon,
        text: "应用审核",
        rbac: [{
            key: "admin-app",
            access: [{
                name: "admin-app"
            }]
        }]
    },
    {
        url: "/system/sms_setting/message",
        icon: ApiIcon,
        text: "短信管理",
        rbac: [{
            key: "admin-sms",
            access: [{
                name: "admin-sms"
            }]
        }]
    },
    {
        url: "/system/user",
        icon: ManageAccountsIcon,
        text: "用户管理",
        rbac: [{
            key: "admin-user",
            access: [{
                name: "admin-user"
            }]
        }]
    },
    {
        url: "/system/access/test",
        icon: KeyIcon,
        text: "授权管理",
        rbac: [{
            key: "admin-test",
            access: [{
                name: "admin-test"
            }]
        }]
    }
];