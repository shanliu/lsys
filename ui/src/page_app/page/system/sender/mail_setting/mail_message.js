

import React, { useContext } from 'react';
import { UserSessionContext } from '../../../../../common/context/session';
import { useSearchChange } from '../../../../../common/utils/hook';
import { AppMailMessage } from '../../../common/sender/mail_message';


export default function SystemAppMailMessagePage(props) {
    const { userData } = useContext(UserSessionContext)
    const [searchParam, setSearchParam] = useSearchChange({
        tpl_id: '',
        to_mail: '',
        status: '',
        start_pos: '',
        end_pos: '',
        page_size: 25,
    });
    return <AppMailMessage
        appId={0}
        snId={searchParam.get("sn_id") ?? ''}
        tplId={searchParam.get("tpl_id") ?? ''}
        toMail={searchParam.get("to_mail") ?? ''}
        status={searchParam.get("status") ?? ''}
        startPos={searchParam.get("start_pos") ?? ''}
        endPos={searchParam.get("end_pos") ?? ''}
        pageSize={searchParam.get("page_size") ?? 25}
        onSearchChange={setSearchParam}
    >
    </AppMailMessage>
}


