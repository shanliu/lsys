
import React from 'react';
import { SmsLimitStatusMap } from '../../../../common/rest/sender_setting';
import { SenderLimit } from './lib_limit';



export default function AppSmsLimit(props) {

    return <SenderLimit
        limitType="smser"
        limitMapData={SmsLimitStatusMap}
        {...props}
    />
}


