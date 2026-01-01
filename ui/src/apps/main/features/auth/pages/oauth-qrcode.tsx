
import { Route } from '@/apps/main/routes/_auth/_oauth/oauth-qrcode'
import { AppLogo } from '@apps/main/components/local/app-logo'
import { loginExterCallback } from '@shared/apis/auth/oauth'
import { Alert, AlertDescription, AlertTitle } from '@shared/components/ui/alert'
import { Button } from '@shared/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@shared/components/ui/card'
import { useToast } from '@shared/contexts/toast-context'
import { cn, formatServerError } from '@shared/lib/utils'
import { useMutation } from '@tanstack/react-query'
import { AlertCircle, Loader2, ShieldCheck } from 'lucide-react'
import { useState } from 'react'

export default function OAuthQrCodePage() {
  const toast = useToast()
  const search = Route.useSearch()
  const [isSuccess, setIsSuccess] = useState(false)
  const [successMessage, setSuccessMessage] = useState('')

  // 检查是否有完整的授权参数
  const hasAuthParams = Boolean(search.code && search.state && search.login_type)

  // 授权登录操作
  const authMutation = useMutation({
    mutationFn: async () => {
      if (!search.code || !search.state || !search.login_type) {
        throw new Error('缺少必要的授权参数')
      }
      return await loginExterCallback(search.login_type, {
        code: search.code,
        callback_state: search.state,
      })
    },
    onSuccess: (data) => {
      if (data.status) {
        setIsSuccess(true)
        setSuccessMessage(data.message || '授权登录成功')
        toast.success('授权登录成功')
      } else {
        toast.error(formatServerError(data))
      }
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  // 如果已经成功，显示成功页面
  if (isSuccess) {
    return (
      <div className="bg-primary-foreground container grid h-svh max-w-none items-center justify-center px-4 sm:px-0">
        <div className="mx-auto flex w-full flex-col justify-center space-y-2 py-8 sm:w-[480px] sm:p-8">
          <div className="mb-4 flex items-center justify-center">
            <AppLogo alt="Logo" className="mr-2 h-5" linkToHome />
            <h1 className="text-xl font-medium">扫码登录</h1>
          </div>
          <Alert className="border-green-500 bg-green-50 dark:bg-green-900/20">
            <ShieldCheck className="h-4 w-4 text-green-600" />
            <AlertTitle className="text-green-800 dark:text-green-300">
              授权完成
            </AlertTitle>
            <AlertDescription className="text-green-700 dark:text-green-400">
              {successMessage || '您已完成授权操作，请关闭此窗口并返回原页面。'}
            </AlertDescription>
          </Alert>
        </div>
      </div>
    )
  }

  // 如果没有完整参数，显示错误提示
  if (!hasAuthParams) {
    return (
      <div className="bg-primary-foreground container grid h-svh max-w-none items-center justify-center px-4 sm:px-0">
        <div className="mx-auto flex w-full flex-col justify-center space-y-2 py-8 sm:w-[480px] sm:p-8">
          <div className="mb-4 flex items-center justify-center">
            <AppLogo alt="Logo" className="mr-2 h-5" linkToHome />
            <h1 className="text-xl font-medium">扫码登录</h1>
          </div>
          <Card className={cn('gap-4')}>
            <CardHeader className={cn('items-center')}>
              <AlertCircle className="text-destructive h-12 w-12" />
              <CardTitle className={cn('text-lg tracking-tight')}>
                参数错误
              </CardTitle>
              <CardDescription className={cn('text-center')}>
                缺少必要的授权参数，请重新扫码授权
              </CardDescription>
            </CardHeader>
          </Card>
        </div>
      </div>
    )
  }

  // 显示授权确认页面
  return (
    <div className="bg-primary-foreground container grid h-svh max-w-none items-center justify-center px-4 sm:px-0">
      <div className="mx-auto flex w-full flex-col justify-center space-y-2 py-8 sm:w-[480px] sm:p-8">
        <div className="mb-4 flex items-center justify-center">
          <AppLogo alt="Logo" className="mr-2 h-5" linkToHome />
          <h1 className="text-xl font-medium">扫码登录</h1>
        </div>

        <Card className={cn('gap-4')}>
          <CardHeader className={cn('items-center')}>
            <CardTitle className={cn('text-lg tracking-tight')}>
              扫码授权登录
            </CardTitle>
            <CardDescription className={cn('text-center')}>
              您正在通过 {search.login_type} 进行授权登录
            </CardDescription>
          </CardHeader>

          <CardContent className={cn('space-y-4')}>
            <div className="bg-muted/50 rounded-lg p-4 text-center">
              <p className="text-muted-foreground text-sm">
                点击下方按钮确认授权登录
              </p>
            </div>
          </CardContent>

          <CardFooter className={cn('flex justify-center')}>
            <Button
              className={cn('w-full')}
              onClick={() => authMutation.mutate()}
              disabled={authMutation.isPending}
            >
              {authMutation.isPending ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  <span className="ml-2">授权中...</span>
                </>
              ) : (
                '授权登录'
              )}
            </Button>
          </CardFooter>
        </Card>
      </div>
    </div>
  )
}
