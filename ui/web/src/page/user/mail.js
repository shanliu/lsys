import React from 'react';
import { useNavigate, useParams, useSearchParams } from 'react-router-dom';
import { TabLayout } from '../library/layout';




export default function UserAppMailPage() {
    const path = '/user/mail/'
    const showNav = [
        {
            value: "message",
            name: "邮件列表"
        },
        {
            value: "send",
            name: "邮件发送"
        },
        {
            value: "limit",
            name: "限额配置"
        },
        {
            value: "tpl_body",
            name: "模板内容"
        },
        {
            value: "tpl_config",
            name: "模板配置"
        }

    ];
    const [searchParam, _] = useSearchParams();
    let app_id = searchParam.get("app_id") ?? 0;
    const navigate = useNavigate();
    let param = useParams();
    let type = param['*'].split('/')[1];
    return <TabLayout value={type} onChange={
        (event, newValue) => {
            let find = showNav.find((item) => { return item.value == newValue })
            if (find) {
                let url = path + (find.to ? find.to : find.value);
                if (app_id > 0) url += "?app_id=" + app_id;
                navigate(url);
            }
        }
    } menus={showNav} />

}