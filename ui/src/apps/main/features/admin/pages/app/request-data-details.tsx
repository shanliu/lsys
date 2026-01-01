import { AppRequestItemType } from '@shared/apis/admin/app'
import { Badge } from '@shared/components/ui/badge'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@shared/components/ui/collapsible'
import { cn } from '@shared/lib/utils'
import type { DictList } from '@shared/types/apis-dict'
import { ChevronDown, ChevronRight } from 'lucide-react'
import { useState } from 'react'

interface RequestDataDetailsProps {
    request: AppRequestItemType
    requestTypeDict?: DictList
}

export function RequestDataDetails({ request, requestTypeDict }: RequestDataDetailsProps) {
    const hasChangeData = request.change_data && Object.keys(request.change_data).length > 0
    const hasFeatureData = request.feature_data && Object.keys(request.feature_data).length > 0
    const hasOAuthData = request.oauth_client_data && Object.keys(request.oauth_client_data).length > 0

    // 判断是否为申请类型（type 1）还是变更类型（type 2）
    const isApplication = request.request_type === 1

    // 默认展开第一个有数据的项
    const [isChangeDataOpen, setIsChangeDataOpen] = useState(Boolean(hasChangeData))
    const [isFeatureDataOpen, setIsFeatureDataOpen] = useState(Boolean(!hasChangeData && hasFeatureData))
    const [isOAuthDataOpen, setIsOAuthDataOpen] = useState(Boolean(!hasChangeData && !hasFeatureData && hasOAuthData))

    if (!hasChangeData && !hasFeatureData && !hasOAuthData) {
        return null
    }

    return (
        <div className="space-y-2">
            {/* 申请/变更数据 */}
            {hasChangeData && (
                <Collapsible open={isChangeDataOpen} onOpenChange={setIsChangeDataOpen}>
                    <CollapsibleTrigger className={cn("flex items-center gap-2 text-xs text-muted-foreground hover:text-foreground transition-colors")}>
                        {isChangeDataOpen ? (
                            <ChevronDown className={cn("h-3 w-3")} />
                        ) : (
                            <ChevronRight className={cn("h-3 w-3")} />
                        )}
                        <Badge variant="outline" className={cn("text-xs")}>
                            {isApplication ? '申请数据' : '变更数据'}
                        </Badge>
                    </CollapsibleTrigger>
                    <CollapsibleContent className={cn("mt-1")}>
                        <div className="bg-muted/20 rounded p-3 border border-dashed border-muted">
                            <div className="space-y-2">
                                {request.change_data?.client_id && (
                                    <div className="flex items-center gap-2">
                                        <span className="text-xs font-medium text-muted-foreground min-w-[60px]">
                                            客户端ID:
                                        </span>
                                        <span className="text-xs font-mono bg-muted px-2 py-1 rounded">
                                            {request.change_data.client_id}
                                        </span>
                                    </div>
                                )}
                                {request.change_data?.name && (
                                    <div className="flex items-center gap-2">
                                        <span className="text-xs font-medium text-muted-foreground min-w-[60px]">
                                            名称:
                                        </span>
                                        <span className="text-xs bg-muted px-2 py-1 rounded">
                                            {request.change_data.name}
                                        </span>
                                    </div>
                                )}
                            </div>
                        </div>
                    </CollapsibleContent>
                </Collapsible>
            )}

            {/* 功能数据 */}
            {hasFeatureData && (
                <Collapsible open={isFeatureDataOpen} onOpenChange={setIsFeatureDataOpen}>
                    <CollapsibleTrigger className={cn("flex items-center gap-2 text-xs text-muted-foreground hover:text-foreground transition-colors")}>
                        {isFeatureDataOpen ? (
                            <ChevronDown className={cn("h-3 w-3")} />
                        ) : (
                            <ChevronRight className={cn("h-3 w-3")} />
                        )}
                        <Badge variant="outline" className={cn("text-xs")}>
                            功能数据
                        </Badge>
                    </CollapsibleTrigger>
                    <CollapsibleContent className={cn("mt-1")}>
                        <div className="bg-muted/20 rounded p-3 border border-dashed border-muted">
                            <div className="space-y-2">
                                {request.feature_data?.feature && (
                                    <div className="flex items-start gap-2">
                                        <span className="text-xs font-medium text-muted-foreground min-w-[60px] mt-0.5">
                                            功能:
                                        </span>
                                        <div className="flex flex-wrap gap-1">
                                            {request.feature_data.feature.split(',').filter(Boolean).map((feature, index) => (
                                                <Badge key={index} variant="secondary" className={cn("text-xs")}>
                                                    {feature.trim()}
                                                </Badge>
                                            ))}
                                        </div>
                                    </div>
                                )}
                            </div>
                        </div>
                    </CollapsibleContent>
                </Collapsible>
            )}

            {/* OAuth客户端数据 */}
            {hasOAuthData && (
                <Collapsible open={isOAuthDataOpen} onOpenChange={setIsOAuthDataOpen}>
                    <CollapsibleTrigger className={cn("flex items-center gap-2 text-xs text-muted-foreground hover:text-foreground transition-colors")}>
                        {isOAuthDataOpen ? (
                            <ChevronDown className={cn("h-3 w-3")} />
                        ) : (
                            <ChevronRight className={cn("h-3 w-3")} />
                        )}
                        <Badge variant="outline" className={cn("text-xs")}>
                            OAuth数据
                        </Badge>
                    </CollapsibleTrigger>
                    <CollapsibleContent className={cn("mt-1")}>
                        <div className="bg-muted/20 rounded p-3 border border-dashed border-muted">
                            <div className="space-y-2">
                                <div className="space-y-1">
                                    <span className="text-xs font-medium text-muted-foreground">
                                        授权范围:
                                    </span>
                                    {request.oauth_client_data?.scope_data && request.oauth_client_data.scope_data.length > 0 ? (
                                        <div className="flex flex-wrap gap-1">
                                            {request.oauth_client_data.scope_data.map((scope, index) => (
                                                <Badge key={index} variant="secondary" className={cn("text-xs")}>
                                                    {scope}
                                                </Badge>
                                            ))}
                                        </div>
                                    ) : (
                                        <div className="text-xs text-muted-foreground">无</div>
                                    )}
                                </div>
                            </div>
                        </div>
                    </CollapsibleContent>
                </Collapsible>
            )}
        </div>
    )
}
