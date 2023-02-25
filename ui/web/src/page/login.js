import { Box, Grid, Link, Paper, Tab, Tabs } from '@mui/material';
import React, { useContext, useEffect } from 'react';
import { Link as RouterLink, useNavigate, useParams } from 'react-router-dom';
import { UserSessionContext } from '../context/session';
import { CodeLoginPage } from './login/code';
import { PasswordLoginPage } from './login/password';
import { QrCodeLoginPage } from './login/qrcode';



export default function LoginPage() {
    const navigate = useNavigate();
    let onLogged = () => {
        const params = new URLSearchParams(window.location.hash.replace(/^.*\?/, ''));
        const redirect_uri = params.get("redirect_uri") + '';
        if (redirect_uri.length > 0
            && (
                redirect_uri.indexOf('http://' + window.location.host) === 0
                || redirect_uri.indexOf('https://' + window.location.host) === 0
            )) {
            window.location.href = redirect_uri;
            return;
        }
        navigate("/");
    }

    
    const { userData } = useContext(UserSessionContext);
    useEffect(() => {
        if (userData) {
            onLogged()
        }
    })
    let { type } = useParams();
    const LoginData = [
        {
            key: "name",
            name: "密码登录",
        },
        {
            key: "email",
            name: "邮箱验证码登录",
            props: {
                type: "email",
                label: "邮件",
                sendCaptchaType: "login-email",
                loginCaptchaType: "login"
            },
        },
        {
            key: "mobile",
            name: "短信验证码登录",
            props: {
                label: "手机号",
                type: "text",
                sendCaptchaType: "login-sms",
                loginCaptchaType: "login"
            },
        },
        {
            key: "wechat",
            name: "微信扫码登录"
        }
    ];
    let item = LoginData.find((e) => {
        if (e.key == type) {
            return true
        }
    });
    useEffect(() => {
        if (!item) {
            navigate("/login/" + LoginData[0].key);
        }
    }, [])
    if (!item) {
        return
    }

    let page;
    switch (type) {
        case "name":
            page = <PasswordLoginPage onLogged={onLogged} />
            break;
        case "wechat":
            page = <QrCodeLoginPage type="wechat" label="微信" onLogged={onLogged} />
            break;
        case "email":
        case "mobile":
            page = <CodeLoginPage {...item.props} onLogged={onLogged} />
            break;
        default:
    }
    const handleChange = (event, newValue) => {
        navigate("/login/" + newValue);
    };
    return (
        <Box className='center_page'>
            <Paper elevation={0}
                sx={{
                    margin: 'auto',
                    maxWidth: 525,
                    paddingTop: 2,
                    paddingBottom: 5,
                }}
            >
                <Box sx={{ width: '90%', margin: 'auto' }}>
                    <Box sx={{ borderBottom: 1, borderColor: 'divider', marginBottom: 3 }}>
                        <Tabs value={type} onChange={handleChange}>
                            {
                                LoginData.map((e) => {
                                    return <Tab label={e.name} value={e.key} key={`nav-${e.key}`} />
                                })
                            }
                        </Tabs>
                    </Box>
                    {page}
                    <Grid
                        container
                        justifyContent="center"
                        alignItems="center"
                        sx={{
                            marginTop: 2
                        }}
                    >
                        <Grid item xs={10} direction="row"
                            justifyContent="space-between"
                            alignItems="center" container>
                            <Link component={RouterLink} to="/find_password/email" underline="none"
                                variant="body2" >忘记密码？</Link >
                            <Link component={RouterLink} to="/register/email" underline="none"
                                variant="body2">注册</Link >

                        </Grid>

                    </Grid>
                </Box>
            </Paper>
        </Box >
    );
}