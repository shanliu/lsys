
import { Box, TextField } from '@mui/material';
import React, { useContext, useEffect, useState } from 'react';
import { Form } from 'react-router-dom';
import { ToastContext } from '../../../../common/context/toast';
import { LoadingButton } from '../../../../library/loading';
import { loadLoginConfigGet, loadLoginConfigSet } from '../../../../common/rest/setting';

export default function SystemSettingSiteSettingPage() {
    const { toast } = useContext(ToastContext);
    const [loadData, setLoadData] = useState({
        app_id: '',
        app_secret: '',
        loading: true
    });
    useEffect(() => {
        loadLoginConfigGet("wechat-get").then((data) => {
            if (!data.status) {
                setLoadData({
                    ...loadData,
                    loading: false,
                })
                toast(data.message);
                return;
            }
            setLoadData({
                loading: false,
                app_id: data.config.app_id ?? '',
                app_secret: data.config.app_secret ?? '',
            })
        })
    }, [])

    const onSave = (data) => {
        setLoadData({
            ...loadData,
            loading: true,
        })
        return loadLoginConfigSet("wechat-set", data).then((data) => {
            setLoadData({
                ...loadData,
                loading: false,
            })
            if (!data.status) {
                toast(data.message);
            } else {
                toast("完成保存");
            }
        })
    }

    return <Box sx={{
        m: 2,
        width: 480
    }}><Form method="post" onSubmit={(e) => {
        e.preventDefault();
        onSave(loadData)
    }}>
            <TextField
                sx={{
                    paddingBottom: 2
                }}
                label="微信 app id"
                variant="outlined"
                name="name"
                size="small"
                fullWidth
                disabled={loadData.loading}
                value={loadData.app_id}
                onChange={(e) => {
                    setLoadData({
                        ...loadData,
                        app_id: e.target.value
                    })
                }}
                required
            />
            <TextField
                label="微信 app secret"
                variant="outlined"
                name="name"
                size="small"
                fullWidth
                disabled={loadData.loading}
                value={loadData.app_secret}
                onChange={(e) => {
                    setLoadData({
                        ...loadData,
                        app_secret: e.target.value
                    })
                }}
                sx={{
                    paddingBottom: 2
                }}
                required
            />
            <LoadingButton disabled={loadData.loading} loading={loadData.loading} fullWidth variant="contained" type="submit">保存</LoadingButton>
        </Form >
    </Box>
}
