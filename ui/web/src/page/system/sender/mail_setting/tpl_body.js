
import React from 'react';
import SenderTplBodyPage from '../../../library/sender/tpls';
import { SenderTypeMail } from '../../../../rest/sender_setting';


export default function SystemAppMailTplBodyPage(props) {
    return <SenderTplBodyPage userId={0} tplType={SenderTypeMail} />
}


