import React from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { TabLayout } from '../../common/layout';
export default function SystemSmsSettingPage() {
    const InfoNav = [
        {
            value: "message",
            name: "系统短信列表"
        },
        {
            value: "limit",
            name: "系统短信限额"
        },
        {
            value: "tpl_config",
            name: "系统短信模板"
        },
        {
            to: "map_config/alisms",
            value: "map_config",
            name: "短信端口配置"
        }
    ];
    const navigate = useNavigate();
    let param = useParams();
    let type = param['*'].split('/')[1];
    return <TabLayout value={type} onChange={
        (event, newValue) => {
            let find = InfoNav.find((item) => { return item.value == newValue })
            if (find) {
                let url = '/system/sender_sms/' + (find.to ? find.to : find.value);
                navigate(url);
            }
        }
    } menus={InfoNav} />
}
