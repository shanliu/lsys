"use client"

import { LoadingButton } from '@apps/main/components/local/sender-config/loading-button'
import { AppDetailNavContainer, AppDetailServiceModuleNavContainer } from '@apps/main/features/user/components/ui/app-detail-nav'
import { AppStatusAlert } from '@apps/main/features/user/components/ui/app-status-alert'
import { appQueryKey } from "@apps/main/lib/auth-utils"
import { Route } from '@apps/main/routes/_main/user/app/$appId/service/oauth-client'
import { appList, appOAuthClientRequest, appOAuthClientScopeData, appOAuthClientScopeRequest, appRequestList, type AppListItemType, type AppRequestItemType, type OAuthScopeItemType } from '@shared/apis/user/app'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { CenteredLoading } from '@shared/components/custom/page-placeholder/centered-loading'
import { Alert, AlertDescription, AlertTitle } from '@shared/components/ui/alert'
import { Badge } from '@shared/components/ui/badge'
import { Button } from '@shared/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@shared/components/ui/card'
import { Checkbox } from '@shared/components/ui/checkbox'
import { Label } from '@shared/components/ui/label'
import { useToast } from '@shared/contexts/toast-context'
import { cn, formatServerError, formatTime, getQueryResponseData, TIME_STYLE } from "@shared/lib/utils"
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link } from '@tanstack/react-router'
import { AlertCircle, ArrowRight, CheckCircle2, Clock, Key, Settings, XCircle } from 'lucide-react'
import React from 'react'
import { serviceModuleConfig } from '../nav-info'
import { OAuthClientSecretDrawer } from './oauth-client-secret-drawer'
import { OAuthClientSettingDrawer } from './oauth-client-setting-drawer'

interface AppServiceOauthClientContentProps {
    appData: AppListItemType
    allScopes: OAuthScopeItemType[]
    pendingServiceRequest: AppRequestItemType | null
    pendingScopeRequests: AppRequestItemType[]
}

function AppServiceOauthClientContent({
    appData,
    allScopes,
    pendingServiceRequest,
    pendingScopeRequests
}: AppServiceOauthClientContentProps) {
    const { appId } = Route.useParams()
    const toast = useToast()
    const queryClient = useQueryClient()
    const [selectedScopes, setSelectedScopes] = React.useState<string[]>([])

    // 已开通的scope列表
    const enabledScopes = React.useMemo(() => {
        if (!appData.oauth_client_data?.scope_data) return []
        return appData.oauth_client_data.scope_data.split(',').filter(Boolean)
    }, [appData.oauth_client_data])

    // 申请中的scope列表
    const pendingScopes = React.useMemo(() => {
        const scopes = new Set<string>()
        pendingScopeRequests.forEach(req => {
            req.oauth_client_data?.scope_data?.forEach((scope: string) => scopes.add(scope))
        })
        return Array.from(scopes)
    }, [pendingScopeRequests])

    // 可申请的scope列表（排除已开通和申请中的）
    const availableScopes = React.useMemo(() => {
        return allScopes.filter(scope =>
            !enabledScopes.includes(scope.key) &&
            !pendingScopes.includes(scope.key)
        )
    }, [allScopes, enabledScopes, pendingScopes])

    // 申请开通OAuth客户端服务
    const requestServiceMutation = useMutation({
        mutationFn: (data: { app_id: number; scope_data: string[] }) =>
            appOAuthClientRequest(data),
        onSuccess: (result) => {
            if (result.status) {
                toast.success('OAuth客户端服务申请已提交，等待审核')
                // 刷新所有相关数据
                queryClient.invalidateQueries({ queryKey: appQueryKey(appId), exact: false })
                queryClient.invalidateQueries({ queryKey: ['app-pending-service-request', appId] })
            } else {
                toast.error(formatServerError(result))
            }
        },
        onError: (error: any) => {
            toast.error(formatServerError(error))
        }
    })

    // 申请新的scope权限
    const requestScopeMutation = useMutation({
        mutationFn: (data: { app_id: number; scope_data: string[] }) =>
            appOAuthClientScopeRequest(data),
        onSuccess: (result) => {
            if (result.status) {
                toast.success('权限申请已提交，等待审核')
                // 刷新所有相关数据
                queryClient.invalidateQueries({ queryKey: appQueryKey(appId), exact: false })
                queryClient.invalidateQueries({ queryKey: ['app-pending-scope-requests', appId] })
            } else {
                toast.error(formatServerError(result))
            }
        },
        onError: (error: any) => {
            toast.error(formatServerError(error))
        }
    })

    const handleRequestService = () => {
        requestServiceMutation.mutate({
            app_id: Number(appId),
            scope_data: selectedScopes
        })
    }

    const handleToggleScope = (scopeKey: string, checked: boolean) => {
        if (checked) {
            setSelectedScopes(prev => [...prev, scopeKey])
        } else {
            setSelectedScopes(prev => prev.filter(key => key !== scopeKey))
        }
    }

    const handleSelectAllScopes = (checked: boolean) => {
        if (checked) {
            setSelectedScopes(allScopes.map(s => s.key))
        } else {
            setSelectedScopes([])
        }
    }

    const handleRequestScope = (scopeKey: string) => {
        requestScopeMutation.mutate({
            app_id: Number(appId),
            scope_data: [scopeKey]
        })
    }

    // 服务未开通的情况
    if (!appData.oauth_client) {
        return (
            <div className={cn("space-y-6")}>
                {/* 显示待审核申请提示 */}
                {pendingServiceRequest && (
                    <Alert className={cn("border-l-4")}>
                        <AlertCircle className={cn("h-4 w-4")} />
                        <AlertTitle className={cn("text-sm font-medium")}>OAuth客户端服务申请审核中</AlertTitle>
                        <AlertDescription className={cn("text-xs")}>
                            <div className={cn("space-y-2 mt-1")}>
                                <div>
                                    您在 {formatTime(pendingServiceRequest.request_time, TIME_STYLE.ABSOLUTE_TEXT)} 提交的OAuth客户端服务申请正在审核中。
                                </div>
                                <div className={cn("pt-2")}>
                                    <Button variant="outline" size="sm" asChild>
                                        <Link
                                            to="/user/app/$appId/request"
                                            params={{ appId: appData.id }}
                                            search={{ id: pendingServiceRequest.id }}
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
                        <CardTitle>OAuth客户端服务</CardTitle>
                        <CardDescription>
                            开通后，您的应用可以使用OAuth 2.0协议进行第三方登录授权
                        </CardDescription>
                    </CardHeader>
                    <CardContent>
                        <div className={cn("space-y-6")}>
                            {/* 权限选择 */}
                            {allScopes.length > 0 && (
                                <div className={cn("space-y-4")}>
                                    <div className={cn("flex items-center justify-between")}>
                                        <Label className={cn("text-base font-medium")}>选择需要开通的权限</Label>
                                        <Button
                                            variant="ghost"
                                            size="sm"
                                            onClick={() => handleSelectAllScopes(selectedScopes.length !== allScopes.length)}
                                            disabled={!!pendingServiceRequest || appData.status !== 2}
                                        >
                                            {selectedScopes.length === allScopes.length ? '取消全选' : '全选'}
                                        </Button>
                                    </div>
                                    <div className={cn("space-y-3")}>
                                        {allScopes.map(scope => (
                                            <div
                                                key={scope.key}
                                                className={cn(
                                                    "flex items-start gap-3 p-4 rounded-lg border transition-colors",
                                                    selectedScopes.includes(scope.key) && "border-primary bg-primary/5",
                                                    (appData.status !== 2 || !!pendingServiceRequest) && "cursor-not-allowed opacity-60"
                                                )}
                                                onClick={(e) => {
                                                    if (appData.status !== 2 || !!pendingServiceRequest) {
                                                        e.preventDefault()
                                                        e.stopPropagation()
                                                    }
                                                }}
                                            >
                                                <Checkbox
                                                    id={`scope-${scope.key}`}
                                                    checked={selectedScopes.includes(scope.key)}
                                                    onCheckedChange={(checked) => handleToggleScope(scope.key, checked as boolean)}
                                                    disabled={!!pendingServiceRequest || appData.status !== 2}
                                                    className={cn("mt-1")}
                                                />
                                                <div className={cn("flex-1 min-w-0")}>
                                                    <div className={cn("flex items-center gap-2")}>
                                                        <Label
                                                            htmlFor={`scope-${scope.key}`}
                                                            className={cn("font-medium cursor-pointer")}
                                                        >
                                                            {scope.name}
                                                        </Label>
                                                        <Badge variant="outline">
                                                            {scope.key}
                                                        </Badge>
                                                    </div>
                                                    <div className={cn("text-sm text-muted-foreground mt-1")}>
                                                        {scope.desc}
                                                    </div>
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            )}

                            <div className={cn("flex items-center gap-3")}>
                                <LoadingButton
                                    onClick={handleRequestService}
                                    loading={requestServiceMutation.isPending}
                                    disabled={!!pendingServiceRequest || appData.status !== 2}
                                >
                                    {pendingServiceRequest ? "已提交申请" : "申请开通"}
                                </LoadingButton>
                                {selectedScopes.length > 0 && (
                                    <span className={cn("text-sm text-muted-foreground")}>
                                        已选择 {selectedScopes.length} 个权限
                                    </span>
                                )}
                            </div>
                        </div>
                    </CardContent>
                </Card>
            </div>
        )
    }

    // 服务已开通的情况
    return (
        <div className={cn("space-y-6")}>
            {/* 已开通的权限 */}
            {enabledScopes.length > 0 && (
                <Card>
                    <CardHeader>
                        <CardTitle>OAuth客户端服务已开通</CardTitle>
                        <CardDescription>以下权限已开通，可以直接使用</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <div className={cn("space-y-3")}>
                            {enabledScopes.map(scopeKey => {
                                const scopeInfo = allScopes.find(s => s.key === scopeKey)
                                return (
                                    <div key={scopeKey} className={cn("flex items-start gap-3 p-3 rounded-lg border")}>
                                        <CheckCircle2 className={cn("h-5 w-5 text-muted-foreground mt-0.5 flex-shrink-0")} />
                                        <div className={cn("flex-1 min-w-0")}>
                                            <div className={cn("flex items-center gap-2")}>
                                                <div className={cn("font-medium")}>{scopeInfo?.name || scopeKey}</div>
                                                <Badge variant="outline">
                                                    {scopeKey}
                                                </Badge>
                                            </div>
                                            <div className={cn("text-sm text-muted-foreground mt-1")}>
                                                {scopeInfo?.desc || '暂无描述'}
                                            </div>
                                        </div>
                                    </div>
                                )
                            })}
                        </div>
                    </CardContent>
                </Card>
            )}

            {/* 申请中的权限 */}
            {pendingScopeRequests.length > 0 && (
                <Card>
                    <CardHeader>
                        <CardTitle className={cn("flex items-center gap-2")}>
                            <Clock className={cn("h-5 w-5 text-gray-500 dark:text-white")} />
                            申请审核中的权限
                        </CardTitle>
                        <CardDescription>以下权限正在审核中</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <div className={cn("space-y-3")}>
                            {pendingScopeRequests.map(request => (
                                request.oauth_client_data?.scope_data?.map((scopeKey: string) => {
                                    const scopeInfo = allScopes.find(s => s.key === scopeKey)
                                    return (
                                        <div key={`${request.id}-${scopeKey}`} className={cn("flex items-start gap-3 p-3 rounded-lg border border-gray-200 dark:border-white/20 bg-gray-50 dark:bg-white/5")}>
                                            <Clock className={cn("h-5 w-5 text-gray-500 dark:text-white mt-0.5 flex-shrink-0")} />
                                            <div className={cn("flex-1 min-w-0")}>
                                                <div className={cn("flex items-center gap-2")}>
                                                    <div className={cn("font-medium")}>{scopeInfo?.name || scopeKey}</div>
                                                    <Badge variant="outline">{scopeKey}</Badge>
                                                </div>
                                                <div className={cn("text-sm text-muted-foreground mt-1")}>
                                                    {scopeInfo?.desc || '暂无描述'}
                                                </div>
                                                <div className={cn("text-xs text-muted-foreground mt-2")}>
                                                    申请时间：{formatTime(request.request_time, TIME_STYLE.ABSOLUTE_TEXT)}
                                                </div>
                                            </div>
                                        </div>
                                    )
                                })
                            ))}
                        </div>
                    </CardContent>
                </Card>
            )}

            {/* 可申请的权限 */}
            {availableScopes.length > 0 && (
                <Card>
                    <CardHeader>
                        <CardTitle>可申请的权限</CardTitle>
                        <CardDescription>选择需要的权限进行申请</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <div className={cn("space-y-3")}>
                            {availableScopes.map(scope => (
                                <div key={scope.key} className={cn("flex items-start gap-3 p-3 rounded-lg border")}>
                                    <XCircle className={cn("h-5 w-5 text-gray-400 mt-0.5 flex-shrink-0")} />
                                    <div className={cn("flex-1 min-w-0")}>
                                        <div className={cn("flex items-center gap-2")}>
                                            <div className={cn("font-medium")}>{scope.name}</div>
                                            <Badge variant="outline">
                                                {scope.key}
                                            </Badge>
                                        </div>
                                        <div className={cn("text-sm text-muted-foreground mt-1")}>
                                            {scope.desc}
                                        </div>
                                    </div>
                                    <LoadingButton
                                        size="sm"
                                        onClick={() => handleRequestScope(scope.key)}
                                        loading={requestScopeMutation.isPending}
                                        disabled={appData.status !== 2}
                                    >
                                        申请权限
                                    </LoadingButton>
                                </div>
                            ))}
                        </div>
                    </CardContent>
                </Card>
            )}

            {availableScopes.length === 0 && pendingScopeRequests.length === 0 && enabledScopes.length === allScopes.length && (
                <Card>
                    <CardContent className={cn("pt-6")}>
                        <div className={cn("text-center text-muted-foreground")}>
                            所有可用权限已开通
                        </div>
                    </CardContent>
                </Card>
            )}
        </div>
    )
}

export function AppServiceOauthClientPage() {
    // docs\api\user\app\oauth_client_request.md
    // docs\api\user\app\oauth_client_scope_data.md
    // docs\api\user\app\oauth_client_scope_request.md
    // docs\api\user\app\app_secret_view.md[self]
    // docs\api\user\app\oauth_client_secret_add.md
    // docs\api\user\app\oauth_client_secret_change.md
    // docs\api\user\app\oauth_client_secret_del.md
    // docs\api\user\app\oauth_client_set_domain.md

    const { appId } = Route.useParams()
    const [secretDrawerOpen, setSecretDrawerOpen] = React.useState(false)
    const [settingDrawerOpen, setSettingDrawerOpen] = React.useState(false)

    // 获取应用详情（包含inner_feature和oauth_client_data）
    const {
        data: appQueryData,
        isLoading: isLoadingApp,
        isError: isAppError,
        error: appError
    } = useQuery({
        queryKey: appQueryKey(appId, {
            attr_inner_feature: true,
            attr_oauth_client_data: true,
        }),
        queryFn: ({ signal }) => appList({
            app_id: Number(appId),
            attr_inner_feature: true,
            attr_oauth_client_data: true,
            page: { page: 1, limit: 1 },
            count_num: false
        }, { signal }),
        enabled: !!appId
    })

    const appData = getQueryResponseData<AppListItemType[]>(appQueryData, [])[0] ?? null

    // 获取所有可用的OAuth scope列表
    const {
        data: scopesQueryData,
        isLoading: isLoadingScopes,
        isError: isScopesError,
        error: scopesError
    } = useQuery({
        queryKey: ['oauth-client-scopes', appId, appData?.parent_app?.id],
        queryFn: async ({ signal }) => {
            const result = await appOAuthClientScopeData({
                app_id: appData?.parent_app?.id || 0
            }, { signal })
            if (!result.status) {
                throw result
            }
            return result.response?.scope || []
        },
        enabled: !!appId && !!appData
    })

    const allScopes = scopesQueryData || []

    // 获取OAuth客户端服务开通申请（request_type: 6）
    const {
        data: serviceRequestQueryData,
        isLoading: isLoadingServiceRequest,
        isError: isServiceRequestError,
        error: serviceRequestError
    } = useQuery({
        queryKey: ['app-pending-service-request', appId],
        queryFn: ({ signal }) => appRequestList({
            app_id: Number(appId),
            request_type: 6,
            status: 1,
            page: { page: 1, limit: 1 },
            count_num: false
        }, { signal }),
        enabled: !!appId && !!appData
    })

    const pendingServiceRequest = getQueryResponseData<AppRequestItemType[]>(serviceRequestQueryData, [])[0] ?? null

    // 获取OAuth scope权限申请列表（request_type: 7）
    const {
        data: scopeRequestsQueryData,
        isLoading: isLoadingScopeRequests,
        isError: isScopeRequestsError,
        error: scopeRequestsError
    } = useQuery({
        queryKey: ['app-pending-scope-requests', appId],
        queryFn: ({ signal }) => appRequestList({
            app_id: Number(appId),
            request_type: 7,
            status: 1,
            page: { page: 1, limit: 1 },
            count_num: false
        }, { signal }),
        enabled: !!appId && !!appData
    })

    const pendingScopeRequests = getQueryResponseData<AppRequestItemType[]>(scopeRequestsQueryData, [])

    if (isLoadingApp || isLoadingScopes || isLoadingServiceRequest || isLoadingScopeRequests) {
        return (
            <AppDetailNavContainer {...serviceModuleConfig}>
                <CenteredLoading variant="card" />
            </AppDetailNavContainer>
        )
    }

    if (isAppError || isScopesError || isServiceRequestError || isScopeRequestsError) {
        return (
            <AppDetailNavContainer {...serviceModuleConfig}>
                <CenteredError
                    variant="card"
                    error={appError || scopesError || serviceRequestError || scopeRequestsError}

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

    // 判断是否显示OAuth密钥管理按钮：应用状态为2(已审核)且oauth_client为true
    const showSecretManagement = appData.status === 2 && appData.oauth_client

    return (
        <>
            <AppDetailServiceModuleNavContainer
                {...serviceModuleConfig}
                appData={appData}
                actions={showSecretManagement ? (
                    <div className={cn("flex items-center gap-2")}>
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => setSettingDrawerOpen(true)}
                        >
                            <Settings className={cn("mr-2 h-4 w-4")} />
                            OAuth设置
                        </Button>
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => setSecretDrawerOpen(true)}
                        >
                            <Key className={cn("mr-2 h-4 w-4")} />
                            OAuth密钥管理
                        </Button>
                    </div>
                ) : undefined}
            >
                <div className={cn("space-y-6")}>
                    {/* 应用状态提示 */}
                    <AppStatusAlert appData={appData} />

                    <AppServiceOauthClientContent
                        appData={appData}
                        allScopes={allScopes || []}
                        pendingServiceRequest={pendingServiceRequest || null}
                        pendingScopeRequests={pendingScopeRequests || []}
                    />
                </div>
            </AppDetailServiceModuleNavContainer>

            {/* OAuth密钥管理抽屉 */}
            <OAuthClientSecretDrawer
                appId={String(appId)}
                open={secretDrawerOpen}
                onOpenChange={setSecretDrawerOpen}
            />

            {/* OAuth设置抽屉 */}
            <OAuthClientSettingDrawer
                appId={String(appId)}
                currentDomain={appData.oauth_client_data?.callback_domain}
                open={settingDrawerOpen}
                onOpenChange={setSettingDrawerOpen}
            />
        </>
    )
}
