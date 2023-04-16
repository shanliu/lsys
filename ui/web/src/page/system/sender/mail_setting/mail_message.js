

import React, { useContext } from 'react';
import { UserSessionContext } from '../../../../context/session';
import { useSearchChange } from '../../../../utils/hook';
import { AppMailMessage } from '../../../library/sender/mail_message';


export default function SystemAppMailMessagePage(props) {
    const { userData } = useContext(UserSessionContext)
    const [searchParam, setSearchParam] = useSearchChange({
        tpl_id: '',
        to_mail: '',
        status: '',
        page: 0,
        page_size: 10,
    });
    return <AppMailMessage
        appId={0}
        tplId={searchParam.get("tpl_id") ?? ''}
        to_mail={searchParam.get("to_mail") ?? ''}
        status={searchParam.get("status") ?? ''}
        page={searchParam.get("page") ?? 0}
        pageSize={searchParam.get("page_size") ?? 10}
        onSearchChange={setSearchParam}
    >
    </AppMailMessage>
}


