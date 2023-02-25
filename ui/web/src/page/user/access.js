import React, { Fragment, useContext } from 'react';
import { UserSessionContext } from '../../context/session';
import { UserRolePage } from '../library/user_role';
import { PageNav } from './menu';

export default function UserAccessPage() {
    //页面数据初始化
    const { userData } = useContext(UserSessionContext)
    return <Fragment>
        <PageNav />
        <UserRolePage userId={parseInt(userData.user_data.user_id)} />
    </Fragment >
}