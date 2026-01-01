import { appList, AppListParamType } from '@shared/apis/admin/app'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { Badge } from '@shared/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@shared/components/ui/card'
import { Drawer, DrawerContent, DrawerHeader, DrawerTitle } from '@apps/main/components/local/drawer'
import { cn, formatTime, TIME_STYLE } from '@shared/lib/utils'
import { createStatusMapper } from '@apps/main/lib/status-utils'
import { type DictList } from '@shared/types/apis-dict'
import { useQuery } from '@tanstack/react-query'
import { Globe, Key, Settings, Shield, Users } from 'lucide-react'
import { CenteredLoading } from '@shared/components/custom/page-placeholder/centered-loading'
import { useMemo } from 'react'

interface AppDetailDrawerProps {
  /** 应用ID */
  appId: number | null
  /** 是否打开抽屉 */
  open: boolean
  /** 关闭抽屉的回调 */
  onClose: () => void
  /** 应用状态字典 */
  appStatusDict: DictList
}

/**
 * 应用详情抽屉组件
 * @description 显示应用的详细信息，包括基础信息、OAuth配置、子应用等
 */
export function AppDetailDrawer({ appId, open, onClose, appStatusDict }: AppDetailDrawerProps) {
  // 获取应用详情数据
  const { data: appDetailData, isLoading, isError, error, refetch } = useQuery({
    queryKey: ['appDetail', appId],
    queryFn: async ({ signal }) => {
      if (!appId) return null
      const param: AppListParamType = {
        app_id: appId,
        detail_data: true,
        count_num: false,
        page: { page: 1, limit: 1 }
      }
      const result = await appList(param, { signal })
      if (!result.status) {
        // 如果API返回失败，抛出完整的错误信息
        throw result
      }
      return result.response?.data?.[0] || null
    },
    enabled: open && !!appId,
  })

  const appData = appDetailData

  // 状态样式映射 - 应用状态: 1=审核中, 2=正常, 3=被禁用
  const statusMapper = useMemo(
    () =>
      createStatusMapper(
        { 1: 'warning', 2: 'success', 3: 'neutral' },
        (status: number) => {
          return appStatusDict.getLabel(String(status)) || String(status)
        },
      ),
    [appStatusDict],
  )

  // 渲染OAuth客户端信息
  const renderOAuthClientInfo = () => {
    if (!appData?.oauth_client) return null

    return (
      <Card>
        <CardHeader className={cn("pb-3")}>
          <CardTitle className={cn("flex items-center gap-2 text-base")}>
            <Key className={cn("h-4 w-4")} />
            OAuth客户端配置
          </CardTitle>
        </CardHeader>
        <CardContent className={cn("space-y-3")}>
          <div className="flex items-center gap-2">
            <Badge variant="secondary" className={cn("bg-blue-500 text-white")}>
              已开通
            </Badge>
          </div>

          {appData.oauth_client_data && (
            <div className="space-y-2">
              <div>
                <span className="text-sm font-medium text-muted-foreground">回调域名：</span>
                <span className="text-sm">{appData.oauth_client_data.callback_domain}</span>
              </div>

              {appData.oauth_client_data.scope_data && (
                <div>
                  <span className="text-sm font-medium text-muted-foreground">授权范围：</span>
                  <div className="flex flex-wrap gap-1 mt-1">
                    {appData.oauth_client_data.scope_data.split(',').filter(Boolean).map((scope, index) => (
                      <Badge key={index} variant="outline" className={cn("text-xs")}>
                        {scope.trim()}
                      </Badge>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}
        </CardContent>
      </Card>
    )
  }

  // 渲染OAuth服务器信息
  const renderOAuthServerInfo = () => {
    return (
      <Card>
        <CardHeader className={cn("pb-3")}>
          <CardTitle className={cn("flex items-center gap-2 text-base")}>
            <Shield className={cn("h-4 w-4")} />
            OAuth服务器
          </CardTitle>
        </CardHeader>
        <CardContent className={cn("space-y-3")}>
          <div className="flex items-center gap-2">
            <Badge
              variant="secondary"
              className={cn(appData?.oauth_server ? "bg-purple-500 text-white" : "bg-gray-500 text-white")}
            >
              {appData?.oauth_server ? '已开通' : '未开通'}
            </Badge>
          </div>

          {appData?.oauth_server && appData?.oauth_server_scope_data && appData.oauth_server_scope_data.length > 0 && (
            <div>
              <span className="text-sm font-medium text-muted-foreground">服务器授权范围：</span>
              <div className="flex flex-wrap gap-1 mt-1">
                {appData.oauth_server_scope_data.map((scope, index) => (
                  <Badge key={index} variant="outline" className={cn("text-xs")}>
                    {scope.scope_name}
                  </Badge>
                ))}
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    )
  }

  // 渲染子应用信息
  const renderSubAppInfo = () => {
    return (
      <Card>
        <CardHeader className={cn("pb-3")}>
          <CardTitle className={cn("flex items-center gap-2 text-base")}>
            <Users className={cn("h-4 w-4")} />
            子应用功能
          </CardTitle>
        </CardHeader>
        <CardContent className={cn("space-y-3")}>
          <div className="flex items-center gap-2">
            <Badge
              variant="secondary"
              className={cn(appData?.sup_app ? "bg-indigo-500 text-white" : "bg-gray-500 text-white")}
            >
              {appData?.sup_app ? '已开通' : '未开通'}
            </Badge>
          </div>

          {appData?.sup_app && appData?.sub_app_count && (
            <div className="grid grid-cols-3 gap-2">
              <div className="text-center p-2 rounded-lg bg-green-50 border">
                <div className="text-lg font-semibold text-green-600">{appData.sub_app_count.enable}</div>
                <div className="text-xs text-green-600">启用</div>
              </div>
              <div className="text-center p-2 rounded-lg bg-yellow-50 border">
                <div className="text-lg font-semibold text-yellow-600">{appData.sub_app_count.init}</div>
                <div className="text-xs text-yellow-600">待审核</div>
              </div>
              <div className="text-center p-2 rounded-lg bg-red-50 border">
                <div className="text-lg font-semibold text-red-600">{appData.sub_app_count.disable}</div>
                <div className="text-xs text-red-600">禁用</div>
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    )
  }

  // 渲染外部功能
  const renderExternalFeatures = () => {
    if (!appData?.exter_feature || appData.exter_feature.length === 0) return null

    return (
      <Card>
        <CardHeader className={cn("pb-3")}>
          <CardTitle className={cn("flex items-center gap-2 text-base")}>
            <Settings className={cn("h-4 w-4")} />
            已开通功能
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-1">
            {appData.exter_feature.map((feature, index) => (
              <Badge key={index} variant="secondary" className={cn("bg-orange-500 text-white")}>
                {feature.toString()}
              </Badge>
            ))}
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <Drawer open={open} onOpenChange={(open) => !open && onClose()}>
      <DrawerContent>
        <DrawerHeader className="pt-6">
          <DrawerTitle>应用详情</DrawerTitle>
        </DrawerHeader>

        <div className="mt-6 space-y-6 pb-6">
          {isLoading && (
            <CenteredLoading variant="content"  />
          )}

          {isError && (
            <CenteredError
              variant="content"
              error={error}
              onReset={() => refetch()}
            />
          )}

          {appData && (
            <>
              {/* 基础信息 */}
              <Card>
                <CardHeader className={cn("pb-3")}>
                  <CardTitle className={cn("flex items-center gap-2 text-base")}>
                    <Globe className={cn("h-4 w-4")} />
                    基础信息
                  </CardTitle>
                </CardHeader>
                <CardContent className={cn("space-y-3")}>
                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <div>
                      <span className="text-sm font-medium text-muted-foreground">应用ID：</span>
                      <span className="text-sm font-mono">{appData.id}</span>
                    </div>
                    <div>
                      <span className="text-sm font-medium text-muted-foreground">应用名称：</span>
                      <span className="text-sm font-medium">{appData.name}</span>
                    </div>
                  </div>

                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <div>
                      <span className="text-sm font-medium text-muted-foreground">状态：</span>
                      <Badge variant="secondary" className={cn("ml-2", statusMapper.getClass(appData.status))}>
                        {statusMapper.getText(appData.status)}
                      </Badge>
                    </div>
                    <div>
                      <span className="text-sm font-medium text-muted-foreground">客户端ID：</span>
                      <span className="text-sm font-mono">{appData.client_id || '未设置'}</span>
                    </div>
                  </div>

                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <div>
                      <span className="text-sm font-medium text-muted-foreground">外部登录：</span>
                      <Badge
                        variant="secondary"
                        className={cn("ml-2", appData.exter_login ? "bg-blue-500 text-white" : "bg-gray-500 text-white")}
                      >
                        {appData.exter_login ? '支持' : '不支持'}
                      </Badge>
                    </div>
                    <div>
                      <span className="text-sm font-medium text-muted-foreground">更新时间：</span>
                      <span className="text-sm">{formatTime(appData.change_time, TIME_STYLE.ABSOLUTE_TEXT) || '未知'}</span>
                    </div>
                  </div>

                  {/* {appData.req_count && appData.req_count > 0 && (
                    <div className="flex items-center gap-2 p-2 bg-yellow-50 rounded-lg border border-yellow-200">
                      <AlertCircle className={cn("h-4 w-4 text-yellow-600")} />
                      <span className="text-sm text-yellow-700">
                        待审核数量：{appData.req_count}
                      </span>
                    </div>
                  )} */}
                </CardContent>
              </Card>

              {/* 用户信息 */}
              {appData.user_data && (
                <Card>
                  <CardHeader className={cn("pb-3")}>
                    <CardTitle className={cn("flex items-center gap-2 text-base")}>
                      <Users className={cn("h-4 w-4")} />
                      所属用户
                    </CardTitle>
                  </CardHeader>
                  <CardContent className={cn("space-y-3")}>
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                      <div>
                        <span className="text-sm font-medium text-muted-foreground">用户ID：</span>
                        <span className="text-sm font-mono">{appData.user_data.id}</span>
                      </div>
                      <div>
                        <span className="text-sm font-medium text-muted-foreground">用户账号：</span>
                        <span className="text-sm">{appData.user_data.user_account}</span>
                      </div>
                    </div>
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                      <div>
                        <span className="text-sm font-medium text-muted-foreground">用户昵称：</span>
                        <span className="text-sm">{appData.user_data.user_nickname}</span>
                      </div>
                      <div>
                        <span className="text-sm font-medium text-muted-foreground">所属应用ID：</span>
                        <span className="text-sm font-mono">{appData.user_data.app_id}</span>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              )}

              {/* 外部功能 */}
              {renderExternalFeatures()}

              {/* OAuth客户端 */}
              {renderOAuthClientInfo()}

              {/* OAuth服务器 */}
              {renderOAuthServerInfo()}

              {/* 子应用功能 */}
              {renderSubAppInfo()}
            </>
          )}
        </div>
      </DrawerContent>
    </Drawer>
  )
}
