import { appList, type AppListItemType } from '@shared/apis/user/app'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { CenteredLoading } from '@shared/components/custom/page-placeholder/centered-loading'
import { Alert, AlertDescription, AlertTitle } from '@shared/components/ui/alert'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@shared/components/ui/card'
import { AppDetailNavContainer } from '@apps/main/features/user/components/ui/app-detail-nav'
import { cn, getQueryResponseData } from '@shared/lib/utils'
import { Route } from '@apps/main/routes/_main/user/app/$appId/features-sms/send'
import { useQuery } from '@tanstack/react-query'
import { AlertCircle, MessageSquare } from 'lucide-react'
import { featureSmsModuleConfig } from '../nav-info'
import { AppDetailFeatureSmsSendForm } from './send-form'

export default function AppDetailFeatureSmsSendPage() {
  // user\app_sender_smser\message_send.md

  const { appId } = Route.useParams()

  // 获取应用详情
  const {
    data: appQueryData,
    isLoading: isLoadingApp,
    isError: isAppError,
    error: appError
  } = useQuery({
    queryKey: ['app-sms-send-detail', appId],
    queryFn: ({ signal }) => appList({
      app_id: Number(appId),
      page: { page: 1, limit: 1 },
      attr_inner_feature: true,
      count_num: false
    }, { signal }),
    enabled: !!appId
  })

  const appData = getQueryResponseData<AppListItemType[]>(appQueryData, [])[0] ?? null

  // 加载中状态
  if (isLoadingApp) {
    return (
      <AppDetailNavContainer {...featureSmsModuleConfig}>
        <CenteredLoading variant="card" />
      </AppDetailNavContainer>
    )
  }

  // 加载失败状态
  if (isAppError) {
    return (
      <AppDetailNavContainer {...featureSmsModuleConfig}>
        <CenteredError
          variant="card"
          error={appError}
        />
      </AppDetailNavContainer>
    )
  }

  // 应用不存在
  if (!appData) {
    return (
      <AppDetailNavContainer {...featureSmsModuleConfig}>
        <CenteredError
          variant="card"
          error={new Error('应用不存在')}
        />
      </AppDetailNavContainer>
    )
  }

  // 应用状态检查：status !== 2 表示应用未启用
  if (appData.status !== 2) {
    const statusMessages: Record<number, { title: string; description: string }> = {
      1: { title: '应用待审核', description: '您的应用正在审核中，审核通过后才能使用短信发送功能。' },
      3: { title: '应用已禁用', description: '您的应用已被禁用，无法使用短信发送功能。' },
    }

    const statusInfo = statusMessages[appData.status] ?? {
      title: '应用状态异常',
      description: '应用当前状态无法使用短信发送功能。'
    }

    return (
      <AppDetailNavContainer {...featureSmsModuleConfig}>
        <Card>
          <CardHeader>
            <CardTitle className={cn("flex items-center gap-2")}>
              <MessageSquare className={cn("h-5 w-5")} />
              发送短信
            </CardTitle>
            <CardDescription>发送新短信给指定收件人</CardDescription>
          </CardHeader>
          <CardContent>
            <Alert className={cn("border-l-4")}>
              <AlertCircle className={cn("h-4 w-4")} />
              <AlertTitle className={cn("text-sm font-medium")}>{statusInfo.title}</AlertTitle>
              <AlertDescription className={cn("text-xs")}>
                {statusInfo.description}
              </AlertDescription>
            </Alert>
          </CardContent>
        </Card>
      </AppDetailNavContainer>
    )
  }

  // 应用状态正常，渲染发送表单
  return (
    <AppDetailNavContainer {...featureSmsModuleConfig}>
      <Card>
        <CardHeader>
          <CardTitle className={cn("flex items-center gap-2")}>
            <MessageSquare className={cn("h-5 w-5")} />
            发送短信
          </CardTitle>
          <CardDescription>使用已配置的模板发送短信给指定收件人</CardDescription>
        </CardHeader>
        <CardContent>
          <AppDetailFeatureSmsSendForm appId={Number(appId)} />
        </CardContent>
      </Card>
    </AppDetailNavContainer>
  )
}
