import type { AppRequestItemType } from "@shared/apis/admin/app";
import { UserDataTooltip } from "@apps/main/components/local/user-data-tooltip";
import CopyableText from "@shared/components/custom/text/copyable-text";
import { Badge } from "@shared/components/ui/badge";
import { Separator } from "@shared/components/ui/separator";
import {
    Drawer,
    DrawerContent,
    DrawerDescription,
    DrawerHeader,
    DrawerTitle,
} from "@apps/main/components/local/drawer";
import {
    cn,
    formatTime,
    TIME_STYLE
} from "@shared/lib/utils";
import { DictList } from "@shared/types/apis-dict";
import { AppRequestDataDisplay } from "../../components/ui/app-request-data-display";
import { hasRequestData } from "../../components/ui/app-request-utils";

interface RequestAuditInfoDrawerProps {
    request: AppRequestItemType | null;
    open: boolean;
    onOpenChange: (open: boolean) => void;
    /** 请求类型字典 */
    requestTypeDict?: DictList;
    /** 请求状态字典 */
    requestStatusDict?: DictList;
}

export function RequestAuditInfoDrawer({
    request,
    open,
    onOpenChange,
    requestTypeDict,
    requestStatusDict,
}: RequestAuditInfoDrawerProps) {
    return (
        <Drawer open={open} onOpenChange={onOpenChange}>
            <DrawerContent>
                <DrawerHeader className={cn("mb-6")}>
                    <DrawerTitle>审核详情</DrawerTitle>
                    <DrawerDescription>
                        查看申请的详细信息和审核结果
                    </DrawerDescription>
                </DrawerHeader>

                {/* 申请信息展示 - 可滚动区域 */}
                <div className="space-y-6 pb-6">
                    {request && (
                        <>
                            {/* 申请信息 */}
                            <div className="space-y-4">
                                <h3 className="font-semibold text-base">申请信息</h3>
                                <div className="grid gap-3 text-sm">
                                    {/* 申请类型 */}
                                    <div className="flex items-start gap-2 flex-wrap">
                                        <span className="text-muted-foreground shrink-0">申请类型:</span>
                                        <Badge variant="outline" className={cn("text-xs")}>
                                            {requestTypeDict?.getLabel(String(request.request_type)) || `类型 ${request.request_type}`}
                                        </Badge>
                                    </div>

                                    {/* 应用信息 */}
                                    <Separator className="my-2" />
                                    <div className="flex items-start gap-2 flex-wrap">
                                        <span className="text-muted-foreground shrink-0">应用ID:</span>
                                        <span className="font-mono text-xs">{request.app_id}</span>
                                    </div>

                                    {/* 应用名称和标识 */}
                                    {request.app_name && (
                                        <div className="flex items-start gap-2 flex-wrap">
                                            <span className="text-muted-foreground shrink-0">应用名称:</span>
                                            <span className="font-medium break-all">{request.app_name}</span>
                                        </div>
                                    )}
                                    {request.app_client && (
                                        <div className="flex items-start gap-2 flex-wrap">
                                            <span className="text-muted-foreground shrink-0">应用标识:</span>
                                            <CopyableText value={request.app_client} className="text-xs break-all" />
                                        </div>
                                    )}

                                    {/* 请求用户 */}
                                    <Separator className="my-2" />
                                    <div className="flex items-start gap-2 flex-wrap">
                                        <span className="text-muted-foreground shrink-0">请求用户:</span>
                                        <UserDataTooltip userData={request.request_user_data} className="text-xs" />
                                    </div>

                                    {/* 请求时间 */}
                                    <div className="flex items-start gap-2 flex-wrap">
                                        <span className="text-muted-foreground shrink-0">请求时间:</span>
                                        <span className="text-xs">
                                            {formatTime(request.request_time, TIME_STYLE.RELATIVE_ELEMENT)}
                                        </span>
                                    </div>

                                    {/* 请求数据 */}
                                    {hasRequestData(request) && (
                                        <>
                                            <Separator className="my-2" />
                                            <div className="space-y-2">
                                                <span className="text-muted-foreground">请求数据:</span>
                                                <div>
                                                    <AppRequestDataDisplay data={request} mode="drawer" />
                                                </div>
                                            </div>
                                        </>
                                    )}
                                </div>
                            </div>

                            <Separator />

                            {/* 审核信息 */}
                            <div className="space-y-4">
                                <h3 className="font-semibold text-base">审核信息</h3>
                                <div className="grid gap-3 text-sm">
                                    {/* 审核状态 */}
                                    <div className="flex items-start gap-2 flex-wrap">
                                        <span className="text-muted-foreground shrink-0">审核状态:</span>
                                        <Badge variant="secondary">
                                            {requestStatusDict?.getLabel(String(request.status)) || `状态 ${request.status}`}
                                        </Badge>
                                    </div>

                                    {/* 确认时间 */}
                                    <div className="flex items-start gap-2 flex-wrap">
                                        <span className="text-muted-foreground shrink-0">确认时间:</span>
                                        <span className="text-xs">
                                            {formatTime(request.confirm_time ?? null, TIME_STYLE.RELATIVE_ELEMENT)}
                                        </span>
                                    </div>

                                    {/* 确认用户 */}
                                    <div className="flex items-start gap-2 flex-wrap">
                                        <span className="text-muted-foreground shrink-0">确认用户:</span>
                                        <span className="font-mono text-xs">ID: {request.confirm_user_id || '-'}</span>
                                    </div>

                                    {/* 确认备注 */}
                                    <div className="flex flex-col gap-2">
                                        <span className="text-muted-foreground">确认备注:</span>
                                        <div className="min-h-[60px] p-3 border rounded-md bg-background/50">
                                            {request.confirm_note || <span className="text-muted-foreground/50">暂无备注</span>}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </>
                    )}
                </div>
            </DrawerContent>
        </Drawer>
    );
}
