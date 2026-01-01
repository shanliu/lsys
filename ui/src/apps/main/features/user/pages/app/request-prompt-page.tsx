
import { Button } from '@shared/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@shared/components/ui/card'
import { cn } from '@shared/lib/utils'
import { Route } from "@apps/main/routes/_main/user/app/$appId/request-prompt"
import { useNavigate } from '@tanstack/react-router'
import { AlertCircle, CheckCircle, XCircle } from 'lucide-react'
import { getServiceTypeConfig } from './request-prompt-config'
/**
 * 应用服务未授权提示页面
 * 根据 URL 中的 type 参数显示不同的服务开通引导
 */
export default function AppRequestPromptPage() {

  const { appId } = Route.useParams()
  const search = Route.useSearch()
  const navigate = useNavigate()

  // 从 URL 参数获取服务类型
  const serviceType = search.type

  // 获取服务配置
  const config = getServiceTypeConfig(serviceType)

  // 处理导航到服务开通页面
  const handleNavigateToService = () => {
    if (!config || !appId) return

    // 替换路由参数中的 $appId
    const route = config.serviceRoute.replace('$appId', String(appId))
    navigate({ to: route })
  }

  // 如果没有配置，显示通用错误
  if (!config) {
    return (
      <div className="container mx-auto py-8 px-4">
        <Card className={cn("max-w-2xl mx-auto")}>
          <CardHeader>
            <div className="flex items-center gap-3">
              <XCircle className="h-6 w-6 text-destructive" />
              <div>
                <CardTitle>参数错误</CardTitle>
                <CardDescription>
                  {serviceType
                    ? `未知的服务类型: ${serviceType}`
                    : '缺少服务类型参数'}
                </CardDescription>
              </div>
            </div>
          </CardHeader>
          <CardContent>
            <Button
              variant="outline"
              onClick={() => navigate({ to: `/user/app/${appId}` })}
            >
              返回应用详情
            </Button>
          </CardContent>
        </Card>
      </div>
    )
  }

  const Icon = config.icon

  return (
    <div className="container mx-auto py-8 px-4">
      <Card className={cn("max-w-3xl mx-auto")}>
        <CardHeader>
          <div className="flex items-start gap-4">
            <div className="p-3 bg-primary/10 rounded-lg">
              <Icon className="h-8 w-8 text-primary" />
            </div>
            <div className="flex-1">
              <CardTitle className={cn("text-2xl")}>{config.title}</CardTitle>
              <CardDescription className={cn("mt-2 text-base")}>
                {config.description}
              </CardDescription>
            </div>
          </div>
        </CardHeader>

        <CardContent className={cn("space-y-6")}>
          {/* 功能列表 */}
          <div>
            <h4 className="text-sm font-medium mb-3">开通后即可使用以下功能：</h4>
            <div className="space-y-3">
              {config.details.map((detail, index) => (
                <div key={index} className="flex items-start gap-3">
                  <CheckCircle className="h-5 w-5 text-green-500 mt-0.5 flex-shrink-0" />
                  <span className="text-sm text-muted-foreground">{detail}</span>
                </div>
              ))}
            </div>
          </div>

          {/* 提示信息 */}
          <div className="flex items-start gap-3 p-4 bg-blue-50 dark:bg-blue-950/20 rounded-lg border border-blue-200 dark:border-blue-900">
            <AlertCircle className="h-5 w-5 text-blue-500 mt-0.5 flex-shrink-0" />
            <p className="text-sm text-blue-700 dark:text-blue-300">
              开通服务后，您需要根据实际需求完成相应的配置才能正常使用
            </p>
          </div>

          {/* 操作按钮 */}
          <div className="flex justify-end gap-3">
            <Button
              variant="outline"
              onClick={() => navigate({ to: `/user/app/${appId}` })}
            >
              返回应用详情
            </Button>
            <Button onClick={handleNavigateToService}>
              {config.buttonText}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

