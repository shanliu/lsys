

import React, { useContext } from 'react';
import { UserSessionContext } from '../../../context/session';
import { useSearchChange } from '../../../utils/hook';
import AppSmsMessage from '../../library/sender/sms_message';
import { AppSelect } from '../../library/sender/lib_app_select';


export default function UserAppSmsMessagePage(props) {
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

    return <AppSmsMessage
        userId={userData.user_data.user_id}
        appId={searchParam.get("app_id") ?? ''}
        tplId={searchParam.get("tpl_id") ?? ''}
        mobile={searchParam.get("mobile") ?? ''}
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
            checkSms={true}
            userId={parseInt(userData.user_data.user_id)}
            appId={searchParam.get("app_id") ?? ''}
            onChange={(e) => {
                setSearchParam({
                    app_id: e.target.value,
                    page: 0
                })
            }}
        />
    </AppSmsMessage>
}


