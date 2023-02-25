

import React, { useContext } from 'react';
import { UserSessionContext } from '../../../context/session';
import { useSearchChange } from '../../../utils/hook';
import AppSmsMessage from '../../library/sms_message';
import { AppSelect } from '../../library/sms_app_select';
import { Button } from '@mui/material';
import LogoDevIcon from '@mui/icons-material/LogoDev';


export default function UserAppSmsMessagePage(props) {
    const { userData } = useContext(UserSessionContext)
    const [searchParam, setSearchParam] = useSearchChange({
        app_id: '',
        tpl_id: '',
        mobile: '',
        status: '',
        page: 0,
        page_size: 10,
    });
    return <AppSmsMessage
        userId={userData.user_data.user_id}
        appId={searchParam.get("app_id") ?? ''}
        tplId={searchParam.get("tpl_id") ?? ''}
        mobile={searchParam.get("mobile") ?? ''}
        status={searchParam.get("status") ?? ''}
        page={searchParam.get("page") ?? 0}
        pageSize={searchParam.get("page_size") ?? 10}
        onSearchChange={setSearchParam}
    >
        <Button
            variant="outlined"
            size="medium"
            startIcon={<LogoDevIcon />}
            sx={{ mr: 1, p: "7px 15px" }}
            onClick={() => {
                window.open("https://github.com/shanliu/lsys/tree/main/sdk/go/examples/basic/sms_test.go", "_blank")
            }}>
            接口调用示例
        </Button>
        <AppSelect
            userId={parseInt(userData.user_data.user_id)}
            appId={searchParam.get("app_id") ?? ''}
            onChange={(e) => {
                setSearchParam({
                    app_id: e.target.value,
                    page: 0
                })
            }}
        />
    </AppSmsMessage>
}


