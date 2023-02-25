

import React, { useContext, useState } from 'react';
import { UserSessionContext } from '../../../context/session';
import { useSearchChange } from '../../../utils/hook';
import AppSmsAliSmsMap from '../../library/sms_map_alisms';
import { AppSelect } from '../../library/sms_app_select';


export default function UserAppSmsAliSmsMapPage(props) {
    const { userData } = useContext(UserSessionContext)
    const [appData, setAppData] = useState({});
    const [searchParam, setSearchParam] = useSearchChange({
        id:  '',
        app_id:  '',
        page: 0,
        page_size: 10,
    });
    return <AppSmsAliSmsMap
        userId={userData.user_data.user_id}
        mapId={searchParam.get("id")??''}
        appId={searchParam.get("app_id")??''}
        appName={appData.name??''}
        page={searchParam.get("page")??0}
        pageSize={searchParam.get("page_size")??10}
        onSearchChange={setSearchParam}
    >
        <AppSelect 
                userId={parseInt(userData.user_data.user_id)}
                appId={searchParam.get("app_id")??''} 
                onLoad={(data)=>{
                    setAppData(data)
                }}
                onChange={(e) => {
                    setSearchParam({
                        app_id:e.target.value,
                        page: 0
                    })
                    if(e.target.value==''){
                        setAppData({})
                    }
                }}
            />
    </AppSmsAliSmsMap>
}


