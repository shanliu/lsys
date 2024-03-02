

import React, { useContext, useState } from 'react';
import { UserSessionContext } from '../../../../common/context/session';
import { useSearchChange } from '../../../../common/utils/hook';
import AppSmsTplConfig from '../../common/sender/sms_tpl_config';
import { AppSelect } from '../../common/sender/lib_app_select';


export default function UserAppSmsTplConfigPage(props) {
    const { userData } = useContext(UserSessionContext)
    const [appData, setAppData] = useState({});
    const [searchParam, setSearchParam] = useSearchChange({
        id: '',
        app_id: '',
        page: 0,
        page_size: 25,
    });
    return <AppSmsTplConfig
        userId={userData.user_data.user_id}
        mapId={searchParam.get("id") ?? ''}
        appId={searchParam.get("app_id") ?? ''}
        appName={appData.name ?? ''}
        page={searchParam.get("page") ?? 0}
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
            onLoad={(data) => {
                setAppData(data)
            }}
            onChange={(e) => {
                setSearchParam({
                    app_id: e.target.value,
                    page: 0
                })
                if (e.target.value == '') {
                    setAppData({})
                }
            }}
        />
    </AppSmsTplConfig>
}


