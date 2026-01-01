"use client"

import { appList, appRequestList, appSubAppRequest, type AppListItemType, type AppRequestItemType } from "@shared/apis/user/app"
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error"
import { CenteredLoading } from "@shared/components/custom/page-placeholder/centered-loading"
import { Alert, AlertDescription, AlertTitle } from "@shared/components/ui/alert"
import { Button } from "@shared/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@shared/components/ui/card"
import { useToast } from "@shared/contexts/toast-context"
import { AppDetailNavContainer, AppDetailServiceModuleNavContainer } from '@apps/main/features/user/components/ui/app-detail-nav'
import { AppStatusAlert } from "@apps/main/features/user/components/ui/app-status-alert"
import { cn, formatServerError, formatTime, getQueryResponseData, TIME_STYLE } from "@shared/lib/utils"
import { Route } from "@apps/main/routes/_main/user/app/$appId/service/sub-app"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { Link } from "@tanstack/react-router"
import { AlertCircle, ArrowRight, CheckCircle2 } from "lucide-react"
import { serviceModuleConfig } from '../nav-info'

interface AppServiceSubAppContentProps {
    appData: AppListItemType
    pendingRequest: AppRequestItemType | null | undefined
}

function AppServiceSubAppContent({
    appData,
    pendingRequest
}: AppServiceSubAppContentProps) {
    const { appId } = Route.useParams()
    const toast = useToast()
    const queryClient = useQueryClient()

    const isEnabled = appData.sup_app === true

    const requestMutation = useMutation({
        mutationFn: () => appSubAppRequest({ app_id: Number(appId) }),
        onSuccess: (result) => {
            if (result.status) {
                toast.success("子应用功能申请已提交")
                queryClient.invalidateQueries({ queryKey: ['app-sub-app-detail', appId] })
                queryClient.invalidateQueries({ queryKey: ['app-sub-app-pending-request', appId] })
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
                    <AlertTitle className={cn("text-sm font-medium")}>子应用功能申请审核中</AlertTitle>
                    <AlertDescription className={cn("text-xs")}>
                        <div className={cn("space-y-2 mt-1")}>
                            <div>
                                您在 {formatTime(pendingRequest.request_time, TIME_STYLE.ABSOLUTE_TEXT)} 提交的子应用功能申请正在审核中。
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
                        子应用功能
                        {isEnabled && <CheckCircle2 className={cn("h-5 w-5 text-muted-foreground")} />}
                    </CardTitle>
                    <CardDescription>
                        允许其他应用作为子应用接入，接受其他用户以你的应用为父应用申请子应用功能。
                    </CardDescription>
                </CardHeader>
                <CardContent>
                    {isEnabled ? (
                        <div className={cn("space-y-4")}>
                            <Alert>
                                <CheckCircle2 className={cn("h-4 w-4")} />
                                <div>
                                    <AlertTitle className={cn("text-sm font-medium")}>子应用功能已开通</AlertTitle>
                                    <AlertDescription className={cn("text-xs")}>
                                        您可以管理子应用和配置通知设置
                                    </AlertDescription>
                                </div>
                            </Alert>

                            <Alert variant="destructive" className={cn("border-l-4")}>
                                <AlertCircle className={cn("h-4 w-4")} />
                                <AlertTitle className={cn("text-sm font-medium")}>安全责任提示</AlertTitle>
                                <AlertDescription className={cn("text-xs")}>
                                    您审核通过的子应用由您负责其内容的安全，如果您的子应用出现违规可能导致您的应用被系统禁用。
                                </AlertDescription>
                            </Alert>

                            <div className={cn("pt-4 text-sm text-muted-foreground")}>
                                <p>您可以通过子应用管理页面管理接入的子应用，以及配置相关的通知设置。</p>
                            </div>
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

export function AppServiceSubAppPage() {
    //docs\api\user\app\sub_app_request.md
    // 未开通, 子应用权限[申请开通]
    // 已开通, 子应用权限[LINK:管理]
    // docs\api\user\app\sub_app_notify_get_config.md
    // docs\api\user\app\sub_app_notify_set_config.md
    const { appId } = Route.useParams()


    // 获取应用详情
    const {
        data: appQueryData,
        isLoading: isLoadingApp,
        isError: isAppError,
        error: appError
    } = useQuery({
        queryKey: ['app-sub-app-detail', appId],
        queryFn: ({ signal }) => appList({
            app_id: Number(appId),
            page: { page: 1, limit: 1 },
            attr_inner_feature: true,
            count_num: false
        }, { signal }),
        enabled: !!appId
    })

    const appData = getQueryResponseData<AppListItemType[]>(appQueryData, [])[0] ?? null

    // 检查是否有待审核的子应用功能申请 (request_type: 3 = 子应用功能申请, status: 1 = 待审核)
    const {
        data: requestQueryData,
        isLoading: isLoadingRequest,
        isError: isRequestError,
        error: requestError
    } = useQuery({
        queryKey: ['app-sub-app-pending-request', appId],
        queryFn: ({ signal }) => appRequestList({
            app_id: Number(appId),
            request_type: 3,
            status: 1,
            page: { page: 1, limit: 1 },
            count_num: false
        }, { signal }),
        enabled: !!appId
    })

    const pendingRequest = getQueryResponseData<AppRequestItemType[]>(requestQueryData, [])[0] ?? null

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
        <AppDetailServiceModuleNavContainer
            {...serviceModuleConfig}
            appData={appData}
        >
            <div className={cn("space-y-6")}>
                {/* 应用状态提示 */}
                <AppStatusAlert appData={appData} />

                <AppServiceSubAppContent
                    appData={appData}
                    pendingRequest={pendingRequest}
                />
            </div>


        </AppDetailServiceModuleNavContainer>
    )
}
