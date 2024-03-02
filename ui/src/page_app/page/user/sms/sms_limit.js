
import React, { useContext, useState } from 'react';
import { UserSessionContext } from '../../../../common/context/session';
import { useSearchChange } from '../../../../common/utils/hook';
import AppSmsLimit from '../../common/sender/sms_limit';
import { AppSelect } from '../../common/sender/lib_app_select';


export default function UserAppSmsLimitPage(props) {
    const { userData } = useContext(UserSessionContext)
    const [appData, setAppData] = useState({});
    const [searchParam, setSearchParam] = useSearchChange({
        app_id: '',
        id: '',
        page: 0,
        page_size: 25,
    });
    return <AppSmsLimit
        userId={userData.user_data.user_id}
        appId={searchParam.get("app_id") ?? ''}
        limitId={searchParam.get("id") ?? ''}
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
    </AppSmsLimit>
}


