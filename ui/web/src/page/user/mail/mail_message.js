

import React, { useContext } from 'react';
import { UserSessionContext } from '../../../context/session';
import { useSearchChange } from '../../../utils/hook';
import { AppSelect } from '../../library/sender/lib_app_select';
import { Button } from '@mui/material';
import LogoDevIcon from '@mui/icons-material/LogoDev';
import { AppMailMessage } from '../../library/sender/mail_message';


export default function UserAppMailMessagePage(props) {
    const { userData } = useContext(UserSessionContext)
    const [searchParam, setSearchParam] = useSearchChange({
        app_id: '',
        tpl_id: '',
        mobile: '',
        status: '',
        start_pos: '',
        end_pos: '',
        page_size: 25,
    });
    return <AppMailMessage
        userId={userData.user_data.user_id}
        appId={searchParam.get("app_id") ?? ''}
        tplId={searchParam.get("tpl_id") ?? ''}
        mobile={searchParam.get("mobile") ?? ''}
        status={searchParam.get("status") ?? ''}
        startPos={searchParam.get("start_pos") ?? ''}
        endPos={searchParam.get("end_pos") ?? ''}
        pageSize={searchParam.get("page_size") ?? 25}
        onSearchChange={setSearchParam}
    >
        <Button
            variant="outlined"
            size="medium"
            startIcon={<LogoDevIcon />}
            sx={{ mr: 1, p: "7px 15px", minWidth: 150 }}
            onClick={() => {
                window.open("https://github.com/shanliu/lsys/tree/main/sdk/go/examples/basic/mail_test.go", "_blank")
            }}>
            接口调用示例
        </Button>
        <AppSelect
            checkMail={true}
            userId={parseInt(userData.user_data.user_id)}
            appId={searchParam.get("app_id") ?? ''}
            onChange={(e) => {
                setSearchParam({
                    app_id: e.target.value,
                    page: 0
                })
            }}
        />
    </AppMailMessage>
}


