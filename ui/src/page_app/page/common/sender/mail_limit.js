
import React from 'react';
import { MailLimitStatusMap } from '../../../../common/rest/sender_setting';
import { SenderLimit } from './lib_limit';
export default function AppMailLimit(props) {
    return <SenderLimit
        limitType="mailer"
        limitMapData={MailLimitStatusMap}
        {...props}
    />
}


