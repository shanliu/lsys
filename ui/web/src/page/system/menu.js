
import KeyIcon from '@mui/icons-material/Key';
import SettingsIcon from '@mui/icons-material/Settings';
import ApiIcon from '@mui/icons-material/Api';
import ManageAccountsIcon from '@mui/icons-material/ManageAccounts';

export const Menus = [
    {
        url: "/system/setting",
        icon: SettingsIcon,
        text: "全局设置",
        rbac: [[
            {
                res: "admin",
                ops: ["view", "setting"]
            }
        ]]
    },
    {
        url: "/system/app",
        icon: ApiIcon,
        text: "应用审核",
        rbac: [[
            {
                res: "admin",
                ops: ["view"]
            },
            {
                res: "app",
                ops: ["global-app-confirm"]
            }
        ]]
    },
    {
        url: "/system/sms_setting/message",
        icon: ApiIcon,
        text: "短信管理",
        rbac: [[
            {
                res: "admin",
                ops: ["view"]
            },
            {
                res: "app",
                ops: ["alisms-config"]
            }
        ]]
    },
    {
        url: "/system/user",
        icon: ManageAccountsIcon,
        text: "用户管理",
        rbac: [[
            {
                res: "admin",
                ops: ["view"]
            },
            {
                res: "admin",
                ops: ["user"]
            }
        ]]
    },
    {
        url: "/system/access/role",
        icon: KeyIcon,
        text: "授权管理",
        rbac: [[
            {
                res: "admin",
                ops: ["view"]
            },
            {
                res: "rbac",
                ops: ["global-role-change", "global-role-view", "global-res-change", "global-res-view"]
            }
        ]]
    },
    {
        url: "/system/test/access",
        icon: KeyIcon,
        text: "功能测试",
        rbac: [[
            {
                res: "admin",
                ops: ["view", "test"]
            }
        ]]
    },
];