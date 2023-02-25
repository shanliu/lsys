import React from 'react';
import { useNavigate } from 'react-router-dom';
import { TabLayout } from '../layout';




export default function UserInfoPage() {
    const InfoNav = [
        {
            key: "index",
            name: "用户资料"
        },
        {
            key: "name",
            name: "登录用户名"
        },
        {
            key: "password",
            name: "登录密码"
        },
        {
            key: "email",
            name: "绑定邮箱"
        },
        {
            key: "mobile",
            name: "绑定手机号"
        },
        {
            key: "oauth",
            name: "第三方账号"
        }
    ];
    const navigate = useNavigate();
    return <TabLayout onChange={
        (event, newValue) => {
            navigate('/user/info/' + newValue);
        }
    } menus={InfoNav} />

}