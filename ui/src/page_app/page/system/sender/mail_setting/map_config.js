
import React from 'react';
import { VerticalTabsLayout } from '../../../common/layout';
import { useNavigate, useParams } from 'react-router';

export default function SystemAppMailMapConfigPage(props) {
    const showNav = [
        {
            value: "smtp",
            name: "SMTP配置"
        },
    ];
    let param = useParams();
    let type = param['*'].split('/')[2];
    const navigate = useNavigate();
    return <VerticalTabsLayout value={type} onChange={
        (event, newValue) => {
            let find = showNav.find((item) => { return item.value == newValue })
            if (find) {
                let url = '/system/sender_mail/map_config/' + (find.to ? find.to : find.value);
                navigate(url);
            }
        }
    } menus={showNav} />
}