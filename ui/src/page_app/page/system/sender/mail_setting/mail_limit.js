
import React from 'react';
import { useSearchChange } from '../../../../../common/utils/hook';
import AppMailLimit from '../../../common/sender/mail_limit';

export default function SystemAppMailLimitPage(props) {
    const [searchParam, setSearchParam] = useSearchChange({
        id: '',
        page: 0,
        page_size: 25,
    });
    return <AppMailLimit
        appId={0}
        userId={0}
        limitId={searchParam.get("id") ?? ''}
        appName={'系统'}
        page={searchParam.get("page") ?? 0}
        pageSize={searchParam.get("page_size") ?? 25}
        onSearchChange={setSearchParam}
    >
    </AppMailLimit>
}


