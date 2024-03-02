import { Alert, Box, CssBaseline, List, ListItem, ListItemText, Paper, Stack, ThemeProvider, Typography } from '@mui/material';
import { default as React, Fragment, useContext, useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { UserProvider, UserSessionContext } from '../common/context/session';
import { ToastProvider } from '../common/context/toast';
import { LoadingButton, Progress } from '../library/loading';
import { OauthDo, OauthGetScope } from '../common/rest/login';
import "../common/style/oauth.css";
import { theme } from '../common/style/theme';
import { LayoutAppBar } from '../common/ui/layout';


export default function OauthAppPage() {
    const [doData, setDoData] = useState({
        loading: true,
        error: '',
        scope: [],
        show_box: true,
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
                show_box: false
            })
            return;
        }
        if (!userData) {
            setDoData({
                ...doData,
                loading: false,
                error: '未登陆，前往登陆中。。。。。',
            })
            let url = window.location.href.replace(/oauth\.html.*$/, "app.html");
            url += "#/login/name?redirect_uri=" + encodeURIComponent(window.location.href);
            window.location.href = url
            return;
        }
        OauthGetScope(client_id, scope).then((data) => {
            if (!data.status) {
                if (data.state == 'not_found') {
                    setDoData({
                        ...doData,
                        loading: false,
                        error: "应用不存在或被禁用",
                    })
                } else {
                    setDoData({
                        ...doData,
                        loading: false,
                        error: data.message,
                    })
                }

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

                        {doData.show_box ? <Stack sx={{
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
                                                <Fragment><Typography
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
                                                        borderRadius: "3px",

                                                    }}>
                                                        {doData.scope.map((item) => {
                                                            return <ListItem sx={{
                                                                pt: 0,
                                                                pb: 0
                                                            }} key={`scope-${item.key}`} >
                                                                <ListItemText
                                                                    primary={item.name ?? item.key}
                                                                    secondary={item.key}
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
                                            {doData.error ? <Alert severity='error' sx={{
                                                m: 1
                                            }} > {doData.error}</Alert> : ""}
                                            {!oauthData.message ? <LoadingButton variant="contained" loading={oauthData.loading} onClick={doAuth}>确认授权</LoadingButton> :
                                                <Alert sx={{
                                                    m: "8px",
                                                }} severity='info'>{oauthData.message}</Alert>}
                                        </Paper>
                                    </Box>
                            }
                        </Stack> : <Alert severity='error' sx={{
                            m: 1
                        }} > {doData.error}</Alert>}

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
            <ToastProvider>
                <UserProvider>
                    <LayoutAppBar />
                    <OauthAppPage />
                </UserProvider>
            </ToastProvider>
        </ThemeProvider>
    </>;
}


const container = document.getElementById('root');
const root = createRoot(container);
root.render(<OauthApp />);

