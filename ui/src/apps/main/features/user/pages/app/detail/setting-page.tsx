"use client"

import { LoadingButton } from "@apps/main/components/local/sender-config/loading-button"
import { AppDetailNavContainer } from "@apps/main/features/user/components/ui/app-detail-nav"
import { AppStatusAlert } from "@apps/main/features/user/components/ui/app-status-alert"
import { appQueryKey } from "@apps/main/lib/auth-utils"
import { Route } from "@apps/main/routes/_main/user/app/$appId/setting"
import { zodResolver } from "@hookform/resolvers/zod"
import { appChange, AppChangeParamSchema, appDelete, appList, appRequestList, type AppChangeParamType, type AppListItemType } from "@shared/apis/user/app"
import { ConfirmDialog } from "@shared/components/custom/dialog/confirm-dialog"
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error"
import { CenteredLoading } from "@shared/components/custom/page-placeholder/centered-loading"
import { Alert, AlertDescription, AlertTitle } from "@shared/components/ui/alert"
import { Button } from "@shared/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@shared/components/ui/card"
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from "@shared/components/ui/form"
import { Input } from "@shared/components/ui/input"
import { useToast } from "@shared/contexts/toast-context"
import { cn, formatServerError, formatTime, TIME_STYLE } from "@shared/lib/utils"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { Link, useNavigate } from "@tanstack/react-router"
import { AlertCircle, ArrowRight, KeyRound } from "lucide-react"
import React from "react"
import { useForm } from "react-hook-form"
import { settingModuleConfig } from "./nav-info"
import { AppSettingSecretDrawer } from "./setting-secret-drawer"

interface AppSettingContentProps {
  appData: AppListItemType
  isDisabled: boolean
}

function AppSettingContent({
  appData,
  isDisabled
}: AppSettingContentProps) {
  const { appId } = Route.useParams()
  const toast = useToast()
  const navigate = useNavigate()
  const queryClient = useQueryClient()

  const form = useForm<AppChangeParamType>({
    resolver: zodResolver(AppChangeParamSchema),
    defaultValues: {
      app_id: Number(appId),
      name: appData.name,
      client_id: appData.client_id
    }
  })

  // 当应用数据更新时同步表单
  React.useEffect(() => {
    form.reset({
      app_id: Number(appId),
      name: appData.name,
      client_id: appData.client_id
    })
  }, [appData, appId, form])

  const changeMutation = useMutation({
    mutationFn: (data: AppChangeParamType) => appChange(data),
    onSuccess: (result) => {
      if (result.status) {
        toast.success("应用信息更新成功")
        // 刷新应用详情和待审核请求列表
        queryClient.invalidateQueries({ queryKey: appQueryKey(appId) })
        queryClient.invalidateQueries({ queryKey: ['app-pending-request', appId] })
      } else {
        toast.error(formatServerError(result))
      }
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    }
  })

  const deleteMutation = useMutation({
    mutationFn: (data: { app_id: number }) => appDelete(data),
    onSuccess: (result) => {
      if (result.status) {
        toast.success("应用删除成功")
        navigate({ to: '/user/app' })
      } else {
        toast.error(formatServerError(result))
      }
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    }
  })

  const onSubmit = (data: AppChangeParamType) => {
    changeMutation.mutate(data)
  }

  const handleDelete = async () => {
    await deleteMutation.mutateAsync({ app_id: Number(appId) })
  }

  return (
    <div className={cn("space-y-6")}>
      <Card>
        <CardHeader>
          <CardTitle>应用设置</CardTitle>
          <CardDescription>修改应用的基本信息</CardDescription>
        </CardHeader>
        <CardContent>
          <Form {...form}>
            <form onSubmit={form.handleSubmit(onSubmit)} className={cn("space-y-4")}>
              <FormField
                control={form.control}
                name="name"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>应用名称</FormLabel>
                    <FormControl>
                      <Input
                        placeholder="请输入应用名称"
                        disabled={isDisabled}
                        {...field}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={form.control}
                name="client_id"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>应用标识 (Client ID)</FormLabel>
                    <FormControl>
                      <Input
                        placeholder="请输入应用标识"
                        disabled={isDisabled}
                        {...field}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <LoadingButton
                type="submit"
                loading={changeMutation.isPending}
                disabled={isDisabled}
              >
                保存更改
              </LoadingButton>
            </form>
          </Form>
        </CardContent>
      </Card>

      {/* 仅在允许删除时显示删除卡片 */}
      {appData.status !== 1 && !(appData.status === 2 && isDisabled) && (
        <Card className={cn("border-destructive")}>
          <CardHeader>
            <CardTitle className={cn("text-destructive")}>温馨提示</CardTitle>
            <CardDescription>删除此应用将无法恢复</CardDescription>
          </CardHeader>
          <CardContent>
            <ConfirmDialog
              title="确认删除应用"
              description={
                <div>
                  <p>您确定要删除应用 <strong>{appData?.name}</strong> 吗？</p>
                  <p className={cn("mt-2 text-destructive")}>此操作无法撤销，所有关联数据都将被永久删除。</p>
                </div>
              }
              onConfirm={handleDelete}
            >
              <Button
                variant="outline"
                className={cn("border-destructive text-destructive hover:bg-destructive/10 hover:border-destructive/80 hover:text-destructive")}
                disabled={deleteMutation.isPending}
              >
                删除应用
              </Button>
            </ConfirmDialog>
          </CardContent>
        </Card>
      )}
    </div>
  )
}

export function AppSettingPage() {
  // docs\api\user\app\change.md
  // docs\api\user\app\delete.md
  // docs\api\user\app\app_secret_view.md
  // docs\api\user\app\app_secret_add.md
  // docs\api\user\app\app_secret_change.md
  // docs\api\user\app\app_secret_del.md
  const { appId } = Route.useParams()
  const navigate = useNavigate()
  const searchParams = Route.useSearch()
  const [secretDrawerOpen, setSecretDrawerOpen] = React.useState(false)

  // Auto-open drawer if URL has secret=1
  React.useEffect(() => {
    if (searchParams.secret === 1) {
      setSecretDrawerOpen(true)
    }
  }, [searchParams.secret])

  // Update URL when drawer opens/closes
  const handleSecretDrawerChange = (open: boolean) => {
    setSecretDrawerOpen(open)
    navigate({
      to: '/user/app/$appId/setting',
      params: { appId },
      search: open ? { secret: 1 } : {},
      replace: true
    })
  }

  // 获取应用详情
  const {
    data: appData,
    isLoading: isLoadingApp,
    isError: isAppError,
    error: appError
  } = useQuery({
    queryKey: appQueryKey(appId),
    queryFn: async () => {
      const result = await appList({
        app_id: Number(appId),
        page: { page: 1, limit: 1 },
        count_num: false
      })
      if (!result.status) {
        throw result
      }
      if (result.response?.data && result.response.data.length > 0) {
        return result.response.data[0]
      }
      return null
    },
    enabled: !!appId
  })

  // 检查是否有待审核的应用设置变更申请 (request_type: 2 = 应用信息变更申请, status: 1 = 待审核)
  const {
    data: pendingRequest,
    isLoading: isLoadingRequest,
    isError: isRequestError,
    error: requestError
  } = useQuery({
    queryKey: ['app-pending-request', appId],
    queryFn: async () => {
      const result = await appRequestList({
        app_id: Number(appId),
        request_type: 2,
        status: 1,
        page: { page: 1, limit: 1 },
        count_num: false
      })
      if (result.status && result.response?.data && result.response.data.length > 0) {
        return result.response.data[0]
      }
      return null
    },
    enabled: !!appId
  })

  // 计算是否禁用操作
  const isDisabled = React.useMemo(() => {
    if (!appData) return true
    // status=1: 应用待审核，禁用所有操作
    if (appData.status === 1) return true
    // status=3: 应用已禁用，禁用所有操作（除删除外）
    if (appData.status === 3) return true
    // status=2 且有待审核的变更请求，禁用所有操作
    if (appData.status === 2 && pendingRequest) return true
    return false
  }, [appData, pendingRequest])



  if (isLoadingApp || isLoadingRequest) {
    return <CenteredLoading variant="card" className={cn('md:m-6')} />;
  }

  if (isAppError || isRequestError) {
    return (
      <CenteredError
        variant="card"
        error={appError || requestError}
        className={cn('md:m-6')}
      />
    );
  }

  if (!appData) {
    return (
      <CenteredError
        variant="card"
        error={new Error('应用不存在')}
        className={cn('md:m-6')}
      />
    );
  }

  return (
    <AppDetailNavContainer
      {...settingModuleConfig}
      actions={
        <Button variant="outline" size="sm" onClick={() => handleSecretDrawerChange(true)}>
          <KeyRound className={cn("mr-2 h-4 w-4")} />
          密钥管理
        </Button>
      }
    >
      <div className={cn("space-y-6")}>
        {/* 应用状态提示 */}
        <AppStatusAlert appData={appData} />

        {/* 显示待审核变更申请提示 */}
        {appData.status === 2 && pendingRequest && (
          <Alert className={cn("border-l-4")}>
            <AlertCircle className={cn("h-4 w-4")} />
            <AlertTitle className={cn("text-sm font-medium")}>已有变更申请待审核</AlertTitle>
            <AlertDescription className={cn("text-xs")}>
              <div className={cn("space-y-2 mt-1")}>
                <div>
                  您在 {formatTime(pendingRequest.request_time, TIME_STYLE.ABSOLUTE_TEXT)} 提交的应用设置变更申请正在审核中。
                </div>
                {pendingRequest.change_data && (
                  <div className={cn("space-y-1")}>
                    <div className={cn("font-medium")}>上次请求的变更信息：</div>
                    <div className={cn("ml-3 space-y-0.5 text-muted-foreground")}>
                      {pendingRequest.change_data.name && (
                        <div className={cn("flex items-baseline gap-1.5")}>
                          <span className={cn("text-[10px]")}>•</span>
                          <span>应用名称:</span>
                          <code className={cn("text-xs px-1.5 py-0.5 rounded bg-muted")}>{pendingRequest.change_data.name}</code>
                        </div>
                      )}
                      {pendingRequest.change_data.client_id && (
                        <div className={cn("flex items-baseline gap-1.5")}>
                          <span className={cn("text-[10px]")}>•</span>
                          <span>应用标识:</span>
                          <code className={cn("text-xs px-1.5 py-0.5 rounded bg-muted")}>{pendingRequest.change_data.client_id}</code>
                        </div>
                      )}
                    </div>
                  </div>
                )}
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

        <AppSettingContent
          appData={appData}
          isDisabled={isDisabled}
        />
      </div>

      <AppSettingSecretDrawer
        appId={String(appId)}
        open={secretDrawerOpen}
        onOpenChange={handleSecretDrawerChange}
      />
    </AppDetailNavContainer>
  )
}
