import React from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { TabLayout } from '../layout';




export default function UserAppSmsPage() {
    const path = '/user/sms/'
    const showNav = [
        {
            key: "message",
            name: "信息列表"
        },
        {
            key: "limit",
            name: "限额配置"
        },
        {
            key: "alisms_map",
            name: "阿里云短信关联配置"
        }
    ];
    const [searchParam, _] = useSearchParams();
    let app_id = searchParam.get("app_id") ?? 0;
    const navigate = useNavigate();
    return <TabLayout onChange={
        (event, newValue) => {
            let url = path + newValue;
            if (app_id > 0) url += "?app_id=" + app_id;
            navigate(url);
        }
    } menus={showNav} />

}