import React from 'react';
import { useNavigate } from 'react-router-dom';
import { TabLayout } from '../layout';
export default function SystemTestPage() {
    const InfoNav = [
        {
            key: "access",
            name: "权限测试"
        }
    ];
    const navigate = useNavigate();
    return <TabLayout onChange={
        (event, newValue) => {
            navigate('/system/test/' + newValue);
        }
    } menus={InfoNav} />
}
