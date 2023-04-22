import { Alert, Box, CssBaseline, List, ListItem, ListItemText, Paper, Stack, ThemeProvider, Typography } from '@mui/material';
import { default as React, Fragment, useContext, useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { UserProvider, UserSessionContext } from './context/session';
import { ToastProvider } from './context/toast';
import { LoadingButton, Progress } from './library/loading';
import { OauthDo, OauthGetScope } from './rest/login';
import "./style/main.css";
import { theme } from './style/theme';
import { ConfigProvider } from './context/config';
import { LayoutAppBar } from './bootstrap';




export default function OauthAppPage() {
    const [doData, setDoData] = useState({
        loading: true,
        error: '',
        scope: []
    });
    const params = new URLSearchParams(window.location.search);
    const scope = params.get("scope");
    const client_id = params.get("client_id");
    const redirect_uri = params.get("redirect_uri");
    const state = params.get("state");
    const { userData } = useContext(UserSessionContext)

    const finish = (code) => {
        let urls = redirect_uri.split("#");
        let url = urls.shift();
        let hashurl = urls.join("#");
        if (url.indexOf('?') == -1) {
            url = url + '?code=' + code;
        } else {
            url = url + '&code=' + code;
        }
        if (state && (state + '').length > 0) {
            url = url + '&state=' + encodeURIComponent(state);
        }
        if (hashurl && (hashurl + '').length > 0) {
            url += '#' + hashurl;
        }
        window.location.href = url;
    }

    useEffect(() => {
        if (!scope || scope == ''
            || !client_id || client_id == ''
            || !redirect_uri || redirect_uri == ''
        ) {
            setDoData({
                ...doData,
                loading: false,
                error: '请勿直接访问此页面',
            })
            return;
        }
        if (!userData) {
            setDoData({
                ...doData,
                loading: false,
                error: '未登陆，前往登陆中。。。。。',
            })
            let url = window.location.href.replace(/oauth\.html.*$/, "");
            url += "#/login/name?redirect_uri=" + encodeURIComponent(window.location.href);
            window.location.href = url
            return;
        }
        OauthGetScope(client_id, scope).then((data) => {
            if (!data.status) {
                setDoData({
                    ...doData,
                    loading: false,
                    error: data.message,
                })

            } else {
                setDoData({
                    ...doData,
                    loading: false,
                    scope: data.scope ?? [],
                })
                if (data.code) {
                    setOauthData({
                        loading: false,
                        error: "",
                        message: '完成过授权，返回请求登陆网站中',
                    })
                    finish(data.code)
                }
            }

        })
    }, [])
    const [oauthData, setOauthData] = useState({
        loading: false,
        error: '',
        message: ''
    });
    const doAuth = () => {
        setOauthData({
            ...oauthData,
            loading: true,
        })
        OauthDo(client_id, scope, redirect_uri).then((data) => {
            if (!data.status) {
                setOauthData({
                    loading: false,
                    error: data.message,
                    message: ""
                })
            } else {
                setOauthData({
                    loading: false,
                    error: "",
                    message: '完成授权，返回请求登陆网站中',
                })
                finish(data.code)
            }
        })
    }

    return (
        <Box className='center_page'>
            <Paper elevation={0}
                sx={{
                    margin: 'auto',
                    maxWidth: 350,
                    paddingTop: 2,
                    paddingBottom: 5,
                }}
            >
                <Box sx={{ width: '90%', margin: 'auto' }}>
                    <Fragment>
                        <Stack sx={{
                            alignItems: "center",
                        }}>
                            {

                                doData.loading ? <Progress /> :
                                    <Box sx={{
                                        width: 1,
                                    }}>

                                        <Paper elevation={1} align="center" sx={{
                                            paddingTop: 2,
                                            paddingBottom: 4
                                        }} >
                                            {oauthData.error ? <Alert sx={{
                                                m: "8px",
                                            }} severity='error' > {oauthData.error}</Alert> : ''}

                                            {doData.scope.length > 0 ?
                                                <Fragment> <Typography
                                                    align="center"
                                                    variant="subtitle1"
                                                    noWrap
                                                    sx={{
                                                        mt: "4px",
                                                        mb: "16px",
                                                        fontWeight: 100,
                                                        alignItems: "center",
                                                        letterSpacing: '.3rem',
                                                        color: 'inherit',
                                                        fontSize: "0.9rem",
                                                        textDecoration: 'none',
                                                    }}
                                                >
                                                    授权应用登录及以下权限
                                                </Typography>
                                                    {doData.error ? <Alert severity='error' > {doData.error}</Alert> : ""}
                                                    <List sx={{
                                                        border: "1px #ccc dotted",
                                                        background: "#fcfcfc",
                                                        margin: "24px",
                                                        borderRadius: "3px"
                                                    }}>
                                                        {doData.scope.map((item) => {
                                                            return <ListItem key={`scope-${item.name}`} >
                                                                <ListItemText
                                                                    primary={item.text ?? item.name}
                                                                    secondary={item.name}
                                                                />
                                                            </ListItem>
                                                        })}
                                                    </List>
                                                </Fragment> :
                                                <Typography
                                                    align="center"
                                                    variant="subtitle1"
                                                    noWrap
                                                    sx={{
                                                        mt: "4px",
                                                        mb: "16px",
                                                        fontWeight: 100,
                                                        alignItems: "center",
                                                        letterSpacing: '.3rem',
                                                        color: 'inherit',

                                                        textDecoration: 'none',
                                                    }}
                                                >
                                                    授权应用登录
                                                </Typography>}
                                            {doData.error ? <Alert severity='error' > {doData.error}</Alert> : ""}
                                            {!oauthData.message ? <LoadingButton variant="contained" loading={oauthData.loading} onClick={doAuth}>确认授权</LoadingButton> :
                                                <Alert sx={{
                                                    m: "8px",
                                                }} severity='info'>{oauthData.message}</Alert>}
                                        </Paper>
                                    </Box>
                            }
                        </Stack>
                    </Fragment>
                </Box>
            </Paper >
        </Box >
    );
}


function OauthApp() {
    return <>
        <ThemeProvider theme={theme} >
            <CssBaseline />
            <ConfigProvider>
                <ToastProvider>
                    <UserProvider>
                        <LayoutAppBar />
                        <OauthAppPage />
                    </UserProvider>
                </ToastProvider>
            </ConfigProvider>
        </ThemeProvider>
    </>;
}


const container = document.getElementById('root');
const root = createRoot(container);
root.render(<OauthApp />);

