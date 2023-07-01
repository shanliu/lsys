
import { Alert, FormControlLabel, Paper, Switch, TextField } from '@mui/material';
import Box from '@mui/material/Box';
import Tab from '@mui/material/Tab';
import Tabs from '@mui/material/Tabs';
import PropTypes from 'prop-types';
import React, { useContext, useEffect, useState } from 'react';
import { Form } from 'react-router-dom';
import { ToastContext } from '../../../context/toast';
import { LoadingButton } from '../../../library/loading';
import { loadSiteConfigGet, loadSiteConfigSet } from '../../../rest/setting';

export default function SystemSettingSiteSettingPage() {
    const { toast } = useContext(ToastContext);
    const [loadData, setLoadData] = useState({
        site_tips: '',
        password_timeout: 0,
        disable_old_password: false,
        loading:true
    });
    useEffect(() => {
        loadSiteConfigGet("base-get").then((data) => {
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
                password_timeout: data.config.timeout ?? 0,
                disable_old_password: !!data.config.dis_old_password,
                site_tips: data.config.site_tips ?? '',
            })
        })
    }, [])

    const onSave = (data) => {
        setLoadData({
            ...loadData,
            loading: true,
        })
        return loadSiteConfigSet("base-set", data).then((data) => {
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
        m:2,
        width:480
    }}> <Form method="post"  onSubmit={(e) => {
        e.preventDefault();
        onSave(loadData)
    }}>
        <TextField
            multiline
            minRows={2}
            sx={{
                paddingBottom: 2
            }}
            label="站点提示"
            variant="outlined"
            type="text"
            size="small"
            fullWidth
            disabled={loadData.loading}
            value={loadData.site_tips}
            onChange={(e) => {
                setLoadData({
                    ...loadData,
                    site_tips: e.target.value + ''
                })
            }}

        />
        <FormControlLabel
            sx={{
                paddingBottom: 2
            }}
            value='1'
            checked={!!loadData.disable_old_password}
            control={<Switch color="primary" disabled={loadData.loading} />}
            label="是否禁止使用旧密码"
            labelPlacement="start"
            onChange={(e) => {
                setLoadData({
                    ...loadData,
                    disable_old_password: !!e.target.checked
                })
            }}
        />
        <TextField
            sx={{
                paddingBottom: 2
            }}
            label="密码超时(秒)"
            variant="outlined"
            type="munber"
            size="small"
            fullWidth
            disabled={loadData.loading}
            value={loadData.password_timeout}
            onChange={(e) => {
                let n = parseInt(e.target.value ?? 0);
                if (isNaN(n)) return
                setLoadData({
                    ...loadData,
                    password_timeout: n
                })
            }}
            required
        />
        <LoadingButton disabled={loadData.loading} loading={loadData.loading} fullWidth variant="contained" type="submit">保存</LoadingButton>
    </Form >
    </Box>
}
