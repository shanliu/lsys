
import React from 'react';
import { useSearchChange } from '../../../../utils/hook';
import AppMailLimit from '../../../library/sender/mail_limit';

export default function SystemAppMailLimitPage(props) {
    const [searchParam, setSearchParam] = useSearchChange({
        id: '',
        page: 0,
        page_size: 10,
    });
    return <AppMailLimit
        appId={0}
        userId={0}
        limitId={searchParam.get("id") ?? ''}
        appName={'系统'}
        page={searchParam.get("page") ?? 0}
        pageSize={searchParam.get("page_size") ?? 10}
        onSearchChange={setSearchParam}
    >
    </AppMailLimit>
}


