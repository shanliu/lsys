import { CssBaseline, ThemeProvider } from '@mui/material';
import React, { Suspense } from 'react';
import { createRoot } from 'react-dom/client';
import { createHashRouter, createRoutesFromElements, Route, RouterProvider } from 'react-router-dom';
import { ErrorPage, Layout } from './bootstrap';
import { UserProvider } from './context/session';
import { ToastProvider } from './context/toast';
import { Progress } from './library/loading';
import "./style/main.css";
import { theme } from './style/theme';
const LoginPage = React.lazy(() => import('./page/login'));
const MainPage = React.lazy(() => import('./page/main'));
const FindPasswordPage = React.lazy(() => import('./page/password'));
const RegisterPage = React.lazy(() => import('./page/register'));
const SystemAccessPage = React.lazy(() => import('./page/system/access'));
const SystemAccessResPage = React.lazy(() => import('./page/system/access/res'));
const SystemAccessRolePage = React.lazy(() => import('./page/system/access/role'));
const SystemAccessTestPage = React.lazy(() => import('./page/system/access/test'));
const SystemMainPage = React.lazy(() => import('./page/system/main'));
const SystemSettingPage = React.lazy(() => import('./page/system/setting'));
const SystemAppPage = React.lazy(() => import('./page/system/app'));
const SystemSmsSettingPage = React.lazy(() => import('./page/system/sms_setting'));
const SystemSmsSettingAlismsPage = React.lazy(() => import('./page/system/sms_setting/alisms_config'));
const SystemAppSmsAliSmsMapPage = React.lazy(() => import('./page/system/sms_setting/alisms_map'));
const SystemAppSmsLimitPage = React.lazy(() => import('./page/system/sms_setting/sms_limit'));
const SystemAppSmsMessagePage = React.lazy(() => import('./page/system/sms_setting/sms_message'));
const SystemUserPage = React.lazy(() => import('./page/system/user'));
const UserIndexPage = React.lazy(() => import('./page/user'));
const UserAccessPage = React.lazy(() => import('./page/user/access'));
const UserAppPage = React.lazy(() => import('./page/user/app'));
const UserAppSmsPage = React.lazy(() => import('./page/user/sms'));
const UserAppSmsAliSmsMapPage = React.lazy(() => import('./page/user/sms/alisms_map'));
const UserAppSmsLimitPage = React.lazy(() => import('./page/user/sms/sms_limit'));
const UserAppSmsMessagePage = React.lazy(() => import('./page/user/sms/sms_message'));
const UserInfoPage = React.lazy(() => import('./page/user/info'));
const UserEmailPage = React.lazy(() => import('./page/user/info/email'));
const UserInfoIndexPage = React.lazy(() => import('./page/user/info/index'));
const UserMobilePage = React.lazy(() => import('./page/user/info/mobile'));
const UserInfoNamePage = React.lazy(() => import('./page/user/info/name'));
const UserOauthPage = React.lazy(() => import('./page/user/info/oauth'));
const UserInfoPasswordPage = React.lazy(() => import('./page/user/info/password'));
const UserLoginHistroyPage = React.lazy(() => import('./page/user/login_history'));
const UserMainPage = React.lazy(() => import('./page/user/main'));
function PageProgress() {
  return <Progress />
}

function App() {
  const router = createHashRouter(
    createRoutesFromElements(

      <Route path="/*" element={<Layout />} errorElement={<ErrorPage />}>
        <Route path="system/*" element={<Suspense fallback={<PageProgress />}><SystemMainPage /></Suspense>} >
          <Route path="setting" element={<Suspense fallback={<PageProgress />}><SystemSettingPage /></Suspense>} />
          <Route path="app" element={<Suspense fallback={<PageProgress />}><SystemAppPage /></Suspense>} />
          <Route path="sms_setting" element={<Suspense fallback={<PageProgress />}><SystemSmsSettingPage /></Suspense>} >
            <Route path="alisms" element={<Suspense fallback={<PageProgress />}><SystemSmsSettingAlismsPage /></Suspense>} />
            <Route path="alisms_map" element={<Suspense fallback={<PageProgress />}><SystemAppSmsAliSmsMapPage /></Suspense>} />
            <Route path="limit" element={<Suspense fallback={<PageProgress />}><SystemAppSmsLimitPage /></Suspense>} />
            <Route path="message" element={<Suspense fallback={<PageProgress />}><SystemAppSmsMessagePage /></Suspense>} />
          </Route>
          <Route path="user" element={<Suspense fallback={<PageProgress />}><SystemUserPage /></Suspense>} />
          <Route path="access" element={<Suspense fallback={<PageProgress />}><SystemAccessPage /></Suspense>} >
            <Route path="role" element={<Suspense fallback={<PageProgress />}><SystemAccessRolePage /></Suspense>} />
            <Route path="res" element={<Suspense fallback={<PageProgress />}><SystemAccessResPage /></Suspense>} />
            <Route path="test" element={<Suspense fallback={<PageProgress />}><SystemAccessTestPage /></Suspense>} />
          </Route>

        </Route>
        <Route path="user/*" element={<Suspense fallback={<PageProgress />}><UserMainPage /></Suspense>}  >
          <Route path="info" element={<Suspense fallback={<PageProgress />}><UserInfoPage /></Suspense>} >
            <Route path="index" element={<Suspense fallback={<PageProgress />}><UserInfoIndexPage /></Suspense>} />
            <Route path="password" element={<Suspense fallback={<PageProgress />}><UserInfoPasswordPage /></Suspense>} />
            <Route path="name" element={<Suspense fallback={<PageProgress />}><UserInfoNamePage /></Suspense>} />
            <Route path="email" element={<Suspense fallback={<PageProgress />}><UserEmailPage /></Suspense>} />
            <Route path="mobile" element={<Suspense fallback={<PageProgress />}><UserMobilePage /></Suspense>} />
            <Route path="oauth" element={<Suspense fallback={<PageProgress />}><UserOauthPage /></Suspense>} />
          </Route>
          <Route path="access" element={<Suspense fallback={<PageProgress />}><UserAccessPage /></Suspense>} />
          <Route path="app" element={<Suspense fallback={<PageProgress />}><UserAppPage /></Suspense>} />
          <Route path="sms" element={<Suspense fallback={<PageProgress />}><UserAppSmsPage /></Suspense>} >
            <Route path="limit" element={<Suspense fallback={<PageProgress />}><UserAppSmsLimitPage /></Suspense>} />
            <Route path="alisms_map" element={<Suspense fallback={<PageProgress />}><UserAppSmsAliSmsMapPage /></Suspense>} />
            <Route path="message" element={<Suspense fallback={<PageProgress />}><UserAppSmsMessagePage /></Suspense>} />
          </Route>
          <Route path="login_history" element={<Suspense fallback={<PageProgress />}><UserLoginHistroyPage /></Suspense>} />
          <Route path="*" element={<Suspense fallback={<PageProgress />}><UserIndexPage /></Suspense>} />
        </Route>
        <Route path="register/:type" element={<Suspense fallback={<PageProgress />}><RegisterPage /></Suspense>} />
        <Route path="find_password/:type" element={<Suspense fallback={<PageProgress />}><FindPasswordPage /></Suspense>} />
        <Route path="login/:type" element={<Suspense fallback={<PageProgress />}><LoginPage /></Suspense>} />
        <Route path="" element={<Suspense fallback={<PageProgress />}><MainPage /></Suspense>} />

      </Route >

    ));
  return <>
    <ThemeProvider theme={theme} >
      <CssBaseline />
      <ToastProvider>
        <UserProvider>

          <RouterProvider router={router} />

        </UserProvider>
      </ToastProvider>
    </ThemeProvider>
  </>;
}


const container = document.getElementById('root');
const root = createRoot(container);
root.render(<App />);

