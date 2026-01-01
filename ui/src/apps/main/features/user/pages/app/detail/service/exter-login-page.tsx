"use client"

import { appList, appRequestExterLogin, appRequestList, type AppListItemType, type AppRequestItemType } from "@shared/apis/user/app"
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error"
import { CenteredLoading } from "@shared/components/custom/page-placeholder/centered-loading"
import { Alert, AlertDescription, AlertTitle } from "@shared/components/ui/alert"
import { Button } from "@shared/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@shared/components/ui/card"
import { useToast } from "@shared/contexts/toast-context"
import { AppDetailNavContainer, AppDetailServiceModuleNavContainer } from '@apps/main/features/user/components/ui/app-detail-nav'
import { AppStatusAlert } from "@apps/main/features/user/components/ui/app-status-alert"

import { cn, formatServerError, formatTime, getQueryResponseData, TIME_STYLE } from "@shared/lib/utils"
import { Route } from "@apps/main/routes/_main/user/app/$appId/service/exter-login"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { Link } from "@tanstack/react-router"
import { AlertCircle, ArrowRight, CheckCircle2 } from "lucide-react"
import { serviceModuleConfig } from '../nav-info'

interface AppServiceExterLoginContentProps {
    appData: AppListItemType
    pendingRequest: AppRequestItemType | null | undefined
}

function AppServiceExterLoginContent({
    appData,
    pendingRequest
}: AppServiceExterLoginContentProps) {
    const { appId } = Route.useParams()
    const toast = useToast()
    const queryClient = useQueryClient()

    const isEnabled = appData.exter_login === true

    const requestMutation = useMutation({
        mutationFn: () => appRequestExterLogin({ app_id: Number(appId) }),
        onSuccess: (result) => {
            if (result.status) {
                toast.success("接口登录功能申请已提交")
                queryClient.invalidateQueries({ queryKey: ['app-exter-login-detail', appId] })
                queryClient.invalidateQueries({ queryKey: ['app-exter-login-pending-request', appId] })
            } else {
                toast.error(formatServerError(result))
            }
        },
        onError: (error: any) => {
            toast.error(formatServerError(error))
        }
    })

    const handleRequest = () => {
        requestMutation.mutate()
    }

    return (
        <div className={cn("space-y-6")}>
            {/* 显示待审核申请提示 */}
            {!isEnabled && pendingRequest && (
                <Alert className={cn("border-l-4")}>
                    <AlertCircle className={cn("h-4 w-4")} />
                    <AlertTitle className={cn("text-sm font-medium")}>接口登录申请审核中</AlertTitle>
                    <AlertDescription className={cn("text-xs")}>
                        <div className={cn("space-y-2 mt-1")}>
                            <div>
                                您在 {formatTime(pendingRequest.request_time, TIME_STYLE.ABSOLUTE_TEXT)} 提交的接口登录功能申请正在审核中。
                            </div>
                            <div className={cn("pt-2")}>
                                <Button variant="outline" size="sm" asChild>
                                    <Link
                                        to="/user/app/$appId/request"
                                        params={{ appId: appData.id }}
                                        search={{ id: pendingRequest.id }}
                                    >
                                        <ArrowRight className={cn("mr-2 h-4 w-4")} />
                                        查看请求详情
                                    </Link>
                                </Button>
                            </div>
                        </div>
                    </AlertDescription>
                </Alert>
            )}

            <Card>
                <CardHeader>
                    <CardTitle className={cn("flex items-center gap-2")}>
                        接口登录
                        {isEnabled && <CheckCircle2 className={cn("h-5 w-5 text-muted-foreground")} />}
                    </CardTitle>
                    <CardDescription>
                        允许通过API接口进行用户登录验证,适用于需要已有用户系统集成到本系统的进行管理的场景。
                    </CardDescription>
                </CardHeader>
                <CardContent>
                    {isEnabled ? (
                        <div className={cn("space-y-4")}>
                            <Alert >
                                <CheckCircle2 className={cn("h-4 w-4")} />

                                <AlertTitle className={cn("text-sm font-medium")}>接口登录功能已开通</AlertTitle>
                                <AlertDescription className={cn("text-xs")}>
                                    您可以使用API完成用户登录验证
                                </AlertDescription>

                            </Alert>
                        </div>
                    ) : (
                        <div className={cn("space-y-4")}>
                            <Button
                                onClick={handleRequest}
                                disabled={requestMutation.isPending || !!pendingRequest || appData.status !== 2}
                            >
                                {requestMutation.isPending ? "申请中..." : pendingRequest ? "已提交申请" : "申请开通"}
                            </Button>
                        </div>
                    )}
                </CardContent>
            </Card>
        </div>
    )
}

export function AppServiceExterLoginPage() {
    // docs\api\user\app\request_inner_feature_exter_login_request.md
    const { appId } = Route.useParams()

    // 获取应用详情
    const {
        data: appListQueryData,
        isLoading: isLoadingApp,
        isError: isAppError,
        error: appError
    } = useQuery({
        queryKey: ['app-exter-login-detail', appId],
        queryFn: ({ signal }) => appList({
            app_id: Number(appId),
            page: { page: 1, limit: 1 },
            attr_inner_feature: true,
            count_num: false
        }, { signal }),
        enabled: !!appId
    })

    const appData = getQueryResponseData<AppListItemType[]>(appListQueryData, [])[0] ?? null

    // 检查是否有待审核的接口登录申请 (request_type: 4 = 接口登录申请, status: 1 = 待审核)
    const {
        data: appRequestQueryData,
        isLoading: isLoadingRequest,
        isError: isRequestError,
        error: requestError
    } = useQuery({
        queryKey: ['app-exter-login-pending-request', appId],
        queryFn: ({ signal }) => appRequestList({
            app_id: Number(appId),
            request_type: 4,
            status: 1,
            page: { page: 1, limit: 1 },
            count_num: false
        }, { signal }),
        enabled: !!appId
    })

    const pendingRequest = getQueryResponseData<AppRequestItemType[]>(appRequestQueryData, [])[0] ?? null

    if (isLoadingApp || isLoadingRequest) {
        return (
            <AppDetailNavContainer {...serviceModuleConfig}>
                <CenteredLoading variant="card" />
            </AppDetailNavContainer>
        )
    }

    if (isAppError || isRequestError) {
        return (
            <AppDetailNavContainer {...serviceModuleConfig}>
                <CenteredError
                    variant="card"
                    error={appError || requestError}

                />
            </AppDetailNavContainer>
        )
    }

    if (!appData) {
        return (
            <AppDetailNavContainer {...serviceModuleConfig}>
                <CenteredError
                    variant="card"
                    error={new Error('应用不存在')}

                />
            </AppDetailNavContainer>
        )
    }

    return (
        <AppDetailServiceModuleNavContainer {...serviceModuleConfig} appData={appData}>
            <div className={cn("space-y-6")}>
                {/* 应用状态提示 */}
                <AppStatusAlert appData={appData} />

                <AppServiceExterLoginContent
                    appData={appData}
                    pendingRequest={pendingRequest}
                />
            </div>
        </AppDetailServiceModuleNavContainer>
    )
}
