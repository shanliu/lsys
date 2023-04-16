import React from 'react';
import { useNavigate } from 'react-router-dom';
import { TabLayout } from '../../layout';
export default function SystemSmsSettingPage() {
    const InfoNav = [
        {
            key: "message",
            name: "系统邮件列表"
        },
        {
            key: "smtp_map",
            name: "系统邮件映射"
        },
        {
            key: "tpls",
            name: "邮件模板"
        },
        {
            key: "limit",
            name: "系统邮件限额"
        },
        {
            key: "smtp",
            name: "SMTP配置"
        }
    ];
    const navigate = useNavigate();
    return <TabLayout onChange={
        (event, newValue) => {
            navigate('/system/sender_mail/' + newValue);
        }
    } menus={InfoNav} />
}
