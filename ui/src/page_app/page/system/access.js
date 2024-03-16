import React from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { TabLayout } from '../common/layout';
export default function SystemAccessPage() {
    const InfoNav = [
        // {
        //     value: "test",
        //     name: "授权测试"
        // },
        {
            value: "role",
            name: "系统角色"
        },
        {
            value: "res",
            name: "资源管理"
        }
    ];
    const navigate = useNavigate();
    let param = useParams();
    let type = param['*'].split('/')[1];
    return <TabLayout value={type} onChange={
        (event, newValue) => {
            let find = InfoNav.find((item) => { return item.value == newValue })
            if (find) {
                navigate('/system/access/' + find.value);
            }
        }
    } menus={InfoNav} />
}
