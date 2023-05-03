import React from 'react';
import { useNavigate } from 'react-router-dom';
import { TabLayout } from '../../library/layout';
export default function SystemSmsSettingPage() {
    const InfoNav = [
        {
            key: "message",
            name: "系统短信列表"
        },
        {
            key: "alisms_map",
            name: "系统短信映射"
        },
        {
            key: "limit",
            name: "系统短信限额"
        },
        {
            key: "alisms",
            name: "阿里短信配置"
        }
    ];
    const navigate = useNavigate();
    return <TabLayout onChange={
        (event, newValue) => {
            navigate('/system/sender_sms/' + newValue);
        }
    } menus={InfoNav} />
}
