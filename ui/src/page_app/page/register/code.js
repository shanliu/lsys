
import { Button, Dialog, DialogActions, DialogContent, DialogContentText, Grid, TextField, Tooltip } from '@mui/material';
import randomString from 'random-string';
import React, { Fragment, useContext, useEffect, useState } from 'react';

import ShortcutIcon from '@mui/icons-material/Shortcut';
import { Form, useNavigate } from 'react-router-dom';
import { SessionSetData, UserSessionContext } from '../../../common/context/session';
import { ToastContext } from '../../../common/context/toast';
import { CaptchaInput, PasswordInput } from '../../../library/input';
import { LoadingButton } from '../../../library/loading';
import { emailSignup, emailSignupSendCode, mobileSignup, mobileSignupSendCode, nameLogin } from '../../../common/rest/login';
import { captchaSrc } from '../../../common/utils/rest';
import { ConfigContext, ConfigTipPassword } from '../../../common/context/config';





function CodeSignupBox(props) {
    const { name, onPrev, type, label } = props;
    const [doData, setDoData] = useState({
        nikename: '',
        password: '',
        code: '',
        keep_login: true,
        loading: false,
    });
    const [fieldError, setFieldError] = useState({
        nikename: '',
        password: '',
        code: '',
    });
    const [loginData, setLoginData] = useState({
        open: false,
        loading: false,
    });
    const navigate = useNavigate();
    const { toast } = useContext(ToastContext);
    const { dispatch } = useContext(UserSessionContext);
    const configCtx = useContext(ConfigContext);
    const doHome = () => {
        navigate("/")
    }
    const doLogin = () => {
        let doAction;
        switch (type) {
            case 'email':
                doAction = nameLogin('email', {
                    name: name,
                    password: doData.password,
                });
                break;
            case 'text':
                doAction = nameLogin("sms", {
                    name: name,
                    password: doData.password,
                });
                break;
            default: return;
        }
        doAction.then((data) => {
            if (!data.status) {
                setLoginData({
                    ...loginData,
                    loading: false,
                })
                toast(data.message ?? "请手动完成登录");
                navigate(`/login/name`);
            } else {
                setLoginData({
                    ...loginData,
                    loading: false,
                })
                dispatch(SessionSetData(data, false))
                if (data.passwrod_timeout) {
                    configCtx.dispatch(ConfigTipPassword())
                }
                navigate("/");
            }
        })
    }
    const doSignup = () => {
        setDoData({
            ...doData,
            loading: true
        })
        let doAction;
        switch (type) {
            case 'email':
                doAction = emailSignup({
                    email: name,
                    nikename: doData.nikename,
                    password: doData.password,
                    code: doData.code,
                });
                break;
            case 'text':
                doAction = mobileSignup({
                    mobile: name,
                    nikename: doData.nikename,
                    password: doData.password,
                    code: doData.code,
                });
                break;
            default: return;
        }
        doAction.then((data) => {
            if (!data.status) {
                toast(data.message)
                setDoData({
                    ...doData,
                    loading: false
                })
                setFieldError({
                    ...fieldError,
                    ...data.field
                })
            } else {
                setDoData({
                    ...doData,
                    loading: false,
                })
                setLoginData({
                    ...loginData,
                    open: true,
                    loading: false,
                })
            }
        })
    }

    return <Fragment>
        <Dialog
            open={loginData.open}
            onClose={doHome}
        >
            <DialogContent sx={{
                minWidth: 350
            }}>
                <DialogContentText>
                    恭喜你，注册成功
                </DialogContentText>
            </DialogContent>
            <DialogActions>
                <LoadingButton onClick={doLogin} loading={loginData.loading} disabled={loginData.loading} autoFocus >完成登陆</LoadingButton>
                <Button onClick={doHome} >
                    暂不登陆
                </Button>
            </DialogActions>
        </Dialog>
        <Form method="post" onSubmit={(e) => {
            e.preventDefault();
            doSignup();
        }}>
            <Grid
                container
                justifyContent="center"
                alignItems="center"
            >
                <Grid item xs={10}>
                    <TextField
                        required
                        variant="outlined"
                        label={label}
                        type={type}
                        name="name"
                        size="small"
                        sx={{
                            width: 1,
                            paddingBottom: 2
                        }}
                        value={name}
                        disabled={doData.loading}
                        error={!!fieldError.name}
                        helperText={fieldError.name} />
                    <TextField
                        sx={{
                            width: 1,
                            paddingBottom: 2
                        }}
                        required
                        variant="outlined"
                        label={`验证码`}
                        type="text"
                        name="code"
                        size="small"
                        disabled={doData.loading}
                        onChange={(e) => {
                            setDoData({
                                ...doData,
                                code: e.target.value
                            })
                        }}
                        value={doData.code}
                        error={!!fieldError.code}
                        helperText={fieldError.code} />
                    <TextField
                        sx={{
                            width: 1,
                            paddingBottom: 2
                        }}
                        onChange={(e) => {
                            setDoData({
                                ...doData,
                                nikename: e.target.value
                            })
                        }}
                        value={doData.nikename}
                        required
                        variant="outlined"
                        label={`昵称`}
                        type="text"
                        name="nikename"
                        size="small"
                        disabled={doData.loading}
                        error={!!fieldError.nikename}
                        helperText={fieldError.nikename} />
                    <PasswordInput
                        sx={{
                            width: 1,
                            paddingBottom: 2
                        }}
                        onChange={(e) => {
                            setDoData({
                                ...doData,
                                password: e.target.value
                            })
                        }}
                        value={doData.password}
                        required
                        variant="outlined"
                        label={`登陆密码`}
                        name="password"
                        size="small"
                        disabled={doData.loading}
                        error={!!fieldError.password}
                        helperText={fieldError.password} />
                </Grid>

                <Grid item container xs={10} spacing={1}>
                    <Grid item xs={2}>
                        <Tooltip title={`重新获取验证码`} placement="right">
                            <Button variant="outlined" onClick={() => {
                                onPrev()
                            }} sx={{
                                width: 10,
                                minWidth: 50,
                                textAlign: "center"
                            }}>
                                <ShortcutIcon size="1em" color="inherit" sx={{
                                    transform: "rotateY(180deg)"
                                }} />
                            </Button>
                        </Tooltip>
                    </Grid>
                    <Grid item xs={10}>
                        <LoadingButton sx={{
                            width: 1,
                        }} variant="contained" type="submit" loading={doData.loading} disabled={doData.loading} >完成注册</LoadingButton>
                    </Grid>
                </Grid>
            </Grid>
        </Form > </Fragment>

}



function SendCodeBox(props) {
    const { captchaType, captchaKey, name, onNext, onChange, type, label } = props;
    const [doData, setDoData] = useState({
        captcha_val: '',
        loading: false,
        captcha_key: '',
        captcha_src: ''
    });
    useEffect(() => {
        setDoData({
            ...doData,
            captcha_val: '',
            captcha_key: captchaKey,
            captcha_src: captchaSrc(captchaType + "/" + captchaKey)
        })
    }, [props.captchaType, props.captchaKey])
    const [fieldError, setFieldError] = useState({
        captcha: '',
        name: '',
    });
    const { toast } = useContext(ToastContext);
    const doSendCode = () => {
        setDoData({
            ...doData,
            loading: true
        })
        let doAction;
        switch (type) {
            case 'email':
                doAction = emailSignupSendCode({
                    email: name,
                    captcha_key: doData.captcha_key,
                    captcha_code: doData.captcha_val,
                });
                break;
            case 'text':
                doAction = mobileSignupSendCode({
                    mobile: name,
                    captcha_key: doData.captcha_key,
                    captcha_code: doData.captcha_val,
                });
                break;
            default: return;
        }
        doAction.then((data) => {
            if (!data.status) {
                toast(data.message)
                let catpcha = {};
                if (data.is_captcha || data.field.captcha) {
                    catpcha = {
                        captcha_src: captchaSrc(captchaType + "/" + doData.captcha_key, true)
                    }
                }
                setDoData({
                    ...doData,
                    loading: false,
                    ...catpcha
                })
                setFieldError({
                    ...fieldError,
                    ...data.field
                })
            } else {
                setDoData({
                    ...doData,
                    loading: false,
                    captcha_val: ''
                })
                onNext();
            }
        })
    };

    return <Form method="post" onSubmit={(e) => {
        e.preventDefault();
        doSendCode();
    }}>
        <input type="hidden" name="submit_type" value="send_code" />
        <Grid
            container
            justifyContent="center"
            alignItems="center"
        >
            <Grid item xs={10}>
                <TextField
                    onChange={(e) => {
                        setDoData({
                            ...doData,
                            name: e.target.value
                        })
                        onChange(e.target.value)
                    }}
                    value={name}
                    required
                    variant="outlined"
                    name="name"
                    size="small"
                    sx={{
                        width: 1,
                        paddingBottom: 2
                    }}
                    disabled={doData.loading}
                    error={!!fieldError.name}
                    helperText={fieldError.name}
                    type={type}
                    label={label} />
            </Grid>
            <Grid item xs={10}>
                <CaptchaInput
                    value={doData.captcha_val}
                    onChange={(e) => {
                        setDoData({
                            ...doData,
                            captcha_val: e.target.value
                        })
                        setFieldError({
                            ...fieldError,
                            captcha: ''
                        })
                    }}
                    src={doData.captcha_src}
                    variant="outlined"
                    label="验证码"
                    type="text"
                    size="small"
                    required
                    disabled={doData.loading}
                    error={!!fieldError.captcha}
                    helperText={fieldError.captcha}
                /></Grid>
            <Grid item xs={10}>
                <LoadingButton sx={{
                    width: 1,
                }} variant="contained" type="submit" loading={doData.loading} disabled={doData.loading} >{`发送验证码`} </LoadingButton>
            </Grid>
        </Grid>
    </Form >
}


export function SignupFromCode(props) {
    const { sendCaptchaType, type, label } = props;
    const [doData, setDoData] = useState({
        is_send: true,
        val: '',
        send_code_key: randomString(),
    });
    useEffect(() => {
        setDoData({
            ...doData,
            val: '',
            is_send: true,
        })
    }, [props.sendCaptchaType])
    if (!doData.is_send) {
        return <CodeSignupBox
            name={doData.val}
            onPrev={() => {
                setDoData({
                    ...doData,
                    is_send: true,
                    send_code_key: randomString()
                })
            }}
            type={type}
            label={label}
        />
    } else {
        return <SendCodeBox
            type={type}
            label={label}
            captchaKey={doData.send_code_key}
            captchaType={sendCaptchaType}
            name={doData.val}
            onChange={(name) => {
                setDoData({
                    ...doData,
                    val: name,
                })
            }}
            onNext={() => {
                setDoData({
                    ...doData,
                    is_send: false,
                })
            }} />
    }
}