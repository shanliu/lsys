
import React from 'react';
import { useSearchChange } from '../../../../../common/utils/hook';
import AppSmsLimit from '../../../common/sender/sms_limit';

export default function SystemAppSmsLimitPage(props) {
    const [searchParam, setSearchParam] = useSearchChange({
        id: '',
        page: 0,
        page_size: 25,
    });
    return <AppSmsLimit
        appId={0}
        userId={0}
        limitId={searchParam.get("id") ?? ''}
        appName={'系统'}
        page={searchParam.get("page") ?? 0}
        pageSize={searchParam.get("page_size") ?? 25}
        onSearchChange={setSearchParam}
    >
    </AppSmsLimit>
}


