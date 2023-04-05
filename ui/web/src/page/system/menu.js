
import KeyIcon from '@mui/icons-material/Key';
import SettingsIcon from '@mui/icons-material/Settings';
import ApiIcon from '@mui/icons-material/Api';
import ManageAccountsIcon from '@mui/icons-material/ManageAccounts';

export const Menus = [
    {
        url: "/system/setting",
        icon: SettingsIcon,
        text: "全局设置",
        rbac: [
            //该菜单涉及所有接口权限列表
            {
                name:"admin-main"
            },
            {
                name: "admin-setting"
            }
        ]
    },
    {
        url: "/system/app",
        icon: ApiIcon,
        text: "应用审核",
        rbac: [{
            name: "admin-app"
        }]
    },
    {
        url: "/system/sms_setting/message",
        icon: ApiIcon,
        text: "短信管理",
        rbac:  [{
            name: "admin-ali-sms-config"
        },{
            name: "admin-sender-config"
        }]
    },
    // {
    //     url: "/system/user",
    //     icon: ManageAccountsIcon,
    //     text: "用户管理",
    //     rbac: [{
    //         name: "admin-user"
    //     }]
    // },
    {
        url: "/system/access/test",
        icon: KeyIcon,
        text: "授权管理",
        rbac:  [{
            name: "res-view"
        },{
            name: "role-view"
        }]
    }
];