import { Route } from '@/apps/main/routes/_auth/_oauth/oauth'
import { useAuthData } from '@apps/main/hooks/use-auth-data'
import { oauthDo, oauthScope, OauthScopeItemType } from '@shared/apis/user/oauth'
import { Button } from '@shared/components/ui/button'
import { CenteredLoading } from '@shared/components/custom/page-placeholder/centered-loading'
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
import { useMutation, useQuery } from '@tanstack/react-query'
import { CheckCircle2, Loader2, Shield, XCircle } from 'lucide-react'

export function OAuthPage() {
    const toast = useToast()
    const search = Route.useSearch()
    const authData = useAuthData()

    // 使用 oauthScope 加载授权范围信息
    const scopeQuery = useQuery({
        queryKey: ['oauth', 'scope', search.client_id, search.scope],
        queryFn: async () => {
            const res = await oauthScope({
                client_id: search.client_id,
                scope: search.scope,
            })
            return res.response
        },
    })

    // 授权操作
    const authMutation = useMutation({
        mutationFn: async () => {
            return await oauthDo({
                client_id: search.client_id,
                scope: search.scope,
                redirect_uri: search.redirect_uri,
            })
        },
        onSuccess: (data) => {
            if (!data.status) {
                throw new Error(formatServerError(data))
            }
            toast.success('授权成功')
            // 构建重定向 URL
            const redirectUrl = new URL(search.redirect_uri)
            redirectUrl.searchParams.set('code', String(data.response?.code))
            // 如果 state 不为空，也拼接到 redirect_uri 中
            if (search.state && search.state.trim() !== '') {
                redirectUrl.searchParams.set('state', search.state)
            }
            // 跳转到回调地址
            window.location.href = redirectUrl.toString()
        },
        onError: (error: any) => {
            toast.error(formatServerError(error))
        },
    })

    // 取消授权，返回上一页或关闭窗口
    const handleCancel = () => {
        // 构建重定向 URL 并添加 error 参数
        const redirectUrl = new URL(search.redirect_uri)
        redirectUrl.searchParams.set('error', 'access_denied')
        if (search.state && search.state.trim() !== '') {
            redirectUrl.searchParams.set('state', search.state)
        }
        window.location.href = redirectUrl.toString()
    }

    // 渲染加载状态
    if (scopeQuery.isLoading) {
        return (
            <div className="bg-primary-foreground container grid h-svh max-w-none items-center justify-center">
                <CenteredLoading variant="content" iconSize="lg" />
            </div>
        )
    }

    // 渲染错误状态
    if (scopeQuery.isError) {
        return (
            <div className="bg-primary-foreground container grid h-svh max-w-none items-center justify-center">
                <Card className={cn('w-[420px] gap-4')}>
                    <CardHeader className={cn("items-center")}>
                        <XCircle className="text-destructive h-12 w-12" />
                        <CardTitle className={cn('text-lg tracking-tight')}>
                            授权信息加载失败
                        </CardTitle>
                        <CardDescription className={cn("text-center")}>
                            {formatServerError(scopeQuery.error)}
                        </CardDescription>
                    </CardHeader>
                    <CardFooter className={cn("justify-center")}>
                        <Button variant="outline" onClick={handleCancel}>
                            返回
                        </Button>
                    </CardFooter>
                </Card>
            </div>
        )
    }

    const scopeData = scopeQuery.data

    return (
        <div className="bg-primary-foreground container grid h-svh max-w-none items-center justify-center">
            <div className="mx-auto flex w-full flex-col justify-center space-y-2 py-8 sm:w-[480px] sm:p-8">
                <div className="mb-4 flex items-center justify-center">
                    <Shield className="text-primary mr-2 h-8 w-8" />
                    <h1 className="text-xl font-medium">应用授权</h1>
                </div>

                <Card className={cn('gap-4')}>
                    <CardHeader>
                        <CardTitle className={cn('text-lg tracking-tight')}>
                            授权确认
                        </CardTitle>
                        <CardDescription>
                            应用 <span className="font-medium">{search.client_id}</span>{' '}
                            请求访问您的账户
                        </CardDescription>
                    </CardHeader>

                    <CardContent className={cn("space-y-4")}>
                        {/* 用户信息 */}
                        <div className="bg-muted/50 rounded-lg p-4">
                            <p className="text-muted-foreground mb-1 text-sm">
                                当前登录账户
                            </p>
                            <p className="font-medium">{authData.userNikeName}</p>
                        </div>

                        {/* 权限列表 */}
                        <div>
                            <p className="text-muted-foreground mb-3 text-sm">
                                该应用将获得以下权限：
                            </p>
                            <ul className="space-y-2">
                                {scopeData?.scope.map((item: OauthScopeItemType) => (
                                    <li
                                        key={item.key}
                                        className="hover:bg-muted/50 flex items-start gap-3 rounded-lg p-2 transition-colors"
                                    >
                                        <CheckCircle2 className="text-primary mt-0.5 h-5 w-5 flex-shrink-0" />
                                        <div>
                                            <p className="font-medium">{item.name}</p>
                                            <p className="text-muted-foreground text-sm">
                                                {item.desc}
                                            </p>
                                        </div>
                                    </li>
                                ))}
                            </ul>
                        </div>
                    </CardContent>

                    <CardFooter className={cn("flex gap-3")}>
                        <Button
                            variant="outline"
                            className={cn("flex-1")}
                            onClick={handleCancel}
                            disabled={authMutation.isPending}
                        >
                            取消
                        </Button>
                        <Button
                            className={cn("flex-1")}
                            onClick={() => authMutation.mutate()}
                            disabled={authMutation.isPending}
                        >
                            {authMutation.isPending ? (
                                <>
                                    <Loader2 className=" h-4 w-4 animate-spin" />
                                    <span className="ml-2">授权中...</span>
                                </>
                            ) : (
                                '确认授权'
                            )}
                        </Button>
                    </CardFooter>
                </Card>

                <p className="text-muted-foreground px-8 text-center text-xs">
                    授权后，您的部分信息将被共享给该应用。
                </p>
            </div>
        </div>
    )
}
