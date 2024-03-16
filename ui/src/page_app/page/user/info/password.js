import { Alert, Stack } from '@mui/material';
import { Box } from '@mui/system';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { Form, useNavigation } from 'react-router-dom';
import { useUpdateEffect } from 'usehooks-ts';
import { ToastContext } from '../../../../common/context/toast';
import { PasswordInput } from '../../../../library/input';
import { LoadingButton, Progress } from '../../../../library/loading';
import { getPasswordModify, setPassword } from '../../../../common/rest/user';
import { showTime } from '../../../../common/utils/utils';


export default function UserInfoPasswordPage() {

    const [pageData, setPageData] = useState({
        init: false,
        err: '',
        loading: false,
        val: '',
        old_val: '',
        last_time: 0,
        password_timeout: false,
    })

    useEffect(() => {
        getPasswordModify().then((data) => {
            if (!data.status) {
                setPageData({
                    ...pageData,
                    init: true,
                    err: data.message
                })
            } else {
                setPageData({
                    ...pageData,
                    init: true,
                    last_time: data.last_time ?? 0,
                    password_timeout: data.password_timeout,
                })
            }
        })
    }, [])


    const navigation = useNavigation();
    useUpdateEffect(() => {
        if (navigation.state == 'submitting') {
            setPageData({
                ...pageData,
                loading: true
            })
        }
    }, [navigation]);

    const { toast } = useContext(ToastContext);
    const onSave = () => {
        setPassword({
            skip_old: pageData.last_time == 0,
            old_password: pageData.old_val,
            new_password: pageData.val
        }).then((data) => {
            if (!data.status) {
                setPageData({
                    ...pageData,
                    loading: false
                })
                toast(data.message)
            } else {
                setPageData({
                    ...pageData,
                    loading: false,
                    val: '',
                    old_val: '',
                    password_timeout: false,
                    last_time: parseInt(new Date().getTime() / 1000)
                })
                if (pageData.last_time == 0) { toast("密码已设置") }
                else { toast("密码已修改") }
            }
        })
    };


    return <Box sx={{
        margin: "auto auto",
        marginTop: 4,
        maxWidth: 400,
    }}>
        {
            !pageData.init ? <Stack sx={{ width: 1 }} spacing={2}>
                <Progress />
            </Stack> :
                pageData.err ? <Alert severity="error">{pageData.err}</Alert> :
                    <Fragment>
                        {
                            pageData.last_time > 0 && pageData.password_timeout ?
                                <Alert sx={{ mb: 2 }} severity="warning">你已经长时间未修改密码,建议立即修改</Alert>
                                : null
                        }
                        {
                            pageData.last_time == 0 ?
                                <Alert severity="info">还未设置登陆密码</Alert>
                                : <Alert severity="info">最后一次修改密码时间: {showTime(pageData.last_time)}</Alert>
                        }
                        <Form method="post" onSubmit={(e) => {
                            e.preventDefault();
                            onSave();
                        }}>
                            {
                                pageData.last_time > 0 ?
                                    <PasswordInput
                                        variant="outlined"
                                        label="输入原密码"
                                        name="old_password"
                                        size="medium"
                                        sx={{
                                            mt: 3,
                                            width: 1,

                                        }}
                                        onChange={(e) => {
                                            setPageData({
                                                ...pageData,
                                                old_val: e.target.value
                                            })
                                        }}
                                        value={pageData.old_val}
                                        required
                                    /> : ""
                            }

                            <PasswordInput
                                variant="outlined"
                                label={pageData.last_time > 0 ? "输入新密码" : "请输入密码"}
                                name="new_password"
                                size="medium"
                                sx={{
                                    mt: 3,
                                    width: 1,

                                }}
                                onChange={(e) => {
                                    setPageData({
                                        ...pageData,
                                        val: e.target.value
                                    })
                                }}
                                value={pageData.val}
                                required
                            />
                            <LoadingButton size="large" sx={{
                                mt: 3,
                                width: 1,
                            }} variant="contained" type="submit">修改密码</LoadingButton>
                        </Form >
                    </Fragment>
        }

    </Box>

}