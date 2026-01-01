import { AppRequestItemType } from "@shared/apis/user/app"
import { Badge } from "@shared/components/ui/badge"
import { cn } from "@shared/lib/utils"

interface AppRequestDataDisplayProps {
    data: AppRequestItemType
    /** 紧凑模式 - 单行显示，超出截断 */
    compact?: boolean
}

/**
 * 应用请求附带数据显示组件
 * 根据 request_type 显示不同的数据格式
 */
export function AppRequestDataDisplay({ data, compact = false }: AppRequestDataDisplayProps) {
    const { request_type, change_data, feature_data, oauth_client_data } = data

    // request_type: 3, 4, 5 - 显示"无附带数据"
    if (request_type === 3 || request_type === 4 || request_type === 5) {
        return !compact ? (
            <div className={cn("text-sm text-muted-foreground", compact && "truncate")}>
                无相关数据
            </div>
        ) : (
            "-"
        )
    }

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

    // request_type: 8 - 显示 feature_data
    if (request_type === 8) {
        if (!feature_data || !feature_data.feature) {
            return (
                <div className={cn("text-sm text-destructive", compact && "truncate")}>
                    数据异常：缺少功能数据
                </div>
            )
        }

        const features = feature_data.feature.split(/[,，]/).map(f => f.trim()).filter(Boolean);

        return (
            <div className={cn("flex gap-1.5", compact ? "items-center overflow-hidden" : "flex-wrap")}>
                {features.map((feature, i) => (
                    <Badge
                        key={i}
                        variant="secondary"
                        className={cn("text-xs", compact && "truncate max-w-[150px]")}
                        title={feature}
                    >
                        {feature}
                    </Badge>
                ))}
            </div>
        )
    }

    // request_type: 6, 7 - 显示 oauth_client_data
    if (request_type === 6 || request_type === 7) {
        // 过滤掉空字符串
        const validScopes = oauth_client_data?.scope_data?.filter((scope: string) => scope && scope.trim()) || []

        // 如果没有有效的授权范围数据
        if (validScopes.length === 0) {
            return compact ? "-" : (
                <div className="text-sm text-muted-foreground">
                    无
                </div>
            )
        }

        // 有授权范围数据
        const displayScopes = compact ? validScopes.slice(0, 2) : validScopes
        const hasMore = compact && validScopes.length > 2

        return (
            <>
                {!compact && <span className="text-xs text-muted-foreground whitespace-nowrap">授权范围:</span>}
                <div className={cn("flex gap-1.5", compact ? "overflow-hidden" : "flex-wrap")}>
                    {displayScopes.map((scope: string, index: number) => (
                        <Badge key={index} variant="outline" className={cn("text-xs", compact && "shrink-0")}>
                            {scope}
                        </Badge>
                    ))}
                    {hasMore && (
                        <span className="text-xs text-muted-foreground">+{validScopes.length - 2}</span>
                    )}
                </div>
            </>
        )
    }

    // 其他情况或数据不存在
    return (
        <div className={cn("text-sm text-muted-foreground", compact && "truncate")}>
            无数据
        </div>
    )
}
