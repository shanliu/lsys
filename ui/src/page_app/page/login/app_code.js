import { Alert, Grid } from '@mui/material';
import randomString from "random-string";
import React, { useContext, useEffect, useState } from 'react';
import { Form, useSearchParams } from 'react-router-dom';
import { SessionSetData, UserSessionContext } from '../../../common/context/session';
import { ToastContext } from '../../../common/context/toast';
import { CaptchaInput } from '../../../library/input';
import { LoadingButton } from '../../../library/loading';
import { appCodeLogin } from '../../../common/rest/login';
import { captchaSrc } from '../../../common/utils/rest';
import { ConfigContext } from '../../../common/context/config';


export function AppCodeLoginPage(props) {
  const [searchParam, _] = useSearchParams();
  const client_id = searchParam.get("client_id") ?? '';
  const login_code = searchParam.get("code") ?? '';
  const { onLogged } = props;
  const [loginData, setLoginData] = useState(() => {
    const captchaKey = randomString();
    return {
      captcha_val: '',
      loading: false,
      captcha_key: captchaKey,
      captcha_show: false,
      captcha_src: captchaSrc("login/" + captchaKey)
    }
  });
  const [fieldError, setFieldError] = useState({
    captcha: '',
    info: '',
  });
  const { toast } = useContext(ToastContext);
  const { dispatch } = useContext(UserSessionContext);

  const doLogin = () => {
    setLoginData({
      ...loginData,
      loading: true
    })
    appCodeLogin({
      client_id: client_id,
      login_code: login_code,
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
        if (data.state == 'not_login') {
          setFieldError({
            ...fieldError,
            info: "登录code异常或超时,请重新获取"
          })
        }
      } else {
        setLoginData({
          ...loginData,
          loading: false,
        })
        dispatch(SessionSetData(data, true))
        onLogged()
      }
    })
  }
  useEffect(() => {
    doLogin();
  }, []);
  return (
    <Form method="post"
      onSubmit={(e) => {
        e.preventDefault();
        doLogin();
      }}>
      {fieldError.info != '' ?
        <Alert severity='error' sx={{ m: 5 }} >
          {fieldError.info}
        </Alert>
        : <Grid
          container
          justifyContent="center"
          alignItems="center"
         
        >
          {loginData.loading ? <Grid item xs={10}>
            <Alert severity='info' sx={{ mb: 1 }} >登录中,请稍后</Alert>
          </Grid> : null}

          {loginData.captcha_show ? <Grid item xs={10}>
            <CaptchaInput
             sx={{ mt: 1}}
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
            <LoadingButton sx={{
              width: 1,
            }} variant="contained"
              type="submit"
              loading={loginData.loading}
              disabled={loginData.loading}
            >登录</LoadingButton>
          </Grid>
        </Grid>}

    </Form >
  );
};