import { useAuthRedirect } from '@apps/main/hooks/use-auth-redirect'
import { Route } from '@apps/main/routes/_auth/sign-in/mfa'
import { mfaVerify, MfaVerifyParamSchema, MfaVerifyParamType } from '@shared/apis/auth/login'
import { Button } from '@shared/components/ui/button'
import {
    Form,
    FormControl,
    FormField,
    FormItem,
    FormLabel,
    FormMessage,
} from '@shared/components/ui/form'
import {
    InputOTP,
    InputOTPGroup,
    InputOTPSlot,
} from '@shared/components/custom/input/input-otp'
import { useToast } from '@shared/contexts/toast-context'
import { cn, formatServerError } from '@shared/lib/utils'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation } from '@tanstack/react-query'
import { Link, useNavigate } from '@tanstack/react-router'
import { AlertCircle, ArrowLeft, Loader2, ShieldCheck } from 'lucide-react'
import { useEffect } from 'react'
import { useForm } from 'react-hook-form'

export default function SignInMfaPage() {
    const toast = useToast();
    const navigate = useNavigate();
    const search = Route.useSearch();
    const handleAuthRedirect = useAuthRedirect();

    const mfaToken = search.mfa_token;

    // 如果没有 mfa_token，重定向到登录页
    useEffect(() => {
        if (!mfaToken) {
            navigate({ to: '/sign-in', search: { redirect_uri: search.redirect_uri } });
        }
    }, [mfaToken, navigate, search.redirect_uri]);

    const form = useForm<MfaVerifyParamType>({
        resolver: zodResolver(MfaVerifyParamSchema),
        defaultValues: {
            mfa_token: mfaToken || '',
            code: '',
        },
    });

    const mutation = useMutation({
        mutationFn: async (data: MfaVerifyParamType) => {
            const res = await mfaVerify({
                mfa_token: mfaToken!,
                code: data.code,
            });
            if (res.status) {
                toast.success("登录成功");
                handleAuthRedirect();
            }
            return res;
        },
        onError: (error: any) => {
            // 检查是否是 MFA token 过期
            const state = error?.data?.state || error?.state;
            if (state === 'mfa-token-expired') {
                toast.error("验证已过期，请重新登录");
                navigate({ to: '/sign-in', search: { redirect_uri: search.redirect_uri } });
                return;
            }
            toast.error(formatServerError(error));
            // 清空验证码输入
            form.setValue('code', '');
        }
    });

    const isSubmitDisabled = mutation.isPending || form.watch('code').length !== 6;

    // 如果没有 mfa_token，显示错误状态
    if (!mfaToken) {
        return (
            <div className="flex flex-col items-center justify-center gap-4 py-8">
                <AlertCircle className="h-12 w-12 text-destructive" />
                <p className="text-sm text-muted-foreground">无效的验证请求</p>
                <Link to="/sign-in" search={{ redirect_uri: search.redirect_uri }}>
                    <Button variant="outline" size="sm">
                        <ArrowLeft className="h-4 w-4 mr-2" />
                        返回登录
                    </Button>
                </Link>
            </div>
        );
    }

    return (
        <div className="space-y-6">
            {/* 标题说明 */}
            <div className="flex flex-col items-center gap-2 text-center">
                <div className="flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
                    <ShieldCheck className="h-6 w-6 text-primary" />
                </div>
                <h2 className="text-lg font-semibold">两步验证</h2>
                <p className="text-sm text-muted-foreground">
                    请输入您的身份验证器应用中显示的6位验证码
                </p>
            </div>

            <Form {...form}>
                <form onSubmit={form.handleSubmit((data) => mutation.mutate(data))} className={cn("grid gap-4")}>
                    <FormField
                        control={form.control}
                        name='code'
                        render={({ field }) => (
                            <FormItem className="flex flex-col items-center">
                                <FormLabel className="sr-only">验证码</FormLabel>
                                <FormControl>
                                    <InputOTP
                                        maxLength={6}
                                        value={field.value}
                                        onChange={field.onChange}
                                        disabled={mutation.isPending}
                                    >
                                        <InputOTPGroup>
                                            <InputOTPSlot index={0} />
                                            <InputOTPSlot index={1} />
                                            <InputOTPSlot index={2} />
                                            <InputOTPSlot index={3} />
                                            <InputOTPSlot index={4} />
                                            <InputOTPSlot index={5} />
                                        </InputOTPGroup>
                                    </InputOTP>
                                </FormControl>
                                <FormMessage />
                            </FormItem>
                        )}
                    />

                    <Button type="submit" className={cn('mt-2 w-full max-w-[280px] mx-auto')} disabled={isSubmitDisabled}>
                        {mutation.isPending ? (
                            <>
                                <Loader2 className="h-4 w-4 animate-spin" />
                                <span className="ml-2">验证中...</span>
                            </>
                        ) : (
                            '验证'
                        )}
                    </Button>
                </form>
            </Form>

            {/* 返回登录链接 */}
            <div className="text-center">
                <Link
                    to="/sign-in"
                    search={{ redirect_uri: search.redirect_uri }}
                    className="text-sm text-muted-foreground hover:text-primary inline-flex items-center gap-1"
                >
                    <ArrowLeft className="h-3 w-3" />
                    使用其他方式登录
                </Link>
            </div>

            {/* 帮助信息 */}
            <div className="rounded-lg bg-muted/50 p-4 text-center">
                <p className="text-xs text-muted-foreground">
                    打开您的身份验证器应用（如 Microsoft Authenticator 等），
                    查看并输入当前显示的6位数字验证码。
                </p>
            </div>
        </div>
    )
}
