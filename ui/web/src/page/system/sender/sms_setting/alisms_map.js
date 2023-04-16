

import React from 'react';
import { useSearchChange } from '../../../../utils/hook';
import UserAppSmsAliSmsMap from '../../../library/sender/sms_map_alisms';



export default function SystemAppSmsAliSmsMapPage(props) {
    const [searchParam, setSearchParam] = useSearchChange({
        id: '',
        page: 0,
        page_size: 10,
    });
    return <UserAppSmsAliSmsMap
        appId={0}
        userId={0}
        mapId={searchParam.get("id") ?? ''}
        appName={'系统'}
        page={searchParam.get("page") ?? 0}
        pageSize={searchParam.get("page_size") ?? 10}
        onSearchChange={setSearchParam}
    >
    </UserAppSmsAliSmsMap>
}


