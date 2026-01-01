import { AppSubAppListItemType, appSubSecretView, AppSubSecretViewResType } from '@shared/apis/user/app'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { CenteredLoading } from '@shared/components/custom/page-placeholder/centered-loading'
import { MaskedText } from '@shared/components/custom/text/masked-text'
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
import { useToast } from '@shared/contexts/toast-context'
import { cn, formatTime, TIME_STYLE } from '@shared/lib/utils'
import { createCopyWithToast } from '@shared/lib/utils/copy-utils'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { Bell, Copy, Key, Shield } from 'lucide-react'
import React from 'react'

interface SubAppSecretDrawerProps {
    subApp: AppSubAppListItemType
    children: React.ReactNode
}

export function SubAppSecretDrawer({
    subApp,
    children
}: SubAppSecretDrawerProps) {
    const [open, setOpen] = React.useState(false)
    const { success: showSuccess, error: showError } = useToast()
    const queryClient = useQueryClient()

    // 创建复制函数
    const copyToClipboard = React.useMemo(
        () => createCopyWithToast(showSuccess, showError),
        [showSuccess, showError]
    )

    // 获取子应用密钥数据
    const { data: secretData, isLoading, isError, error, refetch } = useQuery({
        queryKey: ['appSubSecretView', subApp.id],
        queryFn: async ({ signal }) => {
            const result = await appSubSecretView(
                {
                    app_id: subApp.id,
                    app_secret: true,
                    notify_secret: true,
                    oauth_secret: true,
                },
                { signal }
            )
            return result
        },
        enabled: open, // 只有在打开抽屉时才加载数据
    })

    // 提取密钥数据
    const secretInfo = React.useMemo<AppSubSecretViewResType | null>(() => {
        if (!secretData?.response) return null
        return secretData.response
    }, [secretData])

    // 处理复制操作
    const handleCopy = React.useCallback((text: string, label: string) => {
        copyToClipboard(text, `${label} 已复制到剪贴板`)
    }, [copyToClipboard])

    // 刷新数据
    const refreshData = React.useCallback(() => {
        queryClient.refetchQueries({ queryKey: ['appSubSecretView', subApp.id] })
    }, [queryClient, subApp.id])

    return (
        <Drawer open={open} onOpenChange={setOpen}>
            <DrawerTrigger asChild>
                {children}
            </DrawerTrigger>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle className={cn("flex items-center gap-2")}>
                        <Key className={cn("h-5 w-5")} />
                        应用密钥信息
                    </DrawerTitle>
                    <DrawerDescription>
                        查看子应用 <span className={cn("font-medium")}>{subApp.name}</span> 的密钥配置
                    </DrawerDescription>
                </DrawerHeader>

                <div className={cn("mt-6")}>
                    {/* 加载状态 */}
                    {isLoading && (
                        <CenteredLoading variant="content" iconSize="md" />
                    )}

                    {/* 错误状态 */}
                    {isError && (
                        <CenteredError
                            variant="content"
                            error={error}
                            onReset={refetch}
                        />
                    )}

                    {/* 密钥内容 */}
                    {!isLoading && !isError && secretInfo && (
                        <div className={cn("space-y-6")}>
                            {/* 应用密钥 (App Secret) */}
                            <div className={cn("space-y-3")}>
                                <div className={cn("flex items-center justify-between border-b pb-2")}>
                                    <h3 className={cn("text-lg font-semibold flex items-center gap-2")}>
                                        <Shield className={cn("h-4 w-4")} />
                                        应用密钥 (App Secret)
                                    </h3>
                                    {secretInfo.app_secret && secretInfo.app_secret.length > 0 && (
                                        <Badge variant="outline" className={cn("text-xs")}>
                                            {secretInfo.app_secret.length} 个密钥
                                        </Badge>
                                    )}
                                </div>
                                {secretInfo.app_secret && secretInfo.app_secret.length > 0 ? (
                                    <div className={cn("space-y-3")}>
                                        {secretInfo.app_secret.map((item, index) => (
                                            <div
                                                key={index}
                                                className={cn(
                                                    "p-4 rounded-lg border bg-muted/30 space-y-2"
                                                )}
                                            >
                                                <div className={cn("flex items-center justify-between")}>
                                                    <span className={cn("text-sm font-medium text-muted-foreground")}>
                                                        密钥 #{index + 1}
                                                    </span>
                                                    <Button
                                                        variant="ghost"
                                                        size="sm"
                                                        onClick={() => handleCopy(item.secret_data, '应用密钥')}
                                                        className={cn("h-7 px-2")}
                                                    >
                                                        <Copy className={cn("h-3 w-3 mr-1")} />
                                                        复制
                                                    </Button>
                                                </div>
                                                <div className={cn("font-mono text-sm break-all bg-background p-2 rounded")}>
                                                    <MaskedText
                                                        text={item.secret_data}
                                                        type="secret"
                                                        clickable={true}
                                                        onRevealedClick={() => handleCopy(item.secret_data, '应用密钥')}
                                                    />
                                                </div>
                                                <div className={cn("text-xs text-muted-foreground")}>
                                                    过期时间: {formatTime(item.time_out, TIME_STYLE.ABSOLUTE_TEXT)}
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                ) : (
                                    <div className={cn("text-center py-8 text-muted-foreground")}>
                                        暂无应用密钥
                                    </div>
                                )}
                            </div>

                            {/* 回调通知密钥 (Notify Secret) */}
                            <div className={cn("space-y-3")}>
                                <div className={cn("flex items-center justify-between border-b pb-2")}>
                                    <h3 className={cn("text-lg font-semibold flex items-center gap-2")}>
                                        <Bell className={cn("h-4 w-4")} />
                                        回调通知密钥 (Notify Secret)
                                    </h3>
                                </div>
                                {secretInfo.notify_secret ? (
                                    <div className={cn("p-4 rounded-lg border bg-muted/30 space-y-3")}>
                                        <div className={cn("flex items-center justify-between")}>
                                            <span className={cn("text-sm font-medium text-muted-foreground")}>
                                                通知密钥
                                            </span>
                                            <Button
                                                variant="ghost"
                                                size="sm"
                                                onClick={() => handleCopy(secretInfo.notify_secret!.secret, '回调通知密钥')}
                                                className={cn("h-7 px-2")}
                                            >
                                                <Copy className={cn("h-3 w-3 mr-1")} />
                                                复制
                                            </Button>
                                        </div>
                                        <div className={cn("font-mono text-sm break-all bg-background p-2 rounded")}>
                                            <MaskedText
                                                text={secretInfo.notify_secret.secret}
                                                type="secret"
                                                clickable={true}
                                                onRevealedClick={() => handleCopy(secretInfo.notify_secret!.secret, '回调通知密钥')}
                                            />
                                        </div>
                                        <div className={cn("text-xs text-muted-foreground")}>
                                            过期时间: {formatTime(secretInfo.notify_secret.timeout, TIME_STYLE.ABSOLUTE_TEXT)}
                                        </div>
                                    </div>
                                ) : (
                                    <div className={cn("text-center py-8 text-muted-foreground")}>
                                        暂无回调通知密钥
                                    </div>
                                )}
                            </div>

                            {/* OAuth 密钥 (OAuth Secret) */}
                            <div className={cn("space-y-3")}>
                                <div className={cn("flex items-center justify-between border-b pb-2")}>
                                    <h3 className={cn("text-lg font-semibold flex items-center gap-2")}>
                                        <Key className={cn("h-4 w-4")} />
                                        OAuth 密钥 (OAuth Secret)
                                    </h3>
                                    {secretInfo.oauth_secret && secretInfo.oauth_secret.length > 0 && (
                                        <Badge variant="outline" className={cn("text-xs")}>
                                            {secretInfo.oauth_secret.length} 个密钥
                                        </Badge>
                                    )}
                                </div>
                                {secretInfo.oauth_secret && secretInfo.oauth_secret.length > 0 ? (
                                    <div className={cn("space-y-3")}>
                                        {secretInfo.oauth_secret.map((item, index) => (
                                            <div
                                                key={index}
                                                className={cn(
                                                    "p-4 rounded-lg border bg-muted/30 space-y-2"
                                                )}
                                            >
                                                <div className={cn("flex items-center justify-between")}>
                                                    <span className={cn("text-sm font-medium text-muted-foreground")}>
                                                        OAuth 密钥 #{index + 1}
                                                    </span>
                                                    <Button
                                                        variant="ghost"
                                                        size="sm"
                                                        onClick={() => handleCopy(item.secret_data, 'OAuth 密钥')}
                                                        className={cn("h-7 px-2")}
                                                    >
                                                        <Copy className={cn("h-3 w-3 mr-1")} />
                                                        复制
                                                    </Button>
                                                </div>
                                                <div className={cn("font-mono text-sm break-all bg-background p-2 rounded")}>
                                                    <MaskedText
                                                        text={item.secret_data}
                                                        type="secret"
                                                        clickable={true}
                                                        onRevealedClick={() => handleCopy(item.secret_data, 'OAuth 密钥')}
                                                    />
                                                </div>
                                                <div className={cn("text-xs text-muted-foreground")}>
                                                    过期时间: {formatTime(item.time_out, TIME_STYLE.ABSOLUTE_TEXT)}
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                ) : (
                                    <div className={cn("text-center py-8 text-muted-foreground")}>
                                        暂无 OAuth 密钥
                                    </div>
                                )}
                            </div>

                            {/* 底部提示 */}
                            <div className={cn("pt-4 border-t")}>
                                <div className={cn("bg-blue-50 dark:bg-blue-950/30 p-3 rounded-lg text-xs text-blue-700 dark:text-blue-300 space-y-1")}>
                                    <p>💡 <strong>使用提示：</strong></p>
                                    <ul className={cn("list-disc list-inside space-y-1 ml-2")}>
                                        <li>双击密钥可以显示/隐藏完整内容</li>
                                        <li>单击"复制"按钮可以快速复制密钥</li>
                                        <li>显示完整密钥后单击密钥也可以复制</li>
                                    </ul>
                                </div>
                            </div>

                            {/* 底部操作按钮 */}
                            <div className={cn("flex justify-end gap-2 pt-4 border-t")}>
                                <Button onClick={refreshData} variant="outline">
                                    刷新
                                </Button>
                                <Button onClick={() => setOpen(false)} variant="default">
                                    关闭
                                </Button>
                            </div>
                        </div>
                    )}
                </div>
            </DrawerContent>
        </Drawer>
    )
}
