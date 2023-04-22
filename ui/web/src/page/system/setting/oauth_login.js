import React, { useContext, useEffect, useState } from 'react';
import PropTypes from 'prop-types';
import Tabs from '@mui/material/Tabs';
import Tab from '@mui/material/Tab';
import Typography from '@mui/material/Typography';
import Box from '@mui/material/Box';
import { Form } from 'react-router-dom';
import { Alert, Paper, Stack, TextField } from '@mui/material';
import { LoadingButton } from '../../../library/loading';
import { loadLoginConfigGet, loadLoginConfigSet } from '../../../rest/setting';
import { ToastContext } from '../../../context/toast';
export default function SystemSettingOauthLoginPage() {
    return <OauthConfigTabs />
}

function TabPanel(props) {
    const { children, value, index, ...other } = props;
    return (
        <div
            role="tabpanel"
            hidden={value !== index}
            id={`vertical-tabpanel-${index}`}
            aria-labelledby={`vertical-tab-${index}`}
            {...other}
        >
            {value === index && (
                <Box sx={{ p: 3 }}>
                    {children}
                </Box>
            )}
        </div>
    );
}

TabPanel.propTypes = {
    children: PropTypes.node,
    index: PropTypes.number.isRequired,
    value: PropTypes.number.isRequired,
};


function OauthConfigTabs() {
    const { toast } = useContext(ToastContext);
    const [tabVal, setTabVal] = useState(0);
    const [loadData, setLoadData] = useState({
        loading: true,
        data: {}
    });

    const onSave = (savePath, data) => {
        setLoadData({
            ...loadData,
            loading: true,
        })
        loadLoginConfigSet(savePath, data).then((data) => {
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

    const Menus = [
        {
            getPath: "wechat-get",
            text: "微信登录",
        }
    ];
    useEffect(() => {
        let data = Menus[tabVal] ?? null
        if (!data) return
        loadLoginConfigGet(data.getPath).then((data) => {
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
                data: data.config
            })
        })
    }, [tabVal])
    return <Paper
        sx={{ p: 2, display: 'flex', alignItems: 'flex-start', marginBottom: 1, marginTop: 1, minWidth: 900 }}
    >
        <Box
            sx={{ bgcolor: 'background.paper', display: 'flex', mt: 3 }}
        >
            <Tabs
                orientation="vertical"
                variant="scrollable"
                value={tabVal}
                onChange={(e) => {
                    setTabVal(e)
                }}
                aria-label="Vertical tabs example"
                sx={{ borderRight: 1, borderColor: 'divider', minWidth: 120 }}
            >
                {Menus.map((e) => {
                    return <Tab key={`vertical-${e.getPath}`} label={e.text} id={`vertical-tab-${e.getPath}`} aria-controls={`vertical - tabpanel - ${e.getPath}`} />
                })}
            </Tabs>

            {Menus.map((e, i) => {
                let box = null;
                switch (e.getPath) {
                    case "wechat-get":
                        box = <WechatConfig initData={loadData.data ?? {}} loading={loadData.loading} onSave={(data) => {
                            onSave("wechat-set", data)
                        }} />
                        break;
                }
                return <TabPanel key={`vertical-body-${e.getPath}`} value={tabVal} index={i}>
                    <Box sx={{
                        width: 300,
                    }}>
                        {box ? box : <Alert sx={{
                            m: "8px",
                        }} severity='info'>类型异常</Alert>}
                    </Box>
                </TabPanel>
            })}
        </Box >
    </Paper >
        ;
}


function WechatConfig(props) {
    const { initData, loading, onSave } = props;
    const [loadData, setLoadData] = useState({
        app_id: '',
        app_secret: '',
    });
    useEffect(() => {
        setLoadData({
            app_id: initData.app_id ?? '',
            app_secret: initData.app_secret ?? '',
        })
    }, [initData])
    return <Form method="post" onSubmit={(e) => {
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
            disabled={loading}
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
            disabled={loading}
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
        <LoadingButton disabled={loading} loading={loading} fullWidth variant="contained" type="submit">保存</LoadingButton>
    </Form >
}