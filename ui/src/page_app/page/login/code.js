import ShortcutIcon from '@mui/icons-material/Shortcut';
import { Button, Checkbox, FormControlLabel, Grid, TextField, Tooltip } from '@mui/material';
import randomString from "random-string";
import React, { useContext, useEffect, useState } from 'react';
import { Form } from 'react-router-dom';
import { SessionSetData, UserSessionContext } from '../../../common/context/session';
import { ToastContext } from '../../../common/context/toast';
import { CaptchaInput, ClearTextField } from '../../../library/input';
import { LoadingButton } from '../../../library/loading';
import { emailCodeLogin, emailLoginSendCode, mobileCodeLogin, mobileLoginSendCode } from '../../../common/rest/login';
import { captchaSrc } from '../../../common/utils/rest';
import { ConfigContext, ConfigTipPassword } from '../../../common/context/config';

function CodeLoginBox(props) {
  const { captchaType, captchaKey, name, onPrev, type, label, onLogged } = props;
  const [doData, setDoData] = useState({
    code: '',
    keep_login: true,
    captcha_val: '',
    loading: false,
    captcha_key: captchaKey,
    captcha_show: false,
    captcha_src: ''
  });
  useEffect(() => {
    setDoData({
      ...doData,
      captcha_key: captchaKey,
      captcha_src: captchaSrc(captchaType + "/" + captchaKey)
    })
  }, [props.captchaType, props.captchaKey])
  const [fieldError, setFieldError] = useState({
    captcha: '',
    code: '',
  });
  const { toast } = useContext(ToastContext);
  const { dispatch } = useContext(UserSessionContext);
  const configCtx = useContext(ConfigContext);
  const doLogin = () => {
    setDoData({
      ...doData,
      loading: true
    })
    let doAction;
    switch (type) {
      case 'email':
        doAction = emailCodeLogin({
          email: name,
          code: doData.code,
          captcha_key: doData.captcha_show ? doData.captcha_key : '',
          captcha_code: doData.captcha_val,
        });
        break;
      case 'text':
        doAction = mobileCodeLogin({
          mobile: name,
          code: doData.code,
          captcha_key: doData.captcha_show ? doData.captcha_key : '',
          captcha_code: doData.captcha_val,
        });
        break;
      default: return;
    }
    doAction.then((data) => {
      if (!data.status) {
        toast(data.message)
        let catpcha = {};
        let field = {}
        if (
          data.field.captcha
          && data.data?.type != "captcha-login"
        ) {
          field = {
            code: data.field.captcha ?? '错误'
          }
        } else {
          if (data.is_captcha || (
            data.field.captcha
            && data.data?.type == "captcha-login"
          )) {
            catpcha = {
              captcha_show: true,
              captcha_src: captchaSrc(captchaType + "/" + doData.captcha_key, true)
            }
            field = data.field;
          }
        }
        setDoData({
          ...doData,
          loading: false,
          ...catpcha
        })
        setFieldError({
          ...fieldError,
          ...field
        })
      } else {
        setDoData({
          ...doData,
          loading: false,
        })
        dispatch(SessionSetData(data, doData.keep_login))
        if (data.passwrod_timeout) {
          configCtx.dispatch(ConfigTipPassword())
        }
        onLogged()

      }
    })
  };

  return <Form method="post"
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
          required
          variant="outlined"
          size="small"
          sx={{
            width: 1,
            paddingBottom: 2
          }}
          value={name}
          disabled={doData.loading}
          type={type}
          label={label}
        />
        <ClearTextField
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
          value={doData.code}
          disabled={doData.loading}
          onChange={(e, nval) => {
            setDoData({
              ...doData,
              code: nval
            })
            setFieldError({
              ...fieldError,
              code: ''
            })
          }}
          error={!!fieldError.code}
          helperText={fieldError.code} />
      </Grid>
      {doData.captcha_show ? <Grid item xs={10}>
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
        /></Grid> : null}
      <Grid item xs={10}>
        <FormControlLabel control={<Checkbox name="keep_login" value="1" defaultChecked={doData.keep_login}
          onChange={(e) => {
            setDoData({
              ...doData,
              keep_login: e.target.checked
            })
          }} />} label="记录登陆" />
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
          }} variant="contained" type="submit" loading={doData.loading} disabled={doData.loading} >登录</LoadingButton>
        </Grid>
      </Grid>
    </Grid>
  </Form >

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
        doAction = emailLoginSendCode({
          email: name,
          captcha_key: doData.captcha_key,
          captcha_code: doData.captcha_val,
        });
        break;
      case 'text':
        doAction = mobileLoginSendCode({
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
            captcha_show: true,
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
        }} variant="contained" type="submit"
          loading={doData.loading}
          disabled={doData.loading} >{`发送验证码`} </LoadingButton>
      </Grid>
    </Grid>
  </Form >
}
export function CodeLoginPage(props) {
  const { sendCaptchaType, loginCaptchaType, type, label, onLogged } = props;
  const [loginPage, setLoginPage] = useState({
    is_login: false,
    val: '',
    send_code_key: randomString(),
    login_key: '',
  });
  useEffect(() => {
    setLoginPage({
      ...loginPage,
      val: '',
      is_login: false,
      login_key: '',
    })
  }, [props.sendCaptchaType])
  if (loginPage.is_login) {
    return <CodeLoginBox
      captchaKey={loginPage.login_key}
      captchaType={loginCaptchaType}
      name={loginPage.val}
      onPrev={() => {
        setLoginPage({
          ...loginPage,
          is_login: false,
          send_code_key: randomString()
        })
      }}
      type={type}
      label={label}
      onLogged={onLogged}
    />
  } else {
    return <SendCodeBox
      type={type}
      label={label}
      captchaKey={loginPage.send_code_key}
      captchaType={sendCaptchaType}
      name={loginPage.val}
      onChange={(name) => {
        setLoginPage({
          ...loginPage,
          val: name,
        })
      }}
      onNext={() => {
        setLoginPage({
          ...loginPage,
          login_key: randomString(),
          is_login: true
        })
      }} />
  }
}
