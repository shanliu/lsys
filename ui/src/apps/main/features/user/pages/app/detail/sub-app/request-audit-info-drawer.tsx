import type { AppSubRequestItemType } from "@shared/apis/user/app";
import { Badge } from "@shared/components/ui/badge";
import { Separator } from "@shared/components/ui/separator";
import {
    Drawer,
    DrawerContent,
    DrawerDescription,
    DrawerHeader,
    DrawerTitle,
} from "@apps/main/components/local/drawer";
import { SubAppRequestDataDisplay } from "@apps/main/features/user/components/ui/sub-app-request-data-display";
import {
    cn,
    formatTime,
    TIME_STYLE
} from "@shared/lib/utils";
import { DictList } from "@shared/types/apis-dict";

interface SubAppRequestAuditInfoDrawerProps {
    request: AppSubRequestItemType | null;
    open: boolean;
    onOpenChange: (open: boolean) => void;
    /** 请求类型字典 */
    requestTypeDict?: DictList;
    /** 请求状态字典 */
    requestStatusDict?: DictList;
}

export function SubAppRequestAuditInfoDrawer({
    request,
    open,
    onOpenChange,
    requestTypeDict,
    requestStatusDict,
}: SubAppRequestAuditInfoDrawerProps) {
    // 检查是否有请求数据
    const hasRequestData = (data: AppSubRequestItemType): boolean => {
        const { request_type, change_data, feature_data, oauth_client_data } = data

        // 类型 1, 2 检查 change_data
        if ((request_type === 1 || request_type === 2) && change_data && Object.keys(change_data).length > 0) {
            return true
        }

        // 类型 8 检查 feature_data
        if (request_type === 8 && feature_data && Array.isArray(feature_data) && feature_data.length > 0) {
            return true
        }

        // 类型 6, 7 检查 oauth_client_data
        if ((request_type === 6 || request_type === 7) && oauth_client_data && Object.keys(oauth_client_data).length > 0) {
            return true
        }

        return false
    }

    return (
        <Drawer open={open} onOpenChange={onOpenChange}>
            <DrawerContent>
                <DrawerHeader className="mb-6">
                    <DrawerTitle>审核详情</DrawerTitle>
                    <DrawerDescription>
                        查看子应用申请的详细信息和审核结果
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
                                    <div className="flex items-start gap-2">
                                        <span className="text-muted-foreground min-w-[80px]">申请类型:</span>
                                        <Badge variant="outline" className={cn("text-xs")}>
                                            {requestTypeDict?.getLabel(String(request.request_type)) || `类型 ${request.request_type}`}
                                        </Badge>
                                    </div>

                                    {/* 应用信息 */}
                                    <Separator className="my-2" />
                                    <div className="flex items-start gap-2">
                                        <span className="text-muted-foreground min-w-[80px]">申请ID:</span>
                                        <span className="font-mono text-xs">{request.id}</span>
                                    </div>

                                    <div className="flex items-start gap-2">
                                        <span className="text-muted-foreground min-w-[80px]">应用ID:</span>
                                        <span className="font-mono text-xs">{request.app_id}</span>
                                    </div>

                                    {/* 请求用户 */}
                                    <Separator className="my-2" />
                                    {request.user_data && (
                                        <>
                                            <div className="flex items-start gap-2">
                                                <span className="text-muted-foreground min-w-[80px]">用户ID:</span>
                                                <span className="font-mono text-xs">{request.user_data.id}</span>
                                            </div>
                                            <div className="flex items-start gap-2">
                                                <span className="text-muted-foreground min-w-[80px]">用户类型:</span>
                                                <span className="text-xs">{request.user_data.app_id === 0 ? "系统用户" : "应用用户"}</span>
                                            </div>
                                            <div className="flex items-start gap-2">
                                                <span className="text-muted-foreground min-w-[80px]">用户昵称:</span>
                                                <span className="text-xs">{request.user_data.user_nickname}</span>
                                            </div>
                                            <div className="flex items-start gap-2">
                                                <span className="text-muted-foreground min-w-[80px]">用户账号:</span>
                                                <span className="text-xs">{request.user_data.user_account}</span>
                                            </div>
                                        </>
                                    )}

                                    {/* 请求时间 */}
                                    <div className="flex items-start gap-2">
                                        <span className="text-muted-foreground min-w-[80px]">请求时间:</span>
                                        <span className="text-xs">
                                            {formatTime(request.request_time, TIME_STYLE.RELATIVE_ELEMENT)}
                                        </span>
                                    </div>

                                    {/* 请求数据 */}
                                    {request && hasRequestData(request) && (
                                        <>
                                            <Separator className="my-2" />
                                            <div className="space-y-2">
                                                <span className="text-muted-foreground">请求数据:</span>
                                                <div>
                                                    <SubAppRequestDataDisplay data={request} mode="drawer" showLabel={true} />
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
                                    <div className="flex items-start gap-2">
                                        <span className="text-muted-foreground min-w-[80px]">审核状态:</span>
                                        <Badge variant="secondary">
                                            {requestStatusDict?.getLabel(String(request.status)) || `状态 ${request.status}`}
                                        </Badge>
                                    </div>

                                    {/* 确认时间 */}
                                    {request.confirm_time && (
                                        <div className="flex items-start gap-2">
                                            <span className="text-muted-foreground min-w-[80px]">确认时间:</span>
                                            <span className="text-xs">
                                                {formatTime(request.confirm_time, TIME_STYLE.RELATIVE_ELEMENT)}
                                            </span>
                                        </div>
                                    )}

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
