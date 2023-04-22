
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

export default function SystemSettingSitePage() {
    return <SiteConfigTabs />
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


function SiteConfigTabs() {
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
        return loadSiteConfigSet(savePath, data).then((data) => {
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
            getPath: "base-get",
            text: "站点设置",
        }
    ];
    useEffect(() => {
        let data = Menus[tabVal] ?? null
        if (!data) return
        loadSiteConfigGet(data.getPath).then((data) => {
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
                    return <Tab key={`vertical-key-${e.getPath}`} label={e.text} id={`vertical-tab-${e.getPath}`} aria-controls={`vertical - tabpanel - ${e.getPath}`} />
                })}
            </Tabs>

            {Menus.map((e, i) => {
                let box = null;
                switch (e.getPath) {
                    case "base-get":
                        box = <SystemVarsConfig initData={loadData.data ?? {}} loading={loadData.loading} onSave={(data) => {
                            return onSave("base-set", data)
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


function SystemVarsConfig(props) {
    const { initData, loading, onSave } = props;
    const [loadData, setLoadData] = useState({
        site_tips: '',
        password_timeout: 0,
        disable_old_password: false
    });
    useEffect(() => {
        setLoadData({
            ...loadData,
            password_timeout: initData.timeout ?? 0,
            disable_old_password: !!initData.dis_old_password,
            site_tips: initData.site_tips ?? '',
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
            label="站点提示"
            variant="outlined"
            type="text"
            size="small"
            fullWidth
            disabled={loading}
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
            control={<Switch color="primary" />}
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
            disabled={loading}
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
        <LoadingButton disabled={loading} loading={loading} fullWidth variant="contained" type="submit">保存</LoadingButton>
    </Form >
}
