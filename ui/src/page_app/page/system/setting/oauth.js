

import React from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { VerticalTabsLayout } from '../../common/layout';

export default function SystemSettingOauthPage() {

    const showNav = [
        {
            value: "wechat",
            name: "微信登录"
        },
    ];
    let param = useParams();
    let type = param['*'].split('/')[2];
    const navigate = useNavigate();
    return <VerticalTabsLayout value={type} onChange={
        (event, newValue) => {
            let find = showNav.find((item) => { return item.value == newValue })
            if (find) {
                let url = '/system/setting/oauth/' + find.value;
                navigate(url);
            }
        }
    } menus={showNav} />
}