

import React from 'react';
import { useSearchChange } from '../../../../utils/hook';
import AppMailTplConfig from '../../../library/sender/mail_tpl_config';

export default function SystemAppMailTplConfigPage(props) {
    const [searchParam, setSearchParam] = useSearchChange({
        id: '',
        page: 0,
        page_size: 25,
    });
    return <AppMailTplConfig
        appId={0}
        userId={0}
        mapId={searchParam.get("id") ?? ''}
        appName={'系统'}
        page={searchParam.get("page") ?? 0}
        pageSize={searchParam.get("page_size") ?? 25}
        onSearchChange={setSearchParam}
    >
    </AppMailTplConfig>
}


