

import React, { useContext } from 'react';
import { UserSessionContext } from '../../../../../common/context/session';
import { useSearchChange } from '../../../../../common/utils/hook';
import AppSmsMessage from '../../../common/sender/sms_message';


export default function SystemAppSmsMessagePage(props) {
    const { userData } = useContext(UserSessionContext)
    const [searchParam, setSearchParam] = useSearchChange({
        tpl_id: '',
        mobile: '',
        status: '',
        start_pos: '',
        end_pos: '',
        page_size: 25,
    });
    return <AppSmsMessage
        appId={0}
        snId={searchParam.get("sn_id") ?? ''}
        tplId={searchParam.get("tpl_id") ?? ''}
        mobile={searchParam.get("mobile") ?? ''}
        status={searchParam.get("status") ?? ''}
        startPos={searchParam.get("start_pos") ?? ''}
        endPos={searchParam.get("end_pos") ?? ''}
        pageSize={searchParam.get("page_size") ?? 25}
        onSearchChange={setSearchParam}
    >
    </AppSmsMessage>
}


