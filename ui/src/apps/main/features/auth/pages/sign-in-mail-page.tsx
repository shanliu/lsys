import { useDictData } from '@apps/main/hooks/use-dict-data'
import { useAuthRedirect } from '@apps/main/hooks/use-auth-redirect'
import { Route } from '@apps/main/routes/_auth/sign-in/mail'
import {
  LoginEmailCodeParamSchema,
  LoginEmailCodeParamType,
  LoginEmailPasswordParamSchema,
  LoginEmailPasswordParamType,
  loginEmailCode,
  loginEmailCodeSend,
  loginEmailPassword,
} from '@shared/apis/auth/login'
import { CaptchaData, CaptchaInput } from '@shared/components/custom/input/captcha-input'
import { PasswordInput } from '@shared/components/custom/input/password-input'
import { CenteredLoading } from '@shared/components/custom/page-placeholder/centered-loading'
import { Button } from '@shared/components/ui/button'
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@shared/components/ui/form'
import { Input } from '@shared/components/ui/input'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@shared/components/ui/tabs'
import { useToast } from '@shared/contexts/toast-context'
import { cn, formatServerError, getHomeUrl } from '@shared/lib/utils'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation } from '@tanstack/react-query'
import { Link, useNavigate } from '@tanstack/react-router'
import { AlertCircle, CardSim, KeySquare, Loader2, RefreshCw } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'

export default function SignInMailPage() {
  const toast = useToast();
  const search = Route.useSearch();
  const navigate = useNavigate();
  const handleAuthRedirect = useAuthRedirect();

  // 使用字典加载登录类型
  const { dictData, isLoading: dictLoading, isError: dictError, refetch: refetchDict } = useDictData(['auth_login'] as const);

  const [activeTab, setActiveTab] = useState<'password' | 'code'>('password');
  const [captchaData, setCaptchaData] = useState<CaptchaData>({
    code: '',
    key: '',
    validation: null,
  });
  const [sendCodeCaptcha, setSendCodeCaptcha] = useState<CaptchaData>({
    code: '',
    key: '',
    validation: null,
  });
  const [countdown, setCountdown] = useState(0);
  const [codeSent, setCodeSent] = useState(false);

  // 密码登录表单
  const passwordForm = useForm<LoginEmailPasswordParamType>({
    resolver: zodResolver(LoginEmailPasswordParamSchema),
    defaultValues: {
      email: '',
      password: '',
    },
  });

  // 验证码登录表单
  const codeForm = useForm<LoginEmailCodeParamType>({
    resolver: zodResolver(LoginEmailCodeParamSchema),
    defaultValues: {
      email: '',
      code: '',
    },
  });

  // 倒计时逻辑
  useEffect(() => {
    if (countdown > 0) {
      const timer = setTimeout(() => setCountdown(countdown - 1), 1000);
      return () => clearTimeout(timer);
    }
  }, [countdown]);

  // 密码登录
  const passwordMutation = useMutation({
    mutationFn: async (data: LoginEmailPasswordParamType) => {
      if (captchaData.validation !== 'success') {
        throw new Error('请完成图形验证码验证');
      }
      if (!captchaData.code || !captchaData.key) {
        throw new Error('验证码数据异常，请刷新重试');
      }
      const res = await loginEmailPassword({
        ...data,
        captcha: { code: captchaData.code, key: captchaData.key },
      });
      // 检查是否需要 MFA 验证
      if (res.mfa_token) {
        navigate({
          to: '/sign-in/mfa',
          search: { mfa_token: res.mfa_token, redirect_uri: search.redirect_uri }
        });
        return res;
      }
      if (res.status) {
        toast.success("登录成功");
        handleAuthRedirect();
      }
      return res;
    },
    onError: (error: any) => {
      toast.error(formatServerError(error));
    }
  });

  // 发送验证码
  const sendCodeMutation = useMutation({
    mutationFn: async () => {
      const email = codeForm.getValues('email');
      if (!email) {
        throw new Error('请输入邮箱地址');
      }
      if (sendCodeCaptcha.validation !== 'success') {
        throw new Error('请先完成图形验证码验证');
      }
      if (!sendCodeCaptcha.code || !sendCodeCaptcha.key) {
        throw new Error('验证码数据异常，请刷新重试');
      }
      const res = await loginEmailCodeSend({
        email,
        captcha: { code: sendCodeCaptcha.code, key: sendCodeCaptcha.key },
      });
      if (res.status && res.response) {
        setCountdown(res.response.ttl || 60);
        setCodeSent(true);
        toast.success('验证码已发送');
      }
      return res;
    },
    onError: (error: any) => {
      toast.error(formatServerError(error));
    }
  });

  // 验证码登录
  const codeMutation = useMutation({
    mutationFn: async (data: LoginEmailCodeParamType) => {
      if (captchaData.validation !== 'success') {
        throw new Error('请完成图形验证码验证');
      }
      if (!captchaData.code || !captchaData.key) {
        throw new Error('验证码数据异常，请刷新重试');
      }
      const res = await loginEmailCode({
        ...data,
        captcha: { code: captchaData.code, key: captchaData.key },
      });
      // 检查是否需要 MFA 验证
      if (res.mfa_token) {
        navigate({
          to: '/sign-in/mfa',
          search: { mfa_token: res.mfa_token, redirect_uri: search.redirect_uri }
        });
        return res;
      }
      if (res.status) {
        toast.success("登录成功");
        handleAuthRedirect();
      }
      return res;
    },
    onError: (error: any) => {
      toast.error(formatServerError(error));
    }
  });

  // 字典加载中状态
  if (dictLoading) {
    return <CenteredLoading variant="content" iconSize="sm" className="mx-30 my-6 sm:mx-0 sm:my-12" />;
  }

  // 字典加载失败状态
  if (dictError) {
    return (
      <div className="flex flex-col items-center justify-center gap-4 py-8">
        <AlertCircle className="h-12 w-12 text-destructive" />
        <p className="text-sm text-muted-foreground">加载配置失败</p>
        <Button variant="outline" size="sm" onClick={refetchDict}>
          <RefreshCw className="h-4 w-4 mr-2" />
          重试
        </Button>
      </div>
    );
  }

  const loginTypes = dictData?.login_type || [];
  const supportsEmailPassword = loginTypes.some(item => item.key === 'email');
  const supportsEmailCode = loginTypes.some(item => item.key === 'email-code');
  const supportsNameLogin = loginTypes.some(item => item.key === 'name');
  const supportsSmsLogin = loginTypes.some(item => item.key === 'mobile' || item.key === 'mobile-code');

  if (!supportsEmailPassword && !supportsEmailCode) {
    return (
      <div className="flex flex-col items-center justify-center gap-4 py-8">
        <AlertCircle className="h-12 w-12 text-muted-foreground" />
        <p className="text-sm text-muted-foreground">当前不支持邮箱登录</p>
        <div className="flex gap-2">
          {supportsNameLogin && (
            <Link to="/sign-in" search={search}>
              <Button variant="outline" size="sm">
                <KeySquare className="h-4 w-4 mr-2" />
                账号登录
              </Button>
            </Link>
          )}
          {supportsSmsLogin && (
            <Link to="/sign-in/sms" search={search}>
              <Button variant="outline" size="sm">
                <CardSim className="h-4 w-4 mr-2" />
                短信登录
              </Button>
            </Link>
          )}
        </div>
      </div>
    );
  }

  const showTabs = supportsEmailPassword && supportsEmailCode;

  const isPasswordSubmitDisabled = passwordMutation.isPending || !captchaData.code || captchaData.validation !== 'success';
  const isCodeSubmitDisabled = codeMutation.isPending || !captchaData.code || captchaData.validation !== 'success';
  const isSendCodeDisabled = sendCodeMutation.isPending || countdown > 0 || !sendCodeCaptcha.code || sendCodeCaptcha.validation !== 'success';

  const renderPasswordForm = () => (
    <Form {...passwordForm}>
      <form onSubmit={passwordForm.handleSubmit((data) => passwordMutation.mutate(data))} className="grid gap-3">
        <FormField
          control={passwordForm.control}
          name='email'
          render={({ field }) => (
            <FormItem>
              <FormLabel>邮箱</FormLabel>
              <FormControl>
                <Input placeholder='请输入邮箱地址' type="email" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={passwordForm.control}
          name='password'
          render={({ field }) => (
            <FormItem>
              <div className="flex items-center justify-between">
                <FormLabel>密码</FormLabel>
                {search.from !== 'user-switch' && (
                  <Link
                    to="/find-password/mail"
                    className="text-xs text-muted-foreground hover:text-primary"
                  >
                    忘记密码？
                  </Link>
                )}
              </div>
              <FormControl>
                <PasswordInput placeholder='请输入密码' {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <div className="space-y-2">
          <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">验证码</label>
          <CaptchaInput
            captchaType="login"
            value={captchaData}
            onChange={setCaptchaData}
            placeholder="请输入验证码"
          />
        </div>
        <Button type="submit" className={cn('mt-2')} disabled={isPasswordSubmitDisabled}>
          {passwordMutation.isPending ? (
            <>
              <Loader2 className="h-4 w-4 animate-spin" />
              <span className="ml-2">登录中...</span>
            </>
          ) : '登录'}
        </Button>
      </form>
    </Form>
  );

  const renderCodeForm = () => (
    <Form {...codeForm}>
      <form onSubmit={codeForm.handleSubmit((data) => codeMutation.mutate(data))} className="grid gap-3">
        <FormField
          control={codeForm.control}
          name='email'
          render={({ field }) => (
            <FormItem>
              <FormLabel>邮箱</FormLabel>
              <FormControl>
                <Input placeholder='请输入邮箱地址' type="email" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <div className="space-y-2">
          <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">图形验证码</label>
          <CaptchaInput
            captchaType="login-email"
            value={sendCodeCaptcha}
            onChange={setSendCodeCaptcha}
            placeholder="请输入图形验证码"
          />
        </div>
        <FormField
          control={codeForm.control}
          name='code'
          render={({ field }) => (
            <FormItem>
              <FormLabel>邮箱验证码</FormLabel>
              <FormControl>
                <div className="flex gap-2">
                  <Input className="flex-1" placeholder='请输入邮箱验证码' {...field} />
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() => sendCodeMutation.mutate()}
                    disabled={isSendCodeDisabled}
                    className="shrink-0"
                  >
                    {sendCodeMutation.isPending ? (
                      <Loader2 className="h-4 w-4 animate-spin" />
                    ) : countdown > 0 ? `${countdown}s` : '发送验证码'}
                  </Button>
                </div>
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        {codeSent && (
          <div className="space-y-2">
            <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">登录验证码</label>
            <CaptchaInput
              captchaType="login"
              value={captchaData}
              onChange={setCaptchaData}
              placeholder="请输入登录验证码"
            />
          </div>
        )}
        <Button type="submit" className={cn('mt-2')} disabled={isCodeSubmitDisabled || !codeSent}>
          {codeMutation.isPending ? (
            <>
              <Loader2 className="h-4 w-4 animate-spin" />
              <span className="ml-2">登录中...</span>
            </>
          ) : '登录'}
        </Button>
      </form>
    </Form>
  );

  return (
    <div className="grid gap-3">
      {showTabs ? (
        <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as 'password' | 'code')}>
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="password">密码登录</TabsTrigger>
            <TabsTrigger value="code">验证码登录</TabsTrigger>
          </TabsList>
          <TabsContent value="password" className="mt-4">
            {renderPasswordForm()}
          </TabsContent>
          <TabsContent value="code" className="mt-4">
            {renderCodeForm()}
          </TabsContent>
        </Tabs>
      ) : supportsEmailPassword ? (
        renderPasswordForm()
      ) : (
        renderCodeForm()
      )}

      <div className='relative my-2'>
        <div className='absolute inset-0 flex items-center'>
          <span className='w-full border-t' />
        </div>
        <div className='relative flex justify-center text-xs uppercase'>
          <span className='bg-card text-muted-foreground px-2 flex items-center gap-1'>
            {(supportsNameLogin || supportsSmsLogin) && <>其他登录方式 <span className='text-[10px]'>or</span> </>}
            <a href={getHomeUrl()} className='hover:text-primary'>
              返回首页
            </a>
          </span>
        </div>
      </div>

      {(supportsNameLogin || supportsSmsLogin) && (
        <div className={cn('grid gap-2', supportsNameLogin && supportsSmsLogin ? 'grid-cols-2' : 'grid-cols-1')}>
          {supportsNameLogin && (
            <Link to='/sign-in' search={search}>
              <Button variant='outline' type='button' className="w-full">
                <KeySquare className='h-4 w-4' /> 账号登录
              </Button>
            </Link>
          )}
          {supportsSmsLogin && (
            <Link to='/sign-in/sms' search={search}>
              <Button variant='outline' type='button' className="w-full">
                <CardSim className='h-4 w-4' /> 短信登录
              </Button>
            </Link>
          )}
        </div>
      )}
    </div>
  )
}
