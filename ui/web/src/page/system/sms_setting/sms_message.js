

import React, { useContext } from 'react';
import { UserSessionContext } from '../../../context/session';
import { useSearchChange } from '../../../utils/hook';
import AppSmsMessage from '../../library/sms_message';


export default function SystemAppSmsMessagePage(props) {
    const { userData } = useContext(UserSessionContext)
    const [searchParam, setSearchParam] = useSearchChange({
        tpl_id:'',
        mobile:'',
        status:'',
        page: 0,
        page_size: 10,
    });
    return <AppSmsMessage
        appId={0}
        tplId={searchParam.get("tpl_id")??''}
        mobile={searchParam.get("mobile")??''}
        status={searchParam.get("status")??''}
        page={searchParam.get("page")??0}
        pageSize={searchParam.get("page_size")??10}
        onSearchChange={setSearchParam}
    >
    </AppSmsMessage>
}


