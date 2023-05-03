import React from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { TabLayout } from '../library/layout';




export default function UserAppMailPage() {
    const path = '/user/mail/'
    const showNav = [
        {
            key: "message",
            name: "邮件列表"
        },
        {
            key: "limit",
            name: "限额配置"
        },
        {
            key: "tpls",
            name: "邮件模板"
        },
        {
            key: "smtp_map",
            name: "SMTP服务器关联配置"
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