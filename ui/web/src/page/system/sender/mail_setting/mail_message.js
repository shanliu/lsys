

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
        start_pos: '',
        end_pos: '',
        page_size: 10,
    });
    return <AppMailMessage
        appId={0}
        tplId={searchParam.get("tpl_id") ?? ''}
        to_mail={searchParam.get("to_mail") ?? ''}
        status={searchParam.get("status") ?? ''}
        startPos={searchParam.get("start_pos") ?? ''}
        endPos={searchParam.get("end_pos") ?? ''}
        pageSize={searchParam.get("page_size") ?? 10}
        onSearchChange={setSearchParam}
    >
    </AppMailMessage>
}


