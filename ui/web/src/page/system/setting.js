

import React from 'react';
import { useNavigate } from 'react-router-dom';
import { TabLayout } from '../layout';
export default function SystemSettingPage() {
    const InfoNav = [
        {
            key: "site",
            name: "系统变量"
        },
        {
            key: "oauth",
            name: "第三方登陆"
        },
    ];
    const navigate = useNavigate();
    return <TabLayout onChange={
        (event, newValue) => {
            navigate('/system/setting/' + newValue);
        }
    } menus={InfoNav} />
}
