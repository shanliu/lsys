

import React, { useContext } from 'react';
import { UserSessionContext } from '../../../../common/context/session';
import { useSearchChange } from '../../../../common/utils/hook';
import { AppSelect } from '../../common/sender/lib_app_select';
import { AppMailMessage } from '../../common/sender/mail_message';


export default function UserAppMailMessagePage(props) {
    const { userData } = useContext(UserSessionContext)
    const [searchParam, setSearchParam] = useSearchChange({
        app_id: '',
        tpl_id: '',
        mobile: '',
        status: '',
        start_pos: '',
        end_pos: '',
        page_size: 25,
    });
    return <AppMailMessage
        toMail={searchParam.get("to_mail") ?? ''}
        userId={userData.user_data.user_id}
        snId={searchParam.get("sn_id") ?? ''}
        appId={searchParam.get("app_id") ?? ''}
        tplId={searchParam.get("tpl_id") ?? ''}
        status={searchParam.get("status") ?? ''}
        startPos={searchParam.get("start_pos") ?? ''}
        endPos={searchParam.get("end_pos") ?? ''}
        pageSize={searchParam.get("page_size") ?? 25}
        onSearchChange={setSearchParam}
    >

        <AppSelect
            sx={{
                width: 200,
                marginRight: 1
            }}
            checkMail={true}
            userId={parseInt(userData.user_data.user_id)}
            appId={searchParam.get("app_id") ?? ''}
            onChange={(e) => {
                setSearchParam({
                    app_id: e.target.value,
                    page: 0
                })
            }}
        />
    </AppMailMessage>
}


