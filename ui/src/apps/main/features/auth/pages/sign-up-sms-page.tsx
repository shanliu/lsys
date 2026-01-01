import { useAuthRedirect } from '@apps/main/hooks/use-auth-redirect'
import {
  RegisterSmsParamSchema,
  RegisterSmsParamType,
  registerSms,
  registerSmsCode,
} from '@shared/apis/auth/register'
import { CaptchaData, CaptchaInput } from '@shared/components/custom/input/captcha-input'
import { PasswordInput } from '@shared/components/custom/input/password-input'
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
import { Loader2, Mail } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'

export default function SignUpSmsPage() {
  const toast = useToast();
  const handleAuthRedirect = useAuthRedirect();

  const [sendCodeCaptcha, setSendCodeCaptcha] = useState<CaptchaData>({
    code: '',
    key: '',
    validation: null,
  });
  const [countdown, setCountdown] = useState(0);

  const form = useForm<RegisterSmsParamType>({
    resolver: zodResolver(RegisterSmsParamSchema),
    defaultValues: {
      nikename: '',
      mobile: '',
      area_code: '86',
      code: '',
      password: '',
    },
  });

  // 倒计时逻辑
  useEffect(() => {
    if (countdown > 0) {
      const timer = setTimeout(() => setCountdown(countdown - 1), 1000);
      return () => clearTimeout(timer);
    }
  }, [countdown]);

  // 发送验证码
  const sendCodeMutation = useMutation({
    mutationFn: async () => {
      const mobile = form.getValues('mobile');
      const area_code = form.getValues('area_code');
      if (!mobile) {
        throw new Error('请输入手机号');
      }
      if (sendCodeCaptcha.validation !== 'success') {
        throw new Error('请先完成图形验证码验证');
      }
      if (!sendCodeCaptcha.code || !sendCodeCaptcha.key) {
        throw new Error('验证码数据异常，请刷新重试');
      }
      const res = await registerSmsCode({
        mobile,
        area_code,
        captcha: { code: sendCodeCaptcha.code, key: sendCodeCaptcha.key },
      });
      if (res.status) {
        setCountdown(res.response?.ttl || 60);
        toast.success('验证码已发送');
      }
      return res;
    },
    onError: (error: any) => {
      toast.error(formatServerError(error));
    }
  });

  // 注册
  const registerMutation = useMutation({
    mutationFn: async (data: RegisterSmsParamType) => {
      const res = await registerSms(data);
      if (res.status) {
        toast.success("注册成功");
        handleAuthRedirect();
      }
      return res;
    },
    onError: (error: any) => {
      toast.error(formatServerError(error));
    }
  });

  const isSendCodeDisabled = sendCodeMutation.isPending || countdown > 0 || !sendCodeCaptcha.code || sendCodeCaptcha.validation !== 'success';

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit((data) => registerMutation.mutate(data))} className="grid gap-3">
        <FormField
          control={form.control}
          name='nikename'
          render={({ field }) => (
            <FormItem>
              <FormLabel>昵称</FormLabel>
              <FormControl>
                <Input placeholder='请输入昵称' {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name='mobile'
          render={({ field }) => (
            <FormItem>
              <FormLabel>手机号</FormLabel>
              <FormControl>
                <div className="flex gap-2">
                  <Input 
                    className="w-20 shrink-0" 
                    placeholder="+86" 
                    value={`+${form.getValues('area_code')}`}
                    onChange={(e) => {
                      const val = e.target.value.replace(/[^0-9]/g, '');
                      form.setValue('area_code', val || '86');
                    }}
                  />
                  <Input placeholder='请输入手机号' {...field} />
                </div>
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <div className="space-y-2">
          <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">图形验证码</label>
          <CaptchaInput
            captchaType="register-sms"
            value={sendCodeCaptcha}
            onChange={setSendCodeCaptcha}
            placeholder="请输入图形验证码"
          />
        </div>
        <FormField
          control={form.control}
          name='code'
          render={({ field }) => (
            <FormItem>
              <FormLabel>短信验证码</FormLabel>
              <FormControl>
                <div className="flex gap-2">
                  <Input className="flex-1" placeholder='请输入短信验证码' {...field} />
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
        <FormField
          control={form.control}
          name='password'
          render={({ field }) => (
            <FormItem>
              <FormLabel>密码</FormLabel>
              <FormControl>
                <PasswordInput placeholder='请输入密码（至少6位）' {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <Button type="submit" className={cn('mt-2')} disabled={registerMutation.isPending}>
          {registerMutation.isPending ? (
            <>
              <Loader2 className="h-4 w-4 animate-spin" />
              <span className="ml-2">注册中...</span>
            </>
          ) : '注册'}
        </Button>

        <div className='relative my-2'>
          <div className='absolute inset-0 flex items-center'>
            <span className='w-full border-t' />
          </div>
          <div className='relative flex justify-center text-xs uppercase'>
            <span className='bg-card text-muted-foreground px-2'>
              其他注册方式
            </span>
          </div>
        </div>

        <Link to='/sign-up/mail'>
          <Button variant='outline' type='button' className="w-full" disabled={registerMutation.isPending}>
            <Mail className='h-4 w-4' /> 邮箱注册
          </Button>
        </Link>
      </form>
    </Form>
  )
}
