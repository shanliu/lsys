import { Alert, AppBar, Container, CssBaseline, Grid, ThemeProvider, Toolbar, Typography } from '@mui/material';
import { Stack } from '@mui/system';
import React, { useState } from 'react';
import { createRoot } from 'react-dom/client';
import { LoadingButton } from '../library/loading';
import { wechatLogin } from '../rest/user';
import { theme } from '../style/theme';

function WxLoginApp() {
  const [doData, setDoData] = useState({
    loading: false,
    button: true,
  });
  const params = new URLSearchParams(window.location.search);
  const code = params.get("code");
  const state = params.get("state");
  if (!code || state == '' || !state || state == '') {
    setDoData({
      ...doData,
      loading: false,
      error: '请勿直接访问此页面',
    })
  } else {
    const doLogin = () => {
      setDoData({
        ...doData,
        loading: true,
        error: '',
        msg: ''
      })
      wechatLogin(code, state).then((data) => {
        if (!data.status) {
          setDoData({
            ...doData,
            loading: false,
            msg: data.message,
          })
          return
        }
        setDoData({
          ...doData,
          loading: false,
          button: false,
          msg: "已完成操作",
        })
      })
    }
  }


  return <>
    <ThemeProvider theme={theme} >
      <CssBaseline />
      <AppBar position="static" sx={{ zIndex: (theme) => theme.zIndex.drawer + 1, position: "relative" }}>
        <Container maxWidth="xl">
          <Toolbar disableGutters>
            <Typography
              variant="h6"
              noWrap
              component="a"
              href="/"
              sx={{
                mr: 2,
                display: { xs: 'none', md: 'flex' },
                fontFamily: 'monospace',
                fontWeight: 700,
                letterSpacing: '.3rem',
                color: 'inherit',
                textDecoration: 'none',
              }}
            >
              微信登录
            </Typography>
          </Toolbar>
        </Container>
      </AppBar>
      <Grid
        sx={{ width: 1, minHeight: "calc(100vh - 69px)" }}
        container
        justifyContent="center"
        alignItems="center"
      >
        {doData.error ? <Alert severity="error">{doData.error}</Alert> :
          (
            <Stack spacing={2}>
              {doData.msg ? <Alert severity="info">{doData.msg}</Alert> : ''}
              {doData.button ? <LoadingButton onClick={doLogin} variant="contained" loading={doData.loading}>确认授权</LoadingButton> : ''}
            </Stack>
          )
        }
      </Grid>
    </ThemeProvider>
  </>;
}


const container = document.getElementById('root');
const root = createRoot(container);
root.render(<WxLoginApp />);

