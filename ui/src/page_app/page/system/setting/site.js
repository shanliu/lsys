
import React from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { VerticalTabsLayout } from '../../common/layout';

export default function SystemSettingSitePage() {
    const showNav = [
        {
            value: "setting",
            name: "站点设置"
        },
    ];
    let param = useParams();
    let type = param['*'].split('/')[2];
    const navigate = useNavigate();
    return <VerticalTabsLayout value={type} onChange={
        (event, newValue) => {
            let find = showNav.find((item) => { return item.value == newValue })
            if (find) {
                let url = '/system/setting/site/' + find.value;
                navigate(url);
            }
        }
    } menus={showNav} />
}