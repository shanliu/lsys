
import React from 'react';
import { VerticalTabsLayout } from '../../../common/layout';
import { useNavigate, useParams } from 'react-router';

export default function SystemAppMailMapConfigPage(props) {
    const showNav = [
        {
            value: "alisms",
            name: "阿里云短信配置"
        },
        {
            value: "hwsms",
            name: "华为云短信配置"
        },
        {
            value: "tensms",
            name: "腾讯云短信配置"
        },
        {
            value: "jdsms",
            name: "京东云短信配置"
        }, {
            value: "cloopensms",
            name: "融连云短信配置"
        }, {
            value: "163sms",
            name: "网易云短信配置"
        }
    ];
    let param = useParams();
    let type = param['*'].split('/')[2];
    const navigate = useNavigate();
    return <VerticalTabsLayout navWidth={150} value={type} onChange={
        (event, newValue) => {
            let find = showNav.find((item) => { return item.value == newValue })
            if (find) {
                let url = '/system/sender_sms/map_config/' + (find.to ? find.to : find.value);
                navigate(url);
            }
        }
    } menus={showNav} />
}