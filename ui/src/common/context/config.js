import { Alert, Box, IconButton, Snackbar } from "@mui/material";
import React, { Fragment, createContext, useEffect, useReducer, useState } from "react";
import { PageProgress } from "../../library/loading";
import { loadSiteConfigInfo } from "../rest/setting";
import CloseIcon from '@mui/icons-material/Close';

export const ConfigPasswordTips = 'password-tips'
export const ConfigReload = 'reload'
export const ConfigData = 'data'
export const ConfigError = 'error'
export const ConfigDoReload = () => {
    return {
        type: ConfigReload
    }
}
export const ConfigTipPassword = () => {
    return {
        type: ConfigPasswordTips
    }
}


function SiteTips(props) {
    const { site_tips } = props
    const [open, setOpen] = useState(true)
    return <Snackbar
        anchorOrigin={{
            vertical: 'bottom',
            horizontal: 'left',
        }}
        open={open}
    >
        <Box>
            <Alert action={
                <IconButton
                    size="small"
                    aria-label="close"
                    color="inherit"

                    onClick={() => { setOpen(false) }}
                >
                    <CloseIcon fontSize="small" />
                </IconButton>
            } sx={{ m: 0, p: 1, mt: 1, paddingRight: 2 }} severity="info">{site_tips}</Alert>

        </Box>
    </Snackbar >
}

//登录信息处理 reducer 
const reducer = (data, action) => {
    switch (action.type) {
        case ConfigPasswordTips:
            return {
                ...data,
                password_tips: <Alert sx={{ m: 0, p: "2px 9px" }} severity='warning'><a href="#/user/info/password">密码已超时,建议修改密码</a></Alert>,
            };
        case ConfigReload:
            return {
                ...data,
                reload: true,
                error: null,
            };
        case ConfigError:
            return {
                ...data,
                reload: false,
                error: action.error,
            };
        case ConfigData:
            let site_tips = null;
            if (action.data.site_tips) {
                site_tips = <SiteTips site_tips={action.data.site_tips} />
            }
            delete action.data.site_tips;
            return {
                ...data,
                reload: false,
                error: '',
                site_tips: site_tips,
                config_data: action.data
            };
    }
}


export const ConfigContext = createContext({
    config: {},
    dispatch: () => { }
});

export const ConfigProvider = props => {
    let [configData, dispatch] = useReducer(reducer, null, () => {
        return {
            reload: true,
            error: null,
            password_tips: null,
            site_tips: null,
            config_data: {}
        }
    })
    useEffect(() => {
        if (!configData.reload) return
        loadSiteConfigInfo().then((data) => {
            if (!data.status || !data.data) {
                dispatch({
                    type: ConfigError,
                    error: "系统配置加载失败," + (data.message ?? '系统异常'),
                })
            } else {
                dispatch({
                    type: ConfigData,
                    data: data.data,
                })
            }
        })
    }, [configData.reload])
    return (
        <Fragment>

            {configData.site_tips}
            {configData.password_tips}
            {configData.loading ? <PageProgress /> :
                configData.error ? <Alert severity="error">{configData.error}</Alert> : <ConfigContext.Provider
                    value={{
                        config: configData.config_data ?? {},
                        dispatch: dispatch
                    }}
                >
                    {props.children}
                </ConfigContext.Provider >
            }
        </Fragment>
    )
}