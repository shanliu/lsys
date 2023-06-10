

import React from 'react';
import { useSearchChange } from '../../../../utils/hook';
import AppMailSmtpMap from '../../../library/sender/mail_map_smtp';



export default function SystemAppMailSmtpMapPage(props) {
    const [searchParam, setSearchParam] = useSearchChange({
        id: '',
        page: 0,
        page_size: 25,
    });
    return <AppMailSmtpMap
        appId={0}
        userId={0}
        mapId={searchParam.get("id") ?? ''}
        appName={'系统'}
        page={searchParam.get("page") ?? 0}
        pageSize={searchParam.get("page_size") ?? 25}
        onSearchChange={setSearchParam}
    >
    </AppMailSmtpMap>
}


