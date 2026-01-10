import { Route } from '@apps/main/routes/_auth/sign-in/app'
import { loginAppCode } from '@shared/apis/auth/login'
import { CaptchaData, CaptchaInput } from '@shared/components/custom/input/captcha-input'
import { Button } from '@shared/components/ui/button'
import { useToast } from '@shared/contexts/toast-context'
import { formatServerError } from '@shared/lib/utils'
import { useMutation } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { Loader2 } from 'lucide-react'
import { useState } from 'react'

import { useAuthRedirect } from '@apps/main/hooks/use-auth-redirect'

export default function SignInAppPage() {
    const toast = useToast()
    const search = Route.useSearch()
    const navigate = useNavigate()
    const handleAuthRedirect = useAuthRedirect()
    const [captchaData, setCaptchaData] = useState<CaptchaData>({
        code: '',
        key: '',
        validation: null,
    })

    // 登录操作
    const loginMutation = useMutation({
        mutationFn: async () => {
            if (!captchaData.code || !captchaData.key) {
                throw new Error('请输入验证码')
            }
            if (captchaData.validation !== 'success') {
                throw new Error('验证码验证失败，请重新输入')
            }

            const res = await loginAppCode({
                client_id: search.client_id,
                token_data: search.code,
                captcha: captchaData,
            })
            // 检查是否需要 MFA 验证
            if (res.mfa_token) {
                navigate({
                    to: '/sign-in/mfa',
                    search: { mfa_token: res.mfa_token, redirect_uri: search.redirect_uri }
                });
                return res;
            }
            if (res.status && res.response) {
                toast.success('登录成功')
                handleAuthRedirect()
            }
            return res
        },
        onError: (error: any) => {
            toast.error(formatServerError(error))
        },
    })

    const handleLogin = () => {
        loginMutation.mutate()
    }

    const handleCancel = () => {
        if (search.redirect_uri) {
            const redirectUrl = new URL(search.redirect_uri)
            redirectUrl.searchParams.set('error', 'access_denied')
            window.location.href = redirectUrl.toString()
        } else {
            window.history.back()
        }
    }

    const isLoginDisabled =
        loginMutation.isPending ||
        !captchaData.code ||
        captchaData.validation !== 'success'

    return (
        <>
            {/* 验证码输入 */}
            <div className="space-y-2">
                <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                    验证码
                </label>
                <CaptchaInput
                    captchaType="login"
                    value={captchaData}
                    onChange={setCaptchaData}
                    placeholder="请输入验证码"
                />
            </div>

            <div className="flex gap-3 mt-4">
                <Button
                    variant="outline"
                    className="flex-1"
                    onClick={handleCancel}
                    disabled={loginMutation.isPending}
                >
                    取消
                </Button>
                <Button
                    className="flex-1"
                    onClick={handleLogin}
                    disabled={isLoginDisabled}
                >
                    {loginMutation.isPending ? (
                        <>
                            <Loader2 className=" h-4 w-4 animate-spin" />
                            <span className="ml-2">登录中...</span>
                        </>
                    ) : (
                        '确认登录'
                    )}
                </Button>
            </div>
        </>
    )
}
