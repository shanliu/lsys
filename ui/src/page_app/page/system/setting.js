

import React from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { TabLayout } from '../common/layout';
export default function SystemSettingPage() {
    const InfoNav = [
        {
            value: "site",
            to: "site/setting",
            name: "系统变量"
        },
        {
            value: "oauth",
            to: "oauth/wechat",
            name: "第三方登陆"
        },
    ];
    const navigate = useNavigate();
    let param = useParams();
    let type = param['*'].split('/')[1];
    return <TabLayout value={type} onChange={
        (event, newValue) => {
            let find = InfoNav.find((item) => { return item.value == newValue })
            if (find) {
                let url = '/system/setting/' + (find.to ? find.to : find.value);
                navigate(url);
            }
        }
    } menus={InfoNav} />
}
