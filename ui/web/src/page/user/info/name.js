import { Alert,  Stack, TextField } from '@mui/material';
import { Box } from '@mui/system';
import React, { useContext, useEffect, useState } from 'react';
import { Form, useNavigation } from 'react-router-dom';
import { useUpdateEffect } from 'usehooks-ts';
import { ToastContext } from '../../../context/toast';
import { LoadingButton, Progress } from '../../../library/loading';
import { loginData } from '../../../rest/login';
import { setUsername } from '../../../rest/user';

export default function UserInfoNamePage() {

    const [pageData, setPageData] = useState({
        init: false,
        err: '',
        loading: false,
        val: "",
        init_val: "",
    })
    const [nameError, setNameError] = useState({
        name: '',
    });
    useEffect(() => {
        loginData({
            "name": true,
        }).then((data) => {
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
                    val: data.user_data?.name?.username,
                    init_val: data.user_data?.name?.username,
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
        setPageData({
            ...pageData,
            loading: true
        })
        setUsername({
            name: pageData.val,
        }).then((data) => {
            if (!data.status) {
                setPageData({
                    ...pageData,
                    loading: false
                })
                setNameError({
                    ...nameError,
                    ...data.field
                })
                toast(data.message)
            } else {
                setPageData({
                    ...pageData,
                    loading: false,
                    init_val: pageData.val
                })
                toast("设置完成")
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
                    <Form method="post" onSubmit={(e) => {
                        e.preventDefault();
                        onSave();
                    }}>
                        <Alert severity="info">{
                            pageData.init_val ? `你当前可以使用[${pageData.init_val}]进行登陆` : "还未设置登陆用户名"
                        }</Alert>
                        <TextField
                            variant="outlined"
                            label="登陆用户名"
                            name="name"
                            size="medium"
                            disabled={pageData.loading}
                            sx={{
                                mt: 3,
                                width: 1,
                                pb: 3
                            }}
                            onChange={(e) => {
                                setPageData({
                                    ...pageData,
                                    val: e.target.value
                                })
                            }}
                            value={pageData.val}
                            required
                            error={!!nameError.name}
                            helperText={nameError.name}
                        />
                        <LoadingButton loading={pageData.loading} disabled={pageData.loading} size="large" sx={{
                            width: 1,
                        }} variant="contained" type="submit">保存</LoadingButton>

                    </Form >
        }

    </Box>

}