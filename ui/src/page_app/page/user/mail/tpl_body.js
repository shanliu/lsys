
import React, { useContext } from 'react';
import SenderTplBodyPage from '../../common/sender/tpls';
import { SenderTypeMail } from '../../../../common/rest/sender_setting';
import { UserSessionContext } from '../../../../common/context/session';


export default function UserAppMailTplBodyPage(props) {
    const { userData } = useContext(UserSessionContext)
    return <SenderTplBodyPage
        userId={userData.user_data.user_id}
        tplType={SenderTypeMail} />
}


