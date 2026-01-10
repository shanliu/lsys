import { zodResolver } from '@hookform/resolvers/zod';
import { CaptchaInput } from '@shared/components/custom/input/captcha-input';
import { PasswordInput } from '@shared/components/custom/input/password-input';
import { Button } from '@shared/components/ui/button';
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@shared/components/ui/form';
import { Input } from '@shared/components/ui/input';
import { cn, formatServerError } from '@shared/lib/utils';
import { CaptchaType } from '@shared/types/base-schema';
import { useMutation } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import { Loader2, Mail } from 'lucide-react';
import { useCallback, useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { toast } from 'sonner';
import { z } from 'zod';
import {
  passwordMobile,
  PasswordMobileCodeParamSchema,
  PasswordMobileParamSchema,
  passwordMobileCode,
} from '@shared/apis/auth/password';

// 表单 Schema
const FormSchema = z.object({
  area_code: z.string().min(1, "区号不能为空"),
  mobile: z.string().min(1, "手机号不能为空").regex(/^\d+$/, "手机号格式不正确"),
  code: z.string().min(1, "验证码不能为空"),
  new_password: z.string().min(6, "密码长度至少6位"),
  confirm_password: z.string().min(6, "确认密码长度至少6位"),
}).refine((data) => data.new_password === data.confirm_password, {
  message: "两次密码输入不一致",
  path: ["confirm_password"],
});

type FormType = z.infer<typeof FormSchema>;

export function FindPasswordSmsPage() {
  const [captchaData, setCaptchaData] = useState<CaptchaType>({ key: '', code: '' });
  const [countdown, setCountdown] = useState(0);
  const [isSuccess, setIsSuccess] = useState(false);

  const form = useForm<FormType>({
    resolver: zodResolver(FormSchema),
    defaultValues: {
      area_code: '+86',
      mobile: '',
      code: '',
      new_password: '',
      confirm_password: '',
    },
  });

  // 发送验证码倒计时
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
      const area_code = form.getValues('area_code').replace('+', '');
      if (!mobile) {
        throw new Error('请输入手机号');
      }
      if (!captchaData.code || !captchaData.key) {
        throw new Error('请输入图形验证码');
      }
      const param = PasswordMobileCodeParamSchema.parse({
        mobile,
        area_code,
        captcha: captchaData,
      });
      return passwordMobileCode(param);
    },
    onSuccess: (res) => {
      if (res.status) {
        toast.success('验证码已发送');
        setCountdown(60);
      } else {
        toast.error(formatServerError(res, '发送失败'));
      }
    },
    onError: (error: Error) => {
      toast.error(formatServerError(error, '发送失败'));
    },
  });

  // 重置密码
  const resetMutation = useMutation({
    mutationFn: async (data: FormType) => {
      const param = PasswordMobileParamSchema.parse({
        mobile: data.mobile,
        area_code: data.area_code.replace('+', ''),
        code: data.code,
        new_password: data.new_password,
      });
      return passwordMobile(param);
    },
    onSuccess: (res) => {
      if (res.status) {
        toast.success('密码重置成功');
        setIsSuccess(true);
      } else {
        toast.error(formatServerError(res, '重置失败'));
      }
    },
    onError: (error: Error) => {
      toast.error(formatServerError(error, '重置失败'));
    },
  });

  const handleSubmit = useCallback((data: FormType) => {
    resetMutation.mutate(data);
  }, [resetMutation]);

  const isSendCodeDisabled = !form.watch('mobile') || !captchaData.code || countdown > 0 || sendCodeMutation.isPending;
  const isSubmitDisabled = !form.formState.isValid || resetMutation.isPending;

  if (isSuccess) {
    return (
      <div className="grid gap-4 text-center py-8">
        <div className="text-green-500 text-5xl mb-2">✓</div>
        <h3 className="text-lg font-medium">密码重置成功</h3>
        <p className="text-muted-foreground text-sm">您的密码已成功重置，请使用新密码登录</p>
        <Link to="/sign-in/sms">
          <Button className="mt-4">返回登录</Button>
        </Link>
      </div>
    );
  }

  return (
    <div className="grid gap-3">
      <Form {...form}>
        <form onSubmit={form.handleSubmit(handleSubmit)} className="grid gap-3">
          <FormField
            control={form.control}
            name="mobile"
            render={({ field }) => (
              <FormItem>
                <FormLabel>手机号</FormLabel>
                <FormControl>
                  <div className="flex gap-2">
                    <Input
                      {...form.register('area_code')}
                      className="w-20"
                      placeholder="+86"
                    />
                    <Input placeholder="请输入手机号" {...field} />
                  </div>
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
          <div className="space-y-2">
            <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">图形验证码</label>
            <CaptchaInput
              captchaType="reset-password-send-sms"
              value={captchaData}
              onChange={setCaptchaData}
              placeholder="请输入图形验证码"
              className="flex-1"
            />
          </div>
          <FormField
            control={form.control}
            name="code"
            render={({ field }) => (
              <FormItem>
                <FormLabel>短信验证码</FormLabel>
                <FormControl>
                  <div className="flex gap-2">
                    <Input className="flex-1" placeholder="请输入短信验证码" {...field} />
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
            name="new_password"
            render={({ field }) => (
              <FormItem>
                <FormLabel>新密码</FormLabel>
                <FormControl>
                  <PasswordInput placeholder="请输入新密码（至少6位）" {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="confirm_password"
            render={({ field }) => (
              <FormItem>
                <FormLabel>确认密码</FormLabel>
                <FormControl>
                  <PasswordInput placeholder="请再次输入新密码" {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
          <Button type="submit" className={cn('mt-2')} disabled={isSubmitDisabled}>
            {resetMutation.isPending ? (
              <>
                <Loader2 className="h-4 w-4 animate-spin" />
                <span className="ml-2">重置中...</span>
              </>
            ) : '重置密码'}
          </Button>
        </form>
      </Form>
      <div className="relative my-2">
        <div className="absolute inset-0 flex items-center">
          <span className="w-full border-t" />
        </div>
        <div className="relative flex justify-center text-xs uppercase">
          <span className="bg-card px-2 text-muted-foreground">其他方式</span>
        </div>
      </div>
      <Link to="/find-password/mail">
        <Button variant="outline" className="w-full" type="button">
          <Mail className="mr-2 h-4 w-4" />
          邮箱找回
        </Button>
      </Link>
    </div>
  );
}
