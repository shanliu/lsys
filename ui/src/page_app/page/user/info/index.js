import { Alert, FormControl, InputLabel, MenuItem, Select, Stack, TextField } from '@mui/material';
import { Box } from '@mui/system';
import { AdapterDayjs } from '@mui/x-date-pickers/AdapterDayjs';
import { DesktopDatePicker } from '@mui/x-date-pickers/DesktopDatePicker';
import { LocalizationProvider } from '@mui/x-date-pickers/LocalizationProvider';
import dayjs from 'dayjs';
import React, { useContext, useEffect, useState } from 'react';
import { Form } from 'react-router-dom';
import { SessionReloadData, UserSessionContext } from '../../../../common/context/session';
import { ToastContext } from '../../../../common/context/toast';
import { LoadingButton, Progress } from '../../../../library/loading';
import { loginData } from '../../../../common/rest/login';
import { genderStatus, setInfo } from '../../../../common/rest/user';



export default function UserInfoIndexPage() {
    const getData = () => {
        return loginData({
            "info": true,
            "auth": true
        }).then((data) => {
            if (!data.status) return data;
            return {
                info: data.user_data.info,
                auth: data.auth_data,
                status: true
            }
        })
    };


    const [pageData, setPageData] = useState({
        init: false,
        loading: false,
        nikename: '',
        gender: 0,
        birthday: null,
        err: ''
    });

    useEffect(() => {
        getData().then((data) => {
            if (!data.status) {
                setPageData({
                    ...pageData,
                    init: true,
                    err: data.message
                })
            } else {
                let birthday = data.info?.birthday;
                if (typeof birthday == 'string' && birthday.length > 1) {
                    birthday = dayjs(data.info.birthday);
                } else {
                    birthday = null;
                }
                setPageData({
                    ...pageData,
                    init: true,
                    nikename: data.auth.user_nickname,
                    gender: data.info?.gender,
                    birthday: birthday,
                })
            }
        })
    }, [])

    const { userData, dispatch } = useContext(UserSessionContext)
    const { toast } = useContext(ToastContext);
    const onSave = () => {
        setPageData({
            ...pageData,
            loading: true
        })
        setInfo({
            nikename: pageData.nikename,
            gender: pageData.gender,
            birthday: pageData.birthday ? pageData.birthday.format('YYYY-MM-DD') : null
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
                    loading: false
                })
                if (userData.user_data.user_nickname != pageData.nikename) {
                    dispatch(SessionReloadData())
                }
                toast("设置完成")
            }
        })
    };

    return <Box sx={{
        margin: "auto auto",
        marginTop: 4,
        maxWidth: 400,
    }}>

        {!pageData.init ? <Stack sx={{ width: 1 }} spacing={2}>
            <Progress />
        </Stack> : <LocalizationProvider dateAdapter={AdapterDayjs}>
            {pageData.err ? <Alert severity="error">{pageData.err}</Alert> : null}
            <Form method="post" onSubmit={(e) => {
                e.preventDefault();
                onSave();
            }}>
                <TextField
                    variant="outlined"
                    label="用户昵称"
                    name="nikename"
                    size="medium"
                    sx={{
                        width: 1,
                        mb: 2
                    }}
                    disabled={pageData.loading}
                    onChange={(e) => {
                        setPageData({
                            ...pageData,
                            nikename: e.target.value
                        })
                    }}
                    value={pageData.nikename}
                    required
                />
                <FormControl size="medium" sx={{
                    width: 1,
                    mb: 2
                }}>
                    <InputLabel id="demo-multiple-chip-label">性别</InputLabel>
                    <Select
                        label="性别"
                        labelId="select-small"
                        id="select-small"
                        name="gender"
                        value={pageData.gender}
                        variant="outlined"
                        sx={{
                            width: 1,

                        }}
                        disabled={pageData.loading}
                        onChange={(event) => {
                            setPageData({
                                ...pageData,
                                gender: event.target.value
                            })
                        }}
                    >
                        {
                            genderStatus.map((status) => {
                                return <MenuItem key={`status_${status.key}`} value={status.key}>{status.val}</MenuItem>
                            })
                        }
                    </Select>
                </FormControl>
                <DesktopDatePicker
                    label="生日"
                    inputFormat="YYYY-MM-DD"
                    value={pageData.birthday}
                    onChange={(newValue) => {
                        setPageData({
                            ...pageData,
                            birthday: newValue
                        })
                    }}
                    variant="outlined"
                    size="small"
                    disabled={pageData.loading}
                    renderInput={(params) => <TextField name="birthday" {...params} sx={{
                        width: 1,
                        mb: 2
                    }} />}
                />
                <LoadingButton loading={pageData.loading} disabled={pageData.loading} size="large" sx={{
                    width: 1,
                }} variant="contained" type="submit">保存</LoadingButton>

            </Form >
        </LocalizationProvider>}
    </Box>
}