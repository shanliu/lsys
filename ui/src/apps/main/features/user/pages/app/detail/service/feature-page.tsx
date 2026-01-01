"use client"

import { LoadingButton } from "@apps/main/components/local/sender-config/loading-button"
import { AppDetailNavContainer, AppDetailServiceModuleNavContainer } from '@apps/main/features/user/components/ui/app-detail-nav'
import { AppStatusAlert } from "@apps/main/features/user/components/ui/app-status-alert"
import { useDictData } from "@apps/main/hooks/use-dict-data"
import { appQueryKey } from '@apps/main/lib/auth-utils'
import { Route } from "@apps/main/routes/_main/user/app/$appId/service/feature"
import { appList, appRequestExterFeature, appRequestList, type AppListItemType, type AppRequestItemType } from "@shared/apis/user/app"
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error"
import { CenteredLoading } from "@shared/components/custom/page-placeholder/centered-loading"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@shared/components/ui/card"
import { Checkbox } from "@shared/components/ui/checkbox"
import { Label } from "@shared/components/ui/label"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@shared/components/ui/tooltip"
import { useToast } from "@shared/contexts/toast-context"
import { cn, formatServerError, formatTime, getQueryResponseData, TIME_STYLE } from "@shared/lib/utils"
import { DictList } from "@shared/types/apis-dict"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { Link } from "@tanstack/react-router"
import { Clock } from "lucide-react"
import React from "react"

import { serviceModuleConfig } from '../nav-info'

interface AppServiceFeatureContentProps {
  appData: AppListItemType
  exterFeatures: DictList
  pendingRequests: AppRequestItemType[]
}

function AppServiceFeatureContent({
  appData,
  exterFeatures,
  pendingRequests
}: AppServiceFeatureContentProps) {
  const toast = useToast()
  const queryClient = useQueryClient()
  const { appId } = Route.useParams()

  // 辅助函数：去掉 feature- 前缀，将 API 格式转换为字典格式
  const normalizeFeatureKey = (key: string) => {
    return key.replace(/^feature-/, '')
  }

  // 已开通的服务列表（规范化为字典格式）
  const enabledFeatures = React.useMemo(() => {
    return (appData.exter_feature || []).map(normalizeFeatureKey)
  }, [appData.exter_feature])

  // 正在申请的服务列表（从 pendingRequests 中提取，保持原始格式，附带匹配信息）
  const requestingFeatures = React.useMemo(() => {
    const features: Array<{
      originalKey: string  // API 中的原始值
      displayName: string  // 显示的名称
      matchedKey?: string  // 匹配到的字典 key
      request: AppRequestItemType  // 关联的请求
    }> = []

    pendingRequests.forEach(request => {
      if (request.feature_data && typeof request.feature_data === 'object' && request.feature_data.feature) {
        const featureStr = request.feature_data.feature as string
        if (featureStr) {
          featureStr.split(',').forEach(f => {
            const trimmed = f.trim()
            const normalizedKey = normalizeFeatureKey(trimmed)

            // 尝试在字典中查找匹配项
            const matchedFeature = exterFeatures.findItem(normalizedKey)

            features.push({
              originalKey: trimmed,
              displayName: matchedFeature?.val || trimmed,  // 匹配上用字典名，否则用原始值
              matchedKey: matchedFeature ? normalizedKey : undefined,
              request
            })
          })
        }
      }
    })
    return features
  }, [pendingRequests, exterFeatures])

  // 可申请的服务列表（exterFeatures 中排除已开通和申请中的）
  const availableFeatures = React.useMemo(() => {
    // 收集所有已占用的 key（已开通 + 申请中已匹配的）
    const usedKeys = new Set([
      ...enabledFeatures,
      ...requestingFeatures.filter(f => f.matchedKey).map(f => f.matchedKey!)
    ])

    return exterFeatures.filter((feature) => !usedKeys.has(feature.key))
  }, [exterFeatures, enabledFeatures, requestingFeatures])

  // 选中的服务
  const [selectedFeatures, setSelectedFeatures] = React.useState<string[]>([])

  // 提交申请
  const requestMutation = useMutation({
    mutationFn: (features: string[]) => appRequestExterFeature({
      app_id: Number(appId),
      featuer_data: features // 直接使用字典格式的 key，不需要添加前缀
    }),
    onSuccess: (result) => {
      if (result.status) {
        toast.success("服务申请已提交，等待审核")
        setSelectedFeatures([])
        // 重新加载数据
        queryClient.invalidateQueries({ queryKey: appQueryKey(appId), exact: false })
        queryClient.invalidateQueries({ queryKey: ['app-feature-requests', appId] })
      } else {
        toast.error(formatServerError(result))
      }
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    }
  })

  const handleSubmit = () => {
    if (selectedFeatures.length === 0) {
      toast.warning("请至少选择一个服务")
      return
    }
    requestMutation.mutate(selectedFeatures)
  }

  const handleFeatureToggle = (featureKey: string) => {
    setSelectedFeatures(prev =>
      prev.includes(featureKey)
        ? prev.filter(k => k !== featureKey)
        : [...prev, featureKey]
    )
  }

  return (
    <div className={cn("space-y-6")}>
      {/* 已开通的服务 - 仅在有已开通服务时显示 */}
      {enabledFeatures.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>已开通服务</CardTitle>
            <CardDescription>当前应用已开通的外部功能服务</CardDescription>
          </CardHeader>
          <CardContent>
            <div className={cn("space-y-2")}>
              {enabledFeatures.map(featureKey => {
                const featureInfo = exterFeatures.findItem(featureKey)
                return (
                  <div
                    key={featureKey}
                    className={cn(
                      "flex items-center gap-2 p-3 rounded-lg border bg-muted/50"
                    )}
                  >
                    <div className={cn("flex-1")}>
                      <div className={cn("font-medium")}>{featureInfo?.val || featureKey}</div>
                      {featureInfo?.val && (
                        <div className={cn("text-xs text-muted-foreground")}>
                          {featureKey}
                        </div>
                      )}
                    </div>
                  </div>
                )
              })}
            </div>
          </CardContent>
        </Card>
      )}

      {/* 未开通服务 - 仅在有未开通服务时显示 */}
      {(requestingFeatures.length > 0 || availableFeatures.length > 0) && (
        <Card>
          <CardHeader>
            <CardTitle>未开通服务</CardTitle>
            <CardDescription>可申请开通的外部功能服务</CardDescription>
          </CardHeader>
          <CardContent className={cn("space-y-4")}>
            {/* 申请中的服务 */}
            {requestingFeatures.length > 0 && (
              <div className={cn("space-y-2")}>
                <div className={cn("text-sm font-medium text-muted-foreground")}>
                  申请中的服务
                </div>
                <div className={cn("space-y-2")}>
                  {requestingFeatures.map((feature, index) => (
                    <div
                      key={`${feature.originalKey}-${index}`}
                      className={cn(
                        "flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-3 p-3 rounded-lg border bg-muted/30"
                      )}
                    >
                      <div className={cn("flex items-center gap-2 sm:gap-3 flex-1 min-w-0")}>
                        <Clock className={cn("h-4 w-4 text-muted-foreground flex-shrink-0")} />
                        <div className={cn("flex-1 min-w-0")}>
                          <div className={cn("flex flex-wrap items-baseline gap-x-2 gap-y-1")}>
                            <span className={cn("font-medium")}>{feature.displayName}</span>
                            <span className={cn("text-xs text-muted-foreground")}>{feature.originalKey}</span>
                          </div>
                          <div className={cn("text-xs text-muted-foreground mt-1 sm:hidden")}>
                            申请时间：{formatTime(feature.request.request_time, TIME_STYLE.ABSOLUTE_TEXT)}
                          </div>
                        </div>
                      </div>
                      <div className={cn("flex items-center justify-between sm:justify-end gap-2 pl-6 sm:pl-0")}>
                        <span className={cn("text-xs text-muted-foreground hidden sm:inline")}>
                          · 申请时间：{formatTime(feature.request.request_time, TIME_STYLE.ABSOLUTE_TEXT)}
                        </span>
                        <div className={cn("text-xs text-muted-foreground font-medium")}>
                          审核中
                        </div>
                        <Link
                          to="/user/app/$appId/request"
                          params={{ appId }}
                          search={{ id: feature.request.id }}
                          className={cn("text-xs text-primary hover:underline")}
                        >
                          详细
                        </Link>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* 可申请的服务 */}
            {availableFeatures.length > 0 && (
              <div className={cn("space-y-2")}>
                {requestingFeatures.length > 0 && (
                  <div className={cn("text-sm font-medium text-muted-foreground pt-2")}>
                    可申请的服务
                  </div>
                )}
                <div className={cn("space-y-2")}>
                  {availableFeatures.map((feature) => (
                    <div
                      key={feature.key}
                      className={cn(
                        "flex items-center gap-3 p-3 rounded-lg border transition-colors",
                        appData.status !== 1 && appData.status !== 3 ? "hover:bg-muted/50 cursor-pointer" : "cursor-not-allowed opacity-60"
                      )}
                      onClick={() => {
                        if (appData.status !== 1 && appData.status !== 3) {
                          handleFeatureToggle(feature.key)
                        }
                      }}
                    >
                      <Checkbox
                        id={`checkbox-${feature.key}`}
                        checked={selectedFeatures.includes(feature.key)}
                        onCheckedChange={() => handleFeatureToggle(feature.key)}
                        disabled={appData.status === 1 || appData.status === 3}
                        onClick={(e) => e.stopPropagation()}
                      />
                      <Label
                        htmlFor={`checkbox-${feature.key}`}
                        className={cn("flex-1 cursor-pointer")}
                        onClick={(e) => e.stopPropagation()}
                      >
                        <div className={cn("flex items-baseline gap-2")}>
                          <span className={cn("font-medium")}>{feature.val}</span>
                          <span className={cn("text-xs text-muted-foreground")}>
                            {feature.key}
                          </span>
                        </div>
                      </Label>
                    </div>
                  ))}
                </div>

                {/* 提交按钮 */}
                <div className={cn("pt-4")}>
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <span className={cn("inline-block")}>
                          <LoadingButton
                            onClick={handleSubmit}
                            loading={requestMutation.isPending}
                            disabled={appData.status === 1 || appData.status === 3 || selectedFeatures.length === 0}
                          >
                            提交申请 {selectedFeatures.length > 0 && `(${selectedFeatures.length} 个服务)`}
                          </LoadingButton>
                        </span>
                      </TooltipTrigger>
                      {selectedFeatures.length === 0 && (appData.status !== 1 && appData.status !== 3) && (
                        <TooltipContent>
                          <p>请选择开通服务</p>
                        </TooltipContent>
                      )}
                    </Tooltip>
                  </TooltipProvider>
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  )
}

export function AppServiceFeaturePage() {
  const { appId } = Route.useParams()

  // 加载字典数据
  const {
    dictData,
    isLoading: isDictLoading,
    isError: isDictError,
    errors: dictErrors
  } = useDictData(['user_app'] as const)

  // 获取应用详情（带外部功能信息）
  const {
    data: appQueryData,
    isLoading: isLoadingApp,
    isError: isAppError,
    error: appError
  } = useQuery({
    queryKey: appQueryKey(appId, {
      attr_exter_feature: true,
    }),
    queryFn: ({ signal }) => appList({
      app_id: Number(appId),
      attr_exter_feature: true,
      page: { page: 1, limit: 1 },
      count_num: false
    }, { signal }),
    enabled: !!appId
  })

  const appData = getQueryResponseData<AppListItemType[]>(appQueryData, [])[0] ?? null

  // 获取待审核的外部功能申请 (request_type: 8 = 外部功能申请, status: 1 = 待审核)
  const {
    data: requestQueryData,
    isLoading: isLoadingRequests,
    isError: isRequestsError,
    error: requestsError
  } = useQuery({
    queryKey: ['app-feature-requests', appId],
    queryFn: ({ signal }) => appRequestList({
      app_id: Number(appId),
      request_type: 8,
      status: 1,
      page: { page: 1, limit: 100 },
      count_num: false
    }, { signal }),
    enabled: !!appId
  })

  const pendingRequests = getQueryResponseData<AppRequestItemType[]>(requestQueryData, [])

  if (isLoadingApp || isLoadingRequests || isDictLoading) {
    return (
      <AppDetailNavContainer {...serviceModuleConfig}>
        <CenteredLoading variant="card" />
      </AppDetailNavContainer>
    )
  }

  if (isAppError || isRequestsError || isDictError) {
    return (
      <AppDetailNavContainer {...serviceModuleConfig}>
        <CenteredError
          variant="card"
          error={appError || requestsError || (dictErrors && dictErrors[0])}

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

        <AppServiceFeatureContent
          appData={appData}
          exterFeatures={dictData.exter_features}
          pendingRequests={pendingRequests || []}
        />
      </div>
    </AppDetailServiceModuleNavContainer>
  )
}
