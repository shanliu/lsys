"use client"

import { appList, appOAuthServerRequest, appRequestList, type AppListItemType, type AppRequestItemType } from "@shared/apis/user/app"
import { LoadingButton } from "@apps/main/components/local/sender-config/loading-button"
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error"
import { CenteredLoading } from "@shared/components/custom/page-placeholder/centered-loading"
import { Alert, AlertDescription, AlertTitle } from "@shared/components/ui/alert"
import { Button } from "@shared/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@shared/components/ui/card"
import { useToast } from "@shared/contexts/toast-context"
import { AppDetailNavContainer, AppDetailServiceModuleNavContainer } from '@apps/main/features/user/components/ui/app-detail-nav'
import { AppStatusAlert } from "@apps/main/features/user/components/ui/app-status-alert"
import { cn, formatServerError, formatTime, getQueryResponseData, TIME_STYLE } from "@shared/lib/utils"
import { Route } from "@apps/main/routes/_main/user/app/$appId/service/sub-app-oauth-server"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { Link } from "@tanstack/react-router"
import { AlertCircle, ArrowRight, CheckCircle2, Eye } from "lucide-react"
import React from "react"
import { serviceModuleConfig } from '../nav-info'
import { SubAppOAuthServerSettingDrawer } from './sub-app-oauth-server-setting-drawer'


interface AppServiceSubAppOauthServerContentProps {
    appData: AppListItemType
    pendingRequest: AppRequestItemType | null
    scopeDrawerOpen: boolean
    setScopeDrawerOpen: (open: boolean) => void
}

function AppServiceSubAppOauthServerContent({
    appData,
    pendingRequest,
    scopeDrawerOpen,
    setScopeDrawerOpen,
}: AppServiceSubAppOauthServerContentProps) {
    const { appId } = Route.useParams()
    const toast = useToast()
    const queryClient = useQueryClient()

    const isOAuthServerEnabled = appData.oauth_server === true
    const scopeData = React.useMemo(() => appData.oauth_server_scope_data || [], [appData.oauth_server_scope_data])

    // 申请OAuth服务
    const requestMutation = useMutation({
        mutationFn: () => appOAuthServerRequest({ app_id: Number(appId) }),
        onSuccess: (result) => {
            if (result.status) {
                toast.success("OAuth服务申请已提交")
                queryClient.invalidateQueries({ queryKey: ['app-oauth-server-detail', appId] })
                queryClient.invalidateQueries({ queryKey: ['app-oauth-server-pending-request', appId] })
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
            {!isOAuthServerEnabled && pendingRequest && (
                <Alert className={cn("border-l-4")}>
                    <AlertCircle className={cn("h-4 w-4")} />
                    <AlertTitle className={cn("text-sm font-medium")}>OAuth服务申请审核中</AlertTitle>
                    <AlertDescription className={cn("text-xs")}>
                        <div className={cn("space-y-2 mt-1")}>
                            <div>
                                您在 {formatTime(pendingRequest.request_time, TIME_STYLE.ABSOLUTE_TEXT)} 提交的OAuth服务申请正在审核中。
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
                    <CardTitle>OAuth服务权限</CardTitle>
                    <CardDescription>
                        {isOAuthServerEnabled
                            ? "OAuth服务已开通，您可以管理scope权限配置"
                            : "申请开通OAuth服务，为其他应用提供OAuth登录能力"}
                    </CardDescription>
                </CardHeader>
                <CardContent>
                    {isOAuthServerEnabled ? (
                        <div className={cn("space-y-4")}>
                            <Alert>
                                <CheckCircle2 className={cn("h-4 w-4")} />

                                <AlertTitle className={cn("text-sm font-medium")}>OAuth服务已开通</AlertTitle>
                                <AlertDescription className={cn("text-xs")}>
                                    <div className={cn("space-y-2 mt-1")}>
                                        <p>您的应用已开通OAuth服务功能。</p>
                                        {scopeData.length > 0 && (
                                            <div>
                                                <p className={cn("font-medium mb-1")}>当前配置的权限范围：</p>
                                                <div className={cn("ml-3 space-y-1")}>
                                                    {scopeData.map((scope, index) => (
                                                        <div key={index} className={cn("text-sm text-muted-foreground")}>
                                                            <span className={cn("text-[10px]")}>•</span>
                                                            <code className={cn("mx-1 px-1.5 py-0.5 rounded bg-muted")}>{scope.scope_key}</code>
                                                            <span>{scope.scope_name}</span>
                                                        </div>
                                                    ))}
                                                </div>
                                            </div>
                                        )}
                                    </div>
                                </AlertDescription>

                            </Alert>
                        </div>
                    ) : (
                        <div className={cn("space-y-4")}>
                            <LoadingButton
                                onClick={handleRequest}
                                loading={requestMutation.isPending}
                                disabled={!!pendingRequest || appData.status !== 2}
                            >
                                {pendingRequest ? "已提交申请" : "申请开通OAuth服务"}
                            </LoadingButton>
                        </div>
                    )}
                </CardContent>
            </Card>

            {/* Scope Settings Drawer */}
            <SubAppOAuthServerSettingDrawer
                appId={String(appId)}
                scopeData={scopeData}
                open={scopeDrawerOpen}
                onOpenChange={setScopeDrawerOpen}
            />
        </div>
    )
}

export function AppServiceSubAppOauthServerPage() {
    // docs\api\user\app\oauth_server_request.md
    // docs\api\user\app\oauth_server_setting.md 
    const { appId } = Route.useParams()
    const [scopeDrawerOpen, setScopeDrawerOpen] = React.useState(false)

    // 获取应用详情（包含oauth_server和oauth_server_scope_data）
    const {
        data: appQueryData,
        isLoading: isLoadingApp,
        isError: isAppError,
        error: appError
    } = useQuery({
        queryKey: ['app-oauth-server-detail', appId],
        queryFn: ({ signal }) => appList({
            app_id: Number(appId),
            attr_inner_feature: true,
            attr_oauth_server_data: true,
            page: { page: 1, limit: 1 },
            count_num: false
        }, { signal }),
        enabled: !!appId
    })

    const appData = getQueryResponseData<AppListItemType[]>(appQueryData, [])[0] ?? null

    // 检查是否有待审核的OAuth服务申请 (request_type: 5 = OAuth服务申请, status: 1 = 待审核)
    const {
        data: requestQueryData,
        isLoading: isLoadingRequest,
        isError: isRequestError,
        error: requestError
    } = useQuery({
        queryKey: ['app-oauth-server-pending-request', appId],
        queryFn: ({ signal }) => appRequestList({
            app_id: Number(appId),
            request_type: 5,
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

    const isOAuthServerEnabled = appData.oauth_server === true

    return (
        <AppDetailServiceModuleNavContainer
            {...serviceModuleConfig}
            appData={appData}
            actions={
                isOAuthServerEnabled ? (
                    <Button variant="outline" size="sm" onClick={() => setScopeDrawerOpen(true)}>
                        <Eye className={cn("mr-2 h-4 w-4")} />
                        查看权限配置
                    </Button>
                ) : undefined
            }
        >
            <div className={cn("space-y-6")}>
                {/* 应用状态提示 */}
                <AppStatusAlert appData={appData} />

                <AppServiceSubAppOauthServerContent
                    appData={appData}
                    pendingRequest={pendingRequest || null}
                    scopeDrawerOpen={scopeDrawerOpen}
                    setScopeDrawerOpen={setScopeDrawerOpen}
                />
            </div>
        </AppDetailServiceModuleNavContainer>
    )
}
