import { type AppSubRequestItemType } from "@shared/apis/user/app"
import { Badge } from "@shared/components/ui/badge"
import { cn } from "@shared/lib/utils"

interface SubAppRequestDataDisplayProps {
    data: AppSubRequestItemType
    /** 紧凑模式 - 单行显示，超出截断 */
    compact?: boolean
    /** 是否显示标签 */
    showLabel?: boolean
    /** 显示模式：table-展开行 | drawer-抽屉详情 */
    mode?: 'table' | 'drawer'
}

/**
 * 获取请求数据类型标签
 */
function getRequestDataLabel(requestType: number): string {
    switch (requestType) {
        case 1:
            return "申请数据"
        case 2:
            return "变更数据"
        case 6:
        case 7:
            return "OAuth授权范围"
        case 8:
            return "外部功能数据"
        default:
            return "请求数据"
    }
}

/**
 * 子应用请求附带数据显示组件
 * 根据 request_type 显示不同的数据格式
 */
export function SubAppRequestDataDisplay({ data, compact = false, showLabel = false, mode = 'table' }: SubAppRequestDataDisplayProps) {
    const { request_type, change_data, feature_data, oauth_client_data } = data

    const dataLabel = showLabel ? getRequestDataLabel(request_type) : null

    // 渲染内容
    const renderContent = () => {
        // request_type: 1, 2 - 显示 change_data
        if ((request_type === 1 || request_type === 2) && change_data) {
            return (
                <div className={cn(
                    "flex gap-2 text-sm",
                    compact ? "items-center overflow-hidden" : "flex-wrap"
                )}>
                    {change_data.name && (
                        <div className={cn("flex items-center gap-1", compact && "shrink-0")}>
                            <span className="text-muted-foreground">名称:</span>
                            <span className={cn("font-medium", compact && "truncate")}>{change_data.name}</span>
                        </div>
                    )}
                    {change_data.client_id && (
                        <div className={cn("flex items-center gap-1", compact && "min-w-0")}>
                            <span className="text-muted-foreground">标识:</span>
                            <code className={cn(
                                "px-1.5 py-0.5 rounded bg-muted text-xs font-mono",
                                compact && "truncate max-w-[120px]"
                            )}>
                                {change_data.client_id}
                            </code>
                        </div>
                    )}
                </div>
            )
        }

        // request_type: 8 - 显示 feature_data (数组格式)
        if (request_type === 8 && feature_data) {
            if (!Array.isArray(feature_data) || feature_data.length === 0) {
                return (
                    <div className={cn("text-sm text-destructive", compact && "truncate")}>
                        数据异常：缺少功能数据
                    </div>
                )
            }
            return (
                <div className={cn("flex gap-1.5", compact ? "items-center overflow-hidden" : "flex-wrap")}>
                    {compact ? (
                        <>
                            {feature_data.slice(0, 2).map((feature, index) => (
                                <Badge key={index} variant="secondary" className={cn("text-xs shrink-0")}>
                                    {feature}
                                </Badge>
                            ))}
                            {feature_data.length > 2 && (
                                <span className="text-xs text-muted-foreground">+{feature_data.length - 2}</span>
                            )}
                        </>
                    ) : (
                        feature_data.map((feature, index) => (
                            <Badge key={index} variant="secondary" className={cn("text-xs")}>
                                {feature}
                            </Badge>
                        ))
                    )}
                </div>
            )
        }

        // request_type: 6, 7 - 显示 oauth_client_data
        if (request_type === 6 || request_type === 7) {
            // 如果没有 oauth_client_data 或为空
            if (!oauth_client_data || !oauth_client_data.scope_data || oauth_client_data.scope_data.length === 0) {
                return compact ? (
                    <span className="text-sm text-muted-foreground">-</span>
                ) : (
                    <div className="text-sm text-muted-foreground">
                        无
                    </div>
                )
            }

            // 有授权范围数据
            return (
                <div className={cn("flex items-center gap-2", compact && "overflow-hidden")}>
                    {!compact && <span className="text-xs text-muted-foreground whitespace-nowrap">授权范围:</span>}
                    <div className={cn("flex gap-1.5", compact ? "overflow-hidden" : "flex-wrap")}>
                        {compact ? (
                            <>
                                {oauth_client_data.scope_data.slice(0, 2).map((scope, index) => (
                                    <Badge key={index} variant="outline" className={cn("text-xs shrink-0")}>
                                        {scope}
                                    </Badge>
                                ))}
                                {oauth_client_data.scope_data.length > 2 && (
                                    <span className="text-xs text-muted-foreground">+{oauth_client_data.scope_data.length - 2}</span>
                                )}
                            </>
                        ) : (
                            oauth_client_data.scope_data.map((scope, index) => (
                                <Badge key={index} variant="outline" className={cn("text-xs")}>
                                    {scope}
                                </Badge>
                            ))
                        )}
                    </div>
                </div>
            )
        }

        // 其他情况或数据不存在
        return (
            <div className={cn("text-sm text-muted-foreground", compact && "truncate")}>
                无附加数据
            </div>
        )
    }

    // 如果显示标签，添加标题
    if (showLabel && dataLabel) {
        return (
            <div className="space-y-2">
                <div className="text-sm font-medium text-muted-foreground">
                    {dataLabel}
                </div>
                <div className={cn(!compact && "pl-4")}>
                    {renderContent()}
                </div>
            </div>
        )
    }

    return renderContent()
}
