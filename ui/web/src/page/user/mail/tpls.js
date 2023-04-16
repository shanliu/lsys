
import React, { useContext } from 'react';
import SenderTplsPage from '../../library/sender/tpls';
import { SenderTypeMail } from '../../../rest/sender_setting';
import { UserSessionContext } from '../../../context/session';


export default function UserAppMailTplsPage(props) {
    const { userData } = useContext(UserSessionContext)
    return <SenderTplsPage
        userId={userData.user_data.user_id}
        tplType={SenderTypeMail} />
}


