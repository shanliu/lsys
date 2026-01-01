import { getHomeUrl } from '@apps/main/components/local/app-logo'
import { useDictData } from '@apps/main/hooks/use-dict-data'
import { useAuthRedirect } from '@apps/main/hooks/use-auth-redirect'
import { Route } from '@apps/main/routes/_auth/sign-in'
import { LoginNamePasswordParamSchema, LoginNamePasswordParamType, loginNamePassword } from '@shared/apis/auth/login'
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
import { useToast } from '@shared/contexts/toast-context'
import { cn, formatServerError } from '@shared/lib/utils'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation } from '@tanstack/react-query'
import { Link } from '@tanstack/react-router'
import { AlertCircle, CardSim, Home, Loader2, Mail, RefreshCw } from 'lucide-react'
import { useState } from 'react'
import { useForm } from 'react-hook-form'

export default function SignInAccountPage() {
  const toast = useToast();
  const search = Route.useSearch();
  const handleAuthRedirect = useAuthRedirect();

  // 使用字典加载登录类型
  const { dictData, isLoading: dictLoading, isError: dictError, refetch: refetchDict } = useDictData(['auth_login'] as const);

  const [captchaData, setCaptchaData] = useState<CaptchaData>({
    code: '',
    key: '',
    validation: null,
  });

  const form = useForm<LoginNamePasswordParamType>({
    resolver: zodResolver(LoginNamePasswordParamSchema),
    defaultValues: {
      name: '',
      password: '',
    },
  });

  const mutation = useMutation({
    mutationFn: async (data: LoginNamePasswordParamType) => {
      if (!captchaData.code || !captchaData.key) {
        throw new Error('请输入验证码');
      }
      if (captchaData.validation !== 'success') {
        throw new Error('验证码验证失败，请重新输入');
      }

      const res = await loginNamePassword({
        ...data,
        captcha: {
          code: captchaData.code,
          key: captchaData.key,
        },
      });
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

  const isSubmitDisabled = mutation.isPending || !captchaData.code || captchaData.validation !== 'success';

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

  // 检查是否支持账号密码登录
  const loginTypes = dictData?.login_type || [];
  const supportsNameLogin = loginTypes.some(item => item.key === 'name');
  const supportsEmailLogin = loginTypes.some(item => item.key === 'email' || item.key === 'email-code');
  const supportsSmsLogin = loginTypes.some(item => item.key === 'mobile' || item.key === 'mobile-code');

  if (!supportsNameLogin) {
    return (
      <div className="flex flex-col items-center justify-center gap-4 py-8">
        <AlertCircle className="h-12 w-12 text-muted-foreground" />
        <p className="text-sm text-muted-foreground">当前不支持账号密码登录</p>
        <div className="flex gap-2">
          {supportsEmailLogin && (
            <Link to="/sign-in/mail" search={search}>
              <Button variant="outline" size="sm">
                <Mail className="h-4 w-4 mr-2" />
                邮箱登录
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

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit((data => {
        mutation.mutate(data)
      }))} className={cn("grid gap-3")}>
        <FormField
          control={form.control}
          name='name'
          render={({ field }) => (
            <FormItem>
              <FormLabel>账号</FormLabel>
              <FormControl>
                <Input placeholder='请输入用户名' {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name='password'
          render={({ field }) => (
            <FormItem>
              <div className="flex items-center justify-between">
                <FormLabel>密码</FormLabel>
                {search.from !== 'user-switch' && (
                  <Link
                    to="/find-password/sms"
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

        <Button type="submit" className={cn('mt-2')} disabled={isSubmitDisabled}>
          {mutation.isPending ? (
            <>
              <Loader2 className="h-4 w-4 animate-spin" />
              <span className="ml-2">登录中...</span>
            </>
          ) : (
            '登录'
          )}
        </Button>

        <div className='relative my-2'>
          <div className='absolute inset-0 flex items-center'>
            <span className='w-full border-t' />
          </div>
          <div className='relative flex justify-center text-xs uppercase'>
            <span className='bg-card text-muted-foreground px-2 flex items-center gap-1'>
              {(supportsEmailLogin || supportsSmsLogin) && <>其他登录方式 <span className='text-[10px]'>or</span> </>}
              <a href={getHomeUrl()} className='hover:text-primary'>
                返回首页
              </a>
            </span>
          </div>
        </div>

        {(supportsEmailLogin || supportsSmsLogin) && (
            <div className={cn('grid gap-2', supportsEmailLogin && supportsSmsLogin ? 'grid-cols-2' : 'grid-cols-1')}>
              {supportsEmailLogin && (
                <Link to='/sign-in/mail' search={search}>
                  <Button variant='outline' type='button' className="w-full" disabled={mutation.isPending}>
                    <Mail className='h-4 w-4' /> 邮箱登录
                  </Button>
                </Link>
              )}
              {supportsSmsLogin && (
                <Link to='/sign-in/sms' search={search}>
                  <Button variant='outline' type='button' className="w-full" disabled={mutation.isPending}>
                    <CardSim className='h-4 w-4' /> 短信登录
                  </Button>
                </Link>
              )}
            </div>
        )}
      </form>
    </Form>
  )
}
