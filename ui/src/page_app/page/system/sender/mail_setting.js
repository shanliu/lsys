import React from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { TabLayout } from '../../common/layout';
export default function SystemSmsSettingPage() {
    const InfoNav = [
        {
            value: "message",
            name: "系统邮件列表"
        },
        {
            value: "limit",
            name: "系统邮件限额"
        },
        {
            value: "tpl_body",
            name: "系统邮件内容"
        },
        {
            value: "tpl_config",
            name: "系统邮件模板"
        },

        {
            to: "map_config/smtp",
            value: "map_config",
            name: "系统邮件端口"
        }
    ];
    const navigate = useNavigate();
    let param = useParams();
    let type = param['*'].split('/')[1];
    return <TabLayout value={type} onChange={
        (event, newValue) => {
            let find = InfoNav.find((item) => { return item.value == newValue })
            if (find) {
                let url = '/system/sender_mail/' + (find.to ? find.to : find.value);
                navigate(url);
            }

        }
    } menus={InfoNav} />
}
