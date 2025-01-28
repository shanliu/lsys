
import AdminPanelSettingsIcon from '@mui/icons-material/AdminPanelSettings';
import HistoryIcon from '@mui/icons-material/History';
import KeyIcon from '@mui/icons-material/Key';
import PersonIcon from '@mui/icons-material/Person';
import { Breadcrumbs, Link } from '@mui/material';
import Typography from '@mui/material/Typography';
import React from 'react';
import { Link as RouteLink, useParams } from 'react-router-dom';
import ApiIcon from '@mui/icons-material/Api';
import SmsIcon from '@mui/icons-material/Sms';
import MailIcon from '@mui/icons-material/Mail';
import QrCodeIcon from '@mui/icons-material/QrCode';
import ManageAccountsIcon from '@mui/icons-material/ManageAccounts';
export const Menus = [
    {
        url: "/user/info/index",
        icon: ManageAccountsIcon,
        text: "用户设置",
        dep_account: true
    },
    {
        url: "/user/app",
        icon: ApiIcon,
        text: "应用管理"
    },
    {
        url: "/user/sms/message",
        icon: SmsIcon,
        text: "短信管理"
    },
    {
        url: "/user/mail/message",
        icon: MailIcon,
        text: "邮件管理"
    },
    {
        url: "/user/barcode/create",
        icon: QrCodeIcon,
        text: "条码管理"
    },
    {
        url: "/user/access",
        icon: KeyIcon,
        text: "授权管理"
    },
    {
        url: "/user/login_history",
        icon: HistoryIcon,
        text: "登陆历史",
        dep_account: true
    },
    {
        url: "/system/app",
        icon: AdminPanelSettingsIcon,
        text: "系统管理",
        rbac: [
            {
                name: "admin-main"
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
                <PersonIcon sx={{ mr: 0.5 }} fontSize="inherit" />
                用户中心
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
            用户中心
        </Typography></Breadcrumbs>)

}
