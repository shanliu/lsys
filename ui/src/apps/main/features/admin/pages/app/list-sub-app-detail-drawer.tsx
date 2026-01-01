import { SubAppItemType } from '@shared/apis/admin/app'
import { Badge } from '@shared/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@shared/components/ui/card'
import { Drawer, DrawerContent, DrawerHeader, DrawerTitle } from '@apps/main/components/local/drawer'
import { cn, formatTime, TIME_STYLE } from '@shared/lib/utils'
import { createStatusMapper } from '@apps/main/lib/status-utils'
import { type DictList } from '@shared/types/apis-dict'
import { Globe, Key, Settings, Users } from 'lucide-react'
import { useMemo } from 'react'

interface SubAppDetailDrawerProps {
  /** 子应用数据 */
  subApp: SubAppItemType | null
  /** 是否打开抽屉 */
  open: boolean
  /** 关闭抽屉的回调 */
  onClose: () => void
  /** 应用状态字典 */
  appStatusDict: DictList
}

/**
 * 子应用详情抽屉组件
 * @description 显示子应用的详细信息
 */
export function SubAppDetailDrawer({ subApp, open, onClose, appStatusDict }: SubAppDetailDrawerProps) {
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
    if (!subApp?.oauth_client) return null

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

          {subApp.oauth_client_data && (
            <div className="space-y-2">
              <div>
                <span className="text-sm font-medium text-muted-foreground">回调域名：</span>
                <span className="text-sm">{subApp.oauth_client_data.callback_domain}</span>
              </div>

              {subApp.oauth_client_data.scope_data && (
                <div>
                  <span className="text-sm font-medium text-muted-foreground">授权范围：</span>
                  <div className="flex flex-wrap gap-1 mt-1">
                    {subApp.oauth_client_data.scope_data.split(',').filter(Boolean).map((scope, index) => (
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

  // 渲染外部功能
  const renderExternalFeatures = () => {
    if (!subApp?.exter_feature || subApp.exter_feature.length === 0) return null

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
            {subApp.exter_feature.map((feature, index) => (
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
      <DrawerContent className={cn("sm:max-w-[600px]")}>
        <DrawerHeader>
          <DrawerTitle>子应用详情</DrawerTitle>
        </DrawerHeader>

        <div className="mt-6 space-y-6 overflow-y-auto max-h-[calc(100vh-100px)]">
          {subApp && (
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
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <span className="text-sm font-medium text-muted-foreground">应用ID：</span>
                      <span className="text-sm font-mono">{subApp.id}</span>
                    </div>
                    <div>
                      <span className="text-sm font-medium text-muted-foreground">应用名称：</span>
                      <span className="text-sm font-medium">{subApp.name}</span>
                    </div>
                  </div>

                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <span className="text-sm font-medium text-muted-foreground">状态：</span>
                      <Badge variant="secondary" className={cn("ml-2", statusMapper.getClass(subApp.status))}>
                        {statusMapper.getText(subApp.status)}
                      </Badge>
                    </div>
                    <div>
                      <span className="text-sm font-medium text-muted-foreground">客户端ID：</span>
                      <span className="text-sm font-mono">{subApp.client_id || '未设置'}</span>
                    </div>
                  </div>

                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <span className="text-sm font-medium text-muted-foreground">子应用功能：</span>
                      <Badge
                        variant="secondary"
                        className={cn("ml-2", subApp.sup_app ? "bg-indigo-500 text-white" : "bg-gray-500 text-white")}
                      >
                        {subApp.sup_app ? '已开通' : '未开通'}
                      </Badge>
                    </div>
                    <div>
                      <span className="text-sm font-medium text-muted-foreground">更新时间：</span>
                      <span className="text-sm">{formatTime(subApp.change_time, TIME_STYLE.ABSOLUTE_TEXT) || '未知'}</span>
                    </div>
                  </div>
                </CardContent>
              </Card>

              {/* 用户信息 */}
              {subApp.user_data && (
                <Card>
                  <CardHeader className={cn("pb-3")}>
                    <CardTitle className={cn("flex items-center gap-2 text-base")}>
                      <Users className={cn("h-4 w-4")} />
                      所属用户
                    </CardTitle>
                  </CardHeader>
                  <CardContent className={cn("space-y-3")}>
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <span className="text-sm font-medium text-muted-foreground">用户ID：</span>
                        <span className="text-sm font-mono">{subApp.user_data.id}</span>
                      </div>
                      <div>
                        <span className="text-sm font-medium text-muted-foreground">用户账号：</span>
                        <span className="text-sm">{subApp.user_data.user_account}</span>
                      </div>
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <span className="text-sm font-medium text-muted-foreground">用户昵称：</span>
                        <span className="text-sm">{subApp.user_data.user_nickname}</span>
                      </div>
                      <div>
                        <span className="text-sm font-medium text-muted-foreground">所属应用ID：</span>
                        <span className="text-sm font-mono">{subApp.user_data.app_id}</span>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              )}

              {/* 外部功能 */}
              {renderExternalFeatures()}

              {/* OAuth客户端 */}
              {renderOAuthClientInfo()}
            </>
          )}
        </div>
      </DrawerContent>
    </Drawer>
  )
}
