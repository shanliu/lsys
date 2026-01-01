import { AppSubAppListItemType } from '@shared/apis/user/app'
import { Badge } from '@shared/components/ui/badge'
import { Button } from '@shared/components/ui/button'
import {
    Drawer,
    DrawerContent,
    DrawerDescription,
    DrawerHeader,
    DrawerTitle,
    DrawerTrigger,
} from '@apps/main/components/local/drawer'
import { useAuthData } from '@apps/main/hooks/use-auth-data'
import { cn, formatTime, TIME_STYLE } from '@shared/lib/utils'
import type { StatusMapper } from '@apps/main/lib/status-utils'
import { useNavigate } from '@tanstack/react-router'
import { Settings } from 'lucide-react'
import React from 'react'

interface SubAppDetailDrawerProps {
    subApp: AppSubAppListItemType
    appStatusMapper: StatusMapper<number>
    children: React.ReactNode
}

export function SubAppDetailDrawer({
    subApp,
    appStatusMapper,
    children
}: SubAppDetailDrawerProps) {
    const navigate = useNavigate()
    const [open, setOpen] = React.useState(false)

    // 获取当前登录用户信息
    const authData = useAuthData()
    const currentUserId = authData.userId

    const canManage = subApp.user_id === currentUserId

    return (
        <Drawer open={open} onOpenChange={setOpen}>
            <DrawerTrigger asChild>
                {children}
            </DrawerTrigger>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>子应用详情</DrawerTitle>
                    <DrawerDescription>
                        查看子应用的详细信息和配置
                    </DrawerDescription>
                </DrawerHeader>

                <div className={cn("mt-6 space-y-6")}>
                    {/* 基本信息 */}
                    <div className={cn("space-y-3")}>
                        <h3 className={cn("text-lg font-semibold border-b pb-2")}>基本信息</h3>
                        <div className={cn("grid grid-cols-1 sm:grid-cols-2 gap-4")}>
                            <div>
                                <span className={cn("text-sm text-muted-foreground")}>应用ID</span>
                                <p className={cn("text-sm font-medium mt-1")}>{subApp.id}</p>
                            </div>
                            <div>
                                <span className={cn("text-sm text-muted-foreground")}>应用名称</span>
                                <p className={cn("text-sm font-medium mt-1")}>{subApp.name}</p>
                            </div>
                            <div className={cn("sm:col-span-2")}>
                                <span className={cn("text-sm text-muted-foreground")}>应用标识</span>
                                <p className={cn("text-sm font-medium font-mono mt-1 break-all")}>{subApp.client_id}</p>
                            </div>
                            <div>
                                <span className={cn("text-sm text-muted-foreground block mb-1")}>状态</span>
                                <Badge className={cn(appStatusMapper.getClass(subApp.status))}>
                                    {appStatusMapper.getText(subApp.status)}
                                </Badge>
                            </div>
                            <div>
                                <span className={cn("text-sm text-muted-foreground")}>修改时间</span>
                                <p className={cn("text-sm font-medium mt-1")}>{formatTime(subApp.change_time, TIME_STYLE.ABSOLUTE_TEXT)}</p>
                            </div>
                        </div>
                    </div>

                    {/* 用户信息 */}
                    {subApp.user_data && (
                        <div className={cn("space-y-3")}>
                            <h3 className={cn("text-lg font-semibold border-b pb-2")}>用户信息</h3>
                            <div className={cn("grid grid-cols-1 sm:grid-cols-2 gap-4")}>
                                <div>
                                    <span className={cn("text-sm text-muted-foreground")}>用户ID</span>
                                    <p className={cn("text-sm font-medium mt-1")}>{subApp.user_data.id}</p>
                                </div>
                                <div>
                                    <span className={cn("text-sm text-muted-foreground")}>用户类型</span>
                                    <p className={cn("text-sm font-medium mt-1")}> {subApp.user_data.app_id === 0 ? "系统用户" : "应用用户"}</p>
                                </div>
                                <div >
                                    <span className={cn("text-sm text-muted-foreground")}>用户昵称</span>
                                    <p className={cn("text-sm font-medium mt-1")}>{subApp.user_data.user_nickname}</p>
                                </div>
                                <div >
                                    <span className={cn("text-sm text-muted-foreground")}>用户账号</span>
                                    <p className={cn("text-sm font-medium mt-1")}>{subApp.user_data.user_account}</p>
                                </div>
                            </div>
                        </div>
                    )}

                    {/* OAuth 登录 */}
                    <div className={cn("space-y-3")}>
                        <h3 className={cn("text-lg font-semibold border-b pb-2")}>OAuth 登录</h3>
                        <div className={cn("space-y-3")}>
                            <div>
                                <span className={cn("text-sm text-muted-foreground")}>OAuth 登录</span>
                                <Badge variant={subApp.oauth_client ? 'default' : 'secondary'} className={cn("ml-2")}>
                                    {subApp.oauth_client ? '已启用' : '未启用'}
                                </Badge>
                            </div>
                            {subApp.oauth_client_data && (
                                <>
                                    <div>
                                        <span className={cn("text-sm text-muted-foreground")}>回调域名</span>
                                        <p className={cn("text-sm font-medium font-mono break-all mt-1 bg-muted px-2 py-1 rounded")}>
                                            {subApp.oauth_client_data.callback_domain || '未设置'}
                                        </p>
                                    </div>
                                    <div>
                                        <span className={cn("text-sm text-muted-foreground")}>授权范围</span>
                                        <p className={cn("text-sm font-medium mt-1")}>{subApp.oauth_client_data.scope_data || '未设置'}</p>
                                    </div>
                                </>
                            )}
                        </div>
                    </div>

                    {/* 外部功能 */}
                    <div className={cn("space-y-3")}>
                        <h3 className={cn("text-lg font-semibold border-b pb-2")}>扩展功能</h3>
                        <div>
                            <span className={cn("text-sm text-muted-foreground")}>已启用功能</span>
                            {subApp.exter_feature && subApp.exter_feature.length > 0 ? (
                                <div className={cn("flex flex-wrap gap-2 mt-2")}>
                                    {subApp.exter_feature.map((feature, index) => (
                                        <Badge key={index} variant="outline">{feature}</Badge>
                                    ))}
                                </div>
                            ) : (
                                <p className={cn("text-sm text-muted-foreground mt-1")}>暂无外部功能</p>
                            )}
                        </div>
                    </div>

                    {/* 底部操作按钮 */}
                    <div className={cn("flex justify-end gap-2 pt-4 border-t")}>
                        {canManage && (
                            <Button
                                onClick={() => {
                                    setOpen(false)
                                    navigate({ to: '/user/app/$appId', params: { appId: subApp.id } })
                                }}
                                variant="default"
                            >
                                <Settings className={cn("h-4 w-4 mr-2")} />
                                进入应用管理
                            </Button>
                        )}
                        <Button onClick={() => setOpen(false)} variant="outline">
                            关闭
                        </Button>
                    </div>
                </div>
            </DrawerContent>
        </Drawer>
    )
}
