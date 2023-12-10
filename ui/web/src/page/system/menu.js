
import ApiIcon from '@mui/icons-material/Api';
import KeyIcon from '@mui/icons-material/Key';
import SettingsIcon from '@mui/icons-material/Settings';
import { Breadcrumbs } from '@mui/material';
import { useParams } from 'react-router';
import ManageAccountsIcon from '@mui/icons-material/ManageAccounts';
import { Link } from '@mui/material';
import Typography from '@mui/material/Typography';
import React from 'react';
import { Link as RouteLink } from 'react-router-dom';
import SmsIcon from '@mui/icons-material/Sms';
import MailIcon from '@mui/icons-material/Mail';
import HistoryOutlinedIcon from '@mui/icons-material/HistoryOutlined';
import IntegrationInstructionsOutlinedIcon from '@mui/icons-material/IntegrationInstructionsOutlined';
export const Menus = [

    {
        url: "/system/app",
        icon: ApiIcon,
        text: "应用审核",
        rbac: [{
            name: "admin-app"
        }]
    },
    {
        url: "/system/sender_sms/message",
        icon: SmsIcon,
        text: "短信管理",
        rbac: [{
            name: "admin-sms-config"
        }, {
            name: "admin-sender-config"
        }]
    },
    {
        url: "/system/sender_mail/message",
        icon: MailIcon,
        text: "邮件管理",
        rbac: [{
            name: "admin-mail-config"
        }, {
            name: "admin-sender-config"
        }]
    },
    {
        url: "/system/user",
        icon: ManageAccountsIcon,
        text: "用户管理",
        rbac: [{
            name: "admin-user"
        }]
    },
    {
        url: "/system/access/role",
        icon: KeyIcon,
        text: "授权管理",
        rbac: [{
            name: "res-view"
        }, {
            name: "role-view"
        }]
    },
    {
        url: "/system/docs",
        icon: IntegrationInstructionsOutlinedIcon,
        text: "开发文档",
        rbac: [{
            name: "docs-edit"
        }]
    },
    {
        url: "/system/logs",
        icon: HistoryOutlinedIcon,
        text: "系统日志",
        rbac: [{
            name: "admin-logs"
        }]
    },
    {
        url: "/system/setting/site/setting",
        icon: SettingsIcon,
        text: "全局设置",
        rbac: [
            //该菜单涉及所有接口权限列表
            {
                name: "admin-main"
            },
            {
                name: "admin-setting"
            }
        ]
    },
];




export function PageNav() {
    let param = useParams()//从请求url中获取数据
    let baeadTip = Menus.find((e) => {
        if (param["*"] != '' && e.url.indexOf(param["*"]) != -1) return true
    });
    return baeadTip ? (
        <Breadcrumbs >
            <Link component={RouteLink}
                underline="hover"
                sx={{ display: 'flex', alignItems: 'center' }}
                color="inherit"
                to=""
            >
                <ManageAccountsIcon sx={{ mr: 0.5 }} fontSize="inherit" />
                系统管理
            </Link>
            <Typography
                sx={{ display: 'flex', alignItems: 'center', color: '#999' }}

            >
                {baeadTip.text}
            </Typography></Breadcrumbs>
    ) : (<Breadcrumbs >
        <Typography
            sx={{ display: 'flex', alignItems: 'center', color: '#999' }}

        >
            系统管理
        </Typography></Breadcrumbs>)

}