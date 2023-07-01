
import React, { useContext, useState } from 'react';
import { UserSessionContext } from '../../../context/session';
import { useSearchChange } from '../../../utils/hook';
import { AppSelect } from '../../library/sender/lib_app_select';
import AppMailLimit from '../../library/sender/mail_limit';


export default function UserAppMailLimitPage(props) {
    const { userData } = useContext(UserSessionContext)
    const [appData, setAppData] = useState({});
    const [searchParam, setSearchParam] = useSearchChange({
        app_id: '',
        id: '',
        page: 0,
        page_size: 25,
    });
    return <AppMailLimit
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
                width:200,
                marginRight: 1
            }}
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
    </AppMailLimit>
}


