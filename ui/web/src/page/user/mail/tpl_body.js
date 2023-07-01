
import React, { useContext } from 'react';
import SenderTplBodyPage from '../../library/sender/tpls';
import { SenderTypeMail } from '../../../rest/sender_setting';
import { UserSessionContext } from '../../../context/session';


export default function UserAppMailTplBodyPage(props) {
    const { userData } = useContext(UserSessionContext)
    return <SenderTplBodyPage
        userId={userData.user_data.user_id}
        tplType={SenderTypeMail} />
}


