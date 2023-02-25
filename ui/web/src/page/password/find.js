
import { Button, Dialog, DialogActions, DialogContent, DialogContentText, Grid, TextField, Tooltip } from '@mui/material';
import randomString from 'random-string';
import React, { Fragment, useContext, useEffect, useState } from 'react';

import ShortcutIcon from '@mui/icons-material/Shortcut';
import { Form, useNavigate } from 'react-router-dom';
import { ToastContext } from '../../context/toast';
import { CaptchaInput, PasswordInput, ClearTextField } from '../../library/input';
import { LoadingButton } from '../../library/loading';
import { emailFindPassword, emailFindPasswordSendCode, mobileFindPassword, mobileFindPasswordSendCode } from '../../rest/login';
import { captchaSrc } from '../../utils/rest';



function CodeSetPasswordBox(props) {
    const { name, onPrev, type, label } = props;
    const [doData, setDoData] = useState({
        password: '',
        code: '',
        keep_login: true,
        loading: false,
        tips_open: false,
    });
    const [fieldError, setFieldError] = useState({
        password: '',
        code: '',
    });
    const navigate = useNavigate();
    const { toast } = useContext(ToastContext);
    const doLogin = () => {
        setDoData({
            ...doData,
            tips_open: false
        })
        navigate(`/login/name`);
    }
    const doSetPassword = () => {
        setDoData({
            ...doData,
            loading: true
        })
        let doAction;
        switch (type) {
            case 'email':
                doAction = emailFindPassword({
                    email: name,
                    password: doData.password,
                    code: doData.code,
                });
                break;
            case 'text':
                doAction = mobileFindPassword({
                    mobile: name,
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
                    tips_open: true
                })
            }
        })
    }

    return <Fragment>
        <Dialog
            open={doData.tips_open}
            onClose={doLogin}
        >
            <DialogContent sx={{
                minWidth: 350
            }}>
                <DialogContentText>
                    密码修改完成
                </DialogContentText>
            </DialogContent>
            <DialogActions>
                <Button onClick={doLogin} autoFocus>
                    前往登陆
                </Button>
            </DialogActions>
        </Dialog>
        <Form method="post" onSubmit={(e) => {
            e.preventDefault();
            doSetPassword();
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
                    <ClearTextField
                        sx={{
                            width: 1,
                            paddingBottom: 2
                        }}
                        required
                        variant="outlined"
                        label={`验证码`}
                        onChange={(e, nval) => {
                            setDoData({
                                ...doData,
                                code: nval
                            })
                        }}
                        type="text"
                        name="code"
                        size="small"
                        value={doData.code}
                        disabled={doData.loading}
                        error={!!fieldError.code}
                        helperText={fieldError.code} />
                    <PasswordInput
                        sx={{
                            width: 1,
                            paddingBottom: 2
                        }}
                        required
                        variant="outlined"
                        label={`新密码`}
                        name="password"
                        size="small"
                        onChange={(e) => {
                            setDoData({
                                ...doData,
                                password: e.target.value
                            })
                        }}
                        value={doData.password}
                        disabled={doData.loading}
                        error={!!fieldError.password}
                        helperText={fieldError.password} />
                </Grid>
                <Grid item container xs={10} spacing={1}>
                    <Grid item xs={2}>
                        <Tooltip title={`重新获取验证码`} placement="right">
                            <Button variant="outlined" onClick={onPrev} sx={{
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
                        }} variant="contained" type="submit" loading={doData.loading} disabled={doData.loading} >修改密码</LoadingButton>
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
            captcha_src: captchaSrc(captchaType + "/" + captchaKey, true)
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
                doAction = emailFindPasswordSendCode({
                    email: name,
                    captcha_key: doData.captcha_key,
                    captcha_code: doData.captcha_val,
                });
                break;
            case 'text':
                doAction = mobileFindPasswordSendCode({
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
                    label={label}
                    type={type}
                    name="name"
                    size="small"
                    sx={{
                        width: 1,
                        paddingBottom: 2
                    }}
                    disabled={doData.loading}
                    error={!!fieldError.name}
                    helperText={fieldError.name} />
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




export function FindPasswordBox(props) {
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
        return <CodeSetPasswordBox
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
