
import React from 'react';
import SenderTplBodyPage from '../../../common/sender/tpls';
import { SenderTypeMail } from '../../../../../common/rest/sender_setting';


export default function SystemAppMailTplBodyPage(props) {
    return <SenderTplBodyPage userId={0} tplType={SenderTypeMail} />
}


