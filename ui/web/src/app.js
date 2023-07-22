import { CssBaseline, ThemeProvider } from '@mui/material';
import React, { Suspense } from 'react';
import { createRoot } from 'react-dom/client';
import { createHashRouter, createRoutesFromElements, Route, RouterProvider } from 'react-router-dom';
import { UserProvider } from './context/session';
import { ToastProvider } from './context/toast';
import { Progress } from './library/loading';
import "./style/app.css";
import { theme } from './style/theme';
import { ConfigProvider } from './context/config';
import { PageLayout } from './page/library/layout';
import { ErrorPage } from './page/error';
import DocPage from './page/doc';
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

const SystemSettingOauthPage = React.lazy(() => import('./page/system/setting/oauth'));
const SystemSettingOauthWechatPage = React.lazy(() => import('./page/system/setting/oauth_wechat'));
const SystemSettingSitePage = React.lazy(() => import('./page/system/setting/site'));
const SystemSettingSiteSettingPage = React.lazy(() => import('./page/system/setting/site_setting'));


const SystemLogsPage = React.lazy(() => import('./page/system/logs'));
const SystemAppPage = React.lazy(() => import('./page/system/app'));
const SystemDocsPage = React.lazy(() => import('./page/system/docs'));


const SystemSmsSettingPage = React.lazy(() => import('./page/system/sender/sms_setting'));
const SystemAppSmsLimitPage = React.lazy(() => import('./page/system/sender/sms_setting/sms_limit'));
const SystemAppSmsMessagePage = React.lazy(() => import('./page/system/sender/sms_setting/sms_message'));
const SystemAppSmsTplConfigPage = React.lazy(() => import('./page/system/sender/sms_setting/tpl_config'));
const SystemAppSmsMapConfigPage = React.lazy(() => import('./page/system/sender/sms_setting/map_config'));
const SystemAppSmsSettingAlismsPage = React.lazy(() => import('./page/library/sender/sms/alisms'));
const SystemAppSmsSettingHwsmsPage = React.lazy(() => import('./page/library/sender/sms/hwsms'));
const SystemAppSmsSettingTensmsPage = React.lazy(() => import('./page/library/sender/sms/tensms'));


const SystemMailSettingPage = React.lazy(() => import('./page/system/sender/mail_setting'));
const SystemAppMailLimitPage = React.lazy(() => import('./page/system/sender/mail_setting/mail_limit'));
const SystemAppMailMessagePage = React.lazy(() => import('./page/system/sender/mail_setting/mail_message'));
const SystemAppMailTplConfigPage = React.lazy(() => import('./page/system/sender/mail_setting/tpl_config'));
const SystemAppMailTplBodyPage = React.lazy(() => import('./page/system/sender/mail_setting/tpl_body'));
const SystemAppMailMapConfigPage = React.lazy(() => import('./page/system/sender/mail_setting/map_config'));
const SystemAppMailSettingSmtpPage = React.lazy(() => import('./page/library/sender/mail/smtp'));



const SystemUserPage = React.lazy(() => import('./page/system/user'));
const UserIndexPage = React.lazy(() => import('./page/user'));
const UserAccessPage = React.lazy(() => import('./page/user/access'));
const UserAppPage = React.lazy(() => import('./page/user/app'));
const UserAppSmsPage = React.lazy(() => import('./page/user/sms'));
const UserAppMailPage = React.lazy(() => import('./page/user/mail'));

const UserAppSmsSendPage = React.lazy(() => import('./page/user/sms/send'));
const UserAppSmsLimitPage = React.lazy(() => import('./page/user/sms/sms_limit'));
const UserAppSmsMessagePage = React.lazy(() => import('./page/user/sms/sms_message'));
const UserAppSmsTplConfigPage = React.lazy(() => import('./page/user/sms/tpl_config'));

const UserAppMailSendPage = React.lazy(() => import('./page/user/mail/send'));
const UserAppMailLimitPage = React.lazy(() => import('./page/user/mail/mail_limit'));
const UserAppMailMessagePage = React.lazy(() => import('./page/user/mail/mail_message'));
const UserAppMailTplBodyPage = React.lazy(() => import('./page/user/mail/tpl_body'));
const UserAppMailTplConfigPage = React.lazy(() => import('./page/user/mail/tpl_config'));


const UserInfoPage = React.lazy(() => import('./page/user/info'));
const UserEmailPage = React.lazy(() => import('./page/user/info/email'));
const UserInfoIndexPage = React.lazy(() => import('./page/user/info/index'));
const UserMobilePage = React.lazy(() => import('./page/user/info/mobile'));
const UserAddressPage = React.lazy(() => import('./page/user/info/address'));
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

      <Route path="/*" element={<PageLayout />} errorElement={<ErrorPage />}>
        <Route path="system/*" element={<Suspense fallback={<PageProgress />}><SystemMainPage /></Suspense>} >

          <Route path="setting" element={<Suspense fallback={<PageProgress />}><SystemSettingPage /></Suspense>} >
            <Route path="oauth" element={<Suspense fallback={<PageProgress />}><SystemSettingOauthPage /></Suspense>} >
              <Route path="wechat" element={<Suspense fallback={<PageProgress />}><SystemSettingOauthWechatPage /></Suspense>} />
            </Route>
            <Route path="site" element={<Suspense fallback={<PageProgress />}><SystemSettingSitePage /></Suspense>} >
              <Route path="setting" element={<Suspense fallback={<PageProgress />}><SystemSettingSiteSettingPage /></Suspense>} />
            </Route>
          </Route>
          <Route path="logs" element={<Suspense fallback={<PageProgress />}><SystemLogsPage /></Suspense>} />

          <Route path="app" element={<Suspense fallback={<PageProgress />}><SystemAppPage /></Suspense>} />
          <Route path="docs" element={<Suspense fallback={<PageProgress />}><SystemDocsPage /></Suspense>} />

          <Route path="sender_sms" element={<Suspense fallback={<PageProgress />}><SystemSmsSettingPage /></Suspense>} >
            <Route path="limit" element={<Suspense fallback={<PageProgress />}><SystemAppSmsLimitPage /></Suspense>} />
            <Route path="message" element={<Suspense fallback={<PageProgress />}><SystemAppSmsMessagePage /></Suspense>} />
            <Route path="tpl_config" element={<Suspense fallback={<PageProgress />}><SystemAppSmsTplConfigPage /></Suspense>} />
            <Route path="map_config" element={<Suspense fallback={<PageProgress />}><SystemAppSmsMapConfigPage /></Suspense>} >
              <Route path="alisms" element={<Suspense fallback={<PageProgress />}><SystemAppSmsSettingAlismsPage /></Suspense>} />
              <Route path="hwsms" element={<Suspense fallback={<PageProgress />}><SystemAppSmsSettingHwsmsPage /></Suspense>} />
              <Route path="tensms" element={<Suspense fallback={<PageProgress />}><SystemAppSmsSettingTensmsPage /></Suspense>} />
            </Route>
          </Route>
          <Route path="sender_mail" element={<Suspense fallback={<PageProgress />}><SystemMailSettingPage /></Suspense>} >

            <Route path="limit" element={<Suspense fallback={<PageProgress />}><SystemAppMailLimitPage /></Suspense>} />
            <Route path="message" element={<Suspense fallback={<PageProgress />}><SystemAppMailMessagePage /></Suspense>} />
            <Route path="tpl_body" element={<Suspense fallback={<PageProgress />}><SystemAppMailTplBodyPage /></Suspense>} />
            <Route path="tpl_config" element={<Suspense fallback={<PageProgress />}><SystemAppMailTplConfigPage /></Suspense>} />
            <Route path="map_config" element={<Suspense fallback={<PageProgress />}><SystemAppMailMapConfigPage /></Suspense>} >
              <Route path="smtp" element={<Suspense fallback={<PageProgress />}><SystemAppMailSettingSmtpPage /></Suspense>} />
            </Route>
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
            <Route path="address" element={<Suspense fallback={<PageProgress />}><UserAddressPage /></Suspense>} />
            <Route path="oauth" element={<Suspense fallback={<PageProgress />}><UserOauthPage /></Suspense>} />
          </Route>
          <Route path="access" element={<Suspense fallback={<PageProgress />}><UserAccessPage /></Suspense>} />
          <Route path="app" element={<Suspense fallback={<PageProgress />}><UserAppPage /></Suspense>} />
          <Route path="sms" element={<Suspense fallback={<PageProgress />}><UserAppSmsPage /></Suspense>} >
            <Route path="limit" element={<Suspense fallback={<PageProgress />}><UserAppSmsLimitPage /></Suspense>} />
            <Route path="message" element={<Suspense fallback={<PageProgress />}><UserAppSmsMessagePage /></Suspense>} />
            <Route path="send" element={<Suspense fallback={<PageProgress />}><UserAppSmsSendPage /></Suspense>} />
            <Route path="tpl_config" element={<Suspense fallback={<PageProgress />}><UserAppSmsTplConfigPage /></Suspense>} />
          </Route>
          <Route path="mail" element={<Suspense fallback={<PageProgress />}><UserAppMailPage /></Suspense>} >
            <Route path="limit" element={<Suspense fallback={<PageProgress />}><UserAppMailLimitPage /></Suspense>} />
            <Route path="message" element={<Suspense fallback={<PageProgress />}><UserAppMailMessagePage /></Suspense>} />
            <Route path="send" element={<Suspense fallback={<PageProgress />}><UserAppMailSendPage /></Suspense>} />
            <Route path="tpl_body" element={<Suspense fallback={<PageProgress />}><UserAppMailTplBodyPage /></Suspense>} />
            <Route path="tpl_config" element={<Suspense fallback={<PageProgress />}><UserAppMailTplConfigPage /></Suspense>} />

          </Route>
          <Route path="login_history" element={<Suspense fallback={<PageProgress />}><UserLoginHistroyPage /></Suspense>} />
          <Route path="*" element={<Suspense fallback={<PageProgress />}><UserIndexPage /></Suspense>} />
        </Route>
        <Route path="register/:type" element={<Suspense fallback={<PageProgress />}><RegisterPage /></Suspense>} />
        <Route path="find_password/:type" element={<Suspense fallback={<PageProgress />}><FindPasswordPage /></Suspense>} />
        <Route path="login/:type" element={<Suspense fallback={<PageProgress />}><LoginPage /></Suspense>} />
        <Route path="doc" element={<Suspense fallback={<PageProgress />}><DocPage /></Suspense>} />
        <Route path="" element={<Suspense fallback={<PageProgress />}><MainPage /></Suspense>} />
      </Route >
    ));
  return <>
    <ThemeProvider theme={theme} >
      <CssBaseline />
      <ConfigProvider>
        <ToastProvider>
          <UserProvider>
            <RouterProvider router={router} />
          </UserProvider>
        </ToastProvider>
      </ConfigProvider>
    </ThemeProvider>
  </>;
}


const container = document.getElementById('root');
const root = createRoot(container);
root.render(<App />);

