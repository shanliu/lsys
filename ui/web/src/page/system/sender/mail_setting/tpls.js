
import React from 'react';
import SenderTplsPage from '../../../library/sender/tpls';
import { SenderTypeMail } from '../../../../rest/sender_setting';


export default function SystemAppMailTplsPage(props) {
    return <SenderTplsPage userId={0} tplType={SenderTypeMail} />
}


