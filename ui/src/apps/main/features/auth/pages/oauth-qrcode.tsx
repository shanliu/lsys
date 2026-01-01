
import { Route } from '@/apps/main/routes/_auth/_oauth/oauth-qrcode'
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
import { AlertCircle, Loader2, LogIn, ShieldCheck } from 'lucide-react'
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
      <div className="bg-background text-foreground flex min-h-screen flex-col items-center justify-center p-4">
        <Alert className="max-w-md w-full border-green-500 bg-green-50 dark:bg-green-900/20">
          <ShieldCheck className="h-4 w-4 text-green-600" />
          <AlertTitle className="text-green-800 dark:text-green-300">
            授权完成
          </AlertTitle>
          <AlertDescription className="text-green-700 dark:text-green-400">
            {successMessage || '您已完成授权操作，请关闭此窗口并返回原页面。'}
          </AlertDescription>
        </Alert>
      </div>
    )
  }

  // 如果没有完整参数，显示错误提示
  if (!hasAuthParams) {
    return (
      <div className="bg-background text-foreground flex min-h-screen flex-col items-center justify-center p-4">
        <Card className={cn('w-full max-w-md gap-4')}>
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
    )
  }

  // 显示授权确认页面
  return (
    <div className="bg-background text-foreground flex min-h-screen flex-col items-center justify-center p-4">
      <Card className={cn('w-full max-w-md gap-4')}>
        <CardHeader className={cn('items-center')}>
          <LogIn className="text-primary h-12 w-12" />
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
  )
}
