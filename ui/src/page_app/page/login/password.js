import { Checkbox, FormControlLabel, Grid, TextField } from '@mui/material';
import randomString from "random-string";
import React, { useContext, useState } from 'react';
import { Form, useNavigate } from 'react-router-dom';
import { SessionSetData, UserSessionContext } from '../../../common/context/session';
import { ToastContext } from '../../../common/context/toast';
import { CaptchaInput, PasswordInput } from '../../../library/input';
import { LoadingButton } from '../../../library/loading';
import { matchNameLogin } from '../../../common/rest/login';
import { captchaSrc } from '../../../common/utils/rest';
import { ConfigContext, ConfigTipPassword } from '../../../common/context/config';


export function PasswordLoginPage(props) {
    const { onLogged } = props;
    const [loginData, setLoginData] = useState(() => {
        const captchaKey = randomString();
        return {
            name: '',
            password: '',
            keep_login: true,
            captcha_val: '',
            loading: false,
            captcha_key: captchaKey,
            captcha_show: false,
            captcha_src: captchaSrc("login/" + captchaKey)
        }
    });
    const [fieldError, setFieldError] = useState({
        captcha: '',
        name: '',
        password: '',
    });
    const { toast } = useContext(ToastContext);
    const { dispatch } = useContext(UserSessionContext);
    const configCtx = useContext(ConfigContext);
    const navigate = useNavigate();
    const doLogin = () => {
        setLoginData({
            ...loginData,
            loading: true
        })
        matchNameLogin({
            name: loginData.name,
            password: loginData.password,
            captcha_key: loginData.captcha_show ? loginData.captcha_key : '',
            captcha_code: loginData.captcha_val,
        }).then((data) => {
            if (!data.status) {
                toast(data.message)
                let catpcha = {};
                if (data.is_captcha || data.field.captcha) {
                    catpcha = {
                        captcha_show: true,
                        captcha_src: captchaSrc("login/" + loginData.captcha_key, true)
                    }
                }
                setLoginData({
                    ...loginData,
                    loading: false,
                    ...catpcha
                })
                setFieldError({
                    ...fieldError,
                    ...data.field
                })
            } else {
                setLoginData({
                    ...loginData,
                    loading: false,
                })
                dispatch(SessionSetData(data, loginData.keep_login))
                if (data.passwrod_timeout) {
                    configCtx.dispatch(ConfigTipPassword())
                }
                onLogged()
            }
        })
    }
    return (
        <Form method="post"
            onSubmit={(e) => {
                e.preventDefault();
                doLogin();
            }}>
            <Grid
                container
                justifyContent="center"
                alignItems="center"
            >
                <Grid item xs={10}>
                    <TextField
                        variant="outlined"
                        label="账号"
                        type="text"
                        name="name"
                        size="small"
                        sx={{
                            width: 1,
                            paddingBottom: 2
                        }}
                        required
                        value={loginData.name}
                        onChange={(e) => {
                            setLoginData({
                                ...loginData,
                                name: e.target.value
                            })
                            setFieldError({
                                ...fieldError,
                                name: ''
                            })
                        }}
                        disabled={loginData.loading}
                        error={!!fieldError.name}
                        helperText={fieldError.name}
                        placeholder="用户名,邮箱,手机号"
                    />
                    <PasswordInput
                        sx={{
                            width: 1,
                            paddingBottom: 2
                        }}
                        required
                        variant="outlined"
                        label="密码"
                        value={loginData.password}
                        onChange={(e) => {
                            setLoginData({
                                ...loginData,
                                password: e.target.value
                            })
                            setFieldError({
                                ...fieldError,
                                password: ''
                            })
                        }}
                        name="password"
                        size="small"
                        disabled={loginData.loading}
                        placeholder="登录密码"
                        error={!!fieldError.password}
                        helperText={fieldError.password}
                    />
                </Grid>
                {loginData.captcha_show ? <Grid item xs={10}>
                    <CaptchaInput
                        value={loginData.captcha_val}
                        onChange={(e) => {
                            setLoginData({
                                ...loginData,
                                captcha_val: e.target.value
                            })
                            setFieldError({
                                ...fieldError,
                                captcha: ''
                            })
                        }}
                        src={loginData.captcha_src}
                        variant="outlined"
                        label="验证码"
                        type="text"
                        name="captcha_code"
                        size="small"
                        required
                        disabled={loginData.loading}
                        error={!!fieldError.captcha}
                        helperText={fieldError.captcha}
                    /></Grid> : null}
                <Grid item xs={10}>
                    <FormControlLabel control={<Checkbox name="keep_login" value="1" defaultChecked={loginData.keep_login}
                        onChange={(e) => {
                            setLoginData({
                                ...loginData,
                                keep_login: e.target.checked
                            })
                        }} />} label="记录登陆" />
                </Grid>
                <Grid item xs={10}>
                    <LoadingButton sx={{
                        width: 1,
                    }} variant="contained"
                        type="submit"
                        loading={loginData.loading}
                        disabled={loginData.loading}
                    >登录</LoadingButton>
                </Grid>
            </Grid>
        </Form >
    );
};