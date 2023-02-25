import React from 'react';
import { useNavigate } from 'react-router-dom';
import { TabLayout } from '../layout';
export default function SystemAccessPage() {
    const InfoNav = [
        {
            key: "role",
            name: "系统角色"
        },
        {
            key: "res",
            name: "资源管理"
        },
    ];
    const navigate = useNavigate();
    return <TabLayout onChange={
        (event, newValue) => {
            navigate('/system/access/' + newValue);
        }
    } menus={InfoNav} />
}
