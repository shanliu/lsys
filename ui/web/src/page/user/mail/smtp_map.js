

import React, { useContext, useState } from 'react';
import { UserSessionContext } from '../../../context/session';
import { useSearchChange } from '../../../utils/hook';
import { AppSelect } from '../../library/sender/lib_app_select';
import AppMailSmtpMap from '../../library/sender/mail_map_smtp';


export default function UserAppMailSmtpMapPage(props) {
    const { userData } = useContext(UserSessionContext)
    const [appData, setAppData] = useState({});
    const [searchParam, setSearchParam] = useSearchChange({
        id: '',
        app_id: '',
        page: 0,
        page_size: 25,
    });
    return <AppMailSmtpMap
        userId={userData.user_data.user_id}
        mapId={searchParam.get("id") ?? ''}
        appId={searchParam.get("app_id") ?? ''}
        appName={appData.name ?? ''}
        page={searchParam.get("page") ?? 0}
        pageSize={searchParam.get("page_size") ?? 25}
        onSearchChange={setSearchParam}
    >
        <AppSelect
            checkMail={true}
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
    </AppMailSmtpMap>
}


