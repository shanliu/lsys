import React from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { TabLayout } from '../library/layout';




export default function UserInfoPage() {
    const InfoNav = [
        {
            value: "index",
            name: "用户资料"
        },
        {
            value: "name",
            name: "登录用户名"
        },
        {
            value: "password",
            name: "登录密码"
        },
        {
            value: "email",
            name: "绑定邮箱"
        },
        {
            value: "mobile",
            name: "绑定手机号"
        },
        // {
        //     value: "address",
        //     name: "收货地址"
        // },
        {
            value: "oauth",
            name: "第三方账号"
        }
    ];
    const navigate = useNavigate();
    let param = useParams();
    let type = param['*'].split('/')[1];
    return <TabLayout value={type} onChange={
        (event, newValue) => {
            let find = InfoNav.find((item) => { return item.value == newValue })
            if (find) {
                navigate('/user/info/' + find.value);
            }
        }
    } menus={InfoNav} />

}