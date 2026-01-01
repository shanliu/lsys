import {
    Drawer,
    DrawerContent,
    DrawerDescription,
    DrawerHeader,
    DrawerTitle,
} from "@apps/main/components/local/drawer";
import { AppRequestDataDisplay } from "@apps/main/features/user/components/ui/app-request-data-display";
import { useAuthData } from "@apps/main/hooks/use-auth-data";
import { type TypedDictData } from "@apps/main/hooks/use-dict-data";
import { type StatusMapper } from "@apps/main/lib/status-utils";
import { type AppRequestItemType } from "@shared/apis/user/app";
import CopyableText from "@shared/components/custom/text/copyable-text";
import { Badge } from "@shared/components/ui/badge";
import { cn, formatTime, TIME_STYLE } from "@shared/lib/utils";
import { Link } from "@tanstack/react-router";
import { useMemo } from "react";

interface RequestDetailDrawerProps {
    request: AppRequestItemType | null;
    isOpen: boolean;
    onOpenChange: (open: boolean) => void;
    dictData: TypedDictData<["user_app"]>;
    requestStatusMapper: StatusMapper<number>;
}

export function RequestDetailDrawer({
    request,
    isOpen,
    onOpenChange,
    dictData,
    requestStatusMapper,
}: RequestDetailDrawerProps) {
    // 获取当前登录用户信息
    const authData = useAuthData();
    const currentUserId = authData.userId;

    const requestTypeMapper = useMemo(
        () => (status: number) =>
            dictData.request_type.getLabel(String(status)) || String(status),
        [dictData.request_type]
    );

    if (!request) return null;

    return (
        <Drawer open={isOpen} onOpenChange={onOpenChange}>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>请求详情</DrawerTitle>
                    <DrawerDescription>请求ID: {request.id}</DrawerDescription>
                </DrawerHeader>

                <div className="mt-6 space-y-6">
                    {/* 应用信息 */}
                    <div className="space-y-4">
                        <h3 className="text-sm font-semibold border-b pb-2">应用信息</h3>
                        <div className="space-y-3 text-sm">
                            <div className="flex items-start gap-2">
                                <span className="text-muted-foreground min-w-[80px]">
                                    应用名称:
                                </span>
                                <span className="font-medium">
                                    {request.app_name || "-"}
                                </span>
                            </div>
                            <div className="flex items-start gap-2">
                                <span className="text-muted-foreground min-w-[80px]">
                                    应用标识:
                                </span>
                                {request.app_client ? (
                                    <CopyableText
                                        value={request.app_client}
                                        message="应用标识已复制"
                                        showIcon={true}
                                        className="bg-muted"
                                    />
                                ) : (
                                    <span>-</span>
                                )}
                            </div>
                        </div>
                    </div>

                    {/* 请求信息 */}
                    <div className="space-y-4">
                        <h3 className="text-sm font-semibold border-b pb-2">请求信息</h3>
                        <div className="space-y-3 text-sm">
                            <div className="flex items-start gap-2">
                                <span className="text-muted-foreground min-w-[80px]">
                                    请求类型:
                                </span>
                                <Badge variant="outline">
                                    {requestTypeMapper(request.request_type)}
                                </Badge>
                            </div>
                            <div className="flex items-start gap-2">
                                <span className="text-muted-foreground min-w-[80px]">
                                    请求状态:
                                </span>
                                <Badge
                                    className={requestStatusMapper.getClass(request.status || 1)}
                                >
                                    {requestStatusMapper.getText(request.status || 1)}
                                </Badge>
                            </div>
                            <div className="flex items-start gap-2">
                                <span className="text-muted-foreground min-w-[80px]">
                                    请求时间:
                                </span>
                                <span>
                                    {request.request_time
                                        ? formatTime(request.request_time, TIME_STYLE.ABSOLUTE_TEXT)
                                        : "-"}
                                </span>
                            </div>
                        </div>
                    </div>

                    {/* 请求数据 */}
                    <div className="space-y-4">
                        <h3 className="text-sm font-semibold border-b pb-2">请求数据</h3>
                        <div className="p-3 bg-muted/50 rounded-md">
                            <AppRequestDataDisplay data={request} />
                        </div>
                    </div>

                    {/* 审核信息 */}
                    <div className="space-y-4">
                        <h3 className="text-sm font-semibold border-b pb-2">审核信息</h3>
                        <div className="space-y-3 text-sm">
                            {/* 状态为1（待审）时显示未审核提示 */}
                            {request.status === 1 && (
                                <div className="flex items-start gap-2">
                                    <span className="text-muted-foreground">该请求由</span>
                                    {request.parent_app_id === 0 ? (
                                        <span className="font-medium">系统</span>
                                    ) : request.parent_app_name && request.parent_app_client_id ? (
                                        // 如果父应用所有者是当前用户，显示可点击链接
                                        request.parent_app_id && currentUserId && request.parent_app_id > 0 ? (
                                            <Link
                                                to="/user/app/$appId/sub-app/request"
                                                params={{ appId: request.parent_app_id }}
                                                className="font-medium text-primary hover:underline"
                                            >
                                                {`${request.parent_app_name}(${request.parent_app_client_id})`}
                                            </Link>
                                        ) : (
                                            <span className="font-medium">
                                                {`${request.parent_app_name}(${request.parent_app_client_id})`}
                                            </span>
                                        )
                                    ) : (
                                        <span className="font-medium">{request.parent_app_name || "-"}</span>
                                    )}
                                    <span className="text-muted-foreground">进行审核</span>
                                </div>
                            )}

                            {/* 状态为2（批准）或3（驳回）时显示审核来源 */}
                            {(request.status === 2 || request.status === 3) && (
                                <div className="flex items-start gap-2">
                                    <span className="text-muted-foreground">该请求由</span>
                                    {request.parent_app_id === 0 ? (
                                        <span className="font-medium">系统</span>
                                    ) : request.parent_app_name && request.parent_app_client_id ? (
                                        // 如果父应用所有者是当前用户，显示可点击链接
                                        request.parent_app_id && currentUserId && request.parent_app_id > 0 ? (
                                            <Link
                                                to="/user/app/$appId/sub-app/request"
                                                params={{ appId: request.parent_app_id }}
                                                className="font-medium text-primary hover:underline"
                                            >
                                                {`${request.parent_app_name}(${request.parent_app_client_id})`}
                                            </Link>
                                        ) : (
                                            <span className="font-medium">
                                                {`${request.parent_app_name}(${request.parent_app_client_id})`}
                                            </span>
                                        )
                                    ) : (
                                        <span className="font-medium">{request.parent_app_name || "-"}</span>
                                    )}
                                    <span className="text-muted-foreground">
                                        {request.status === 2 ? "审核通过" : "驳回申请"}
                                    </span>
                                </div>
                            )}

                            {/* 状态为4（作废）时显示作废提示 */}
                            {request.status === 4 && (
                                <div className="text-muted-foreground">
                                    请求已经作废
                                </div>
                            )}

                            {/* 状态为2（批准）或3（驳回）时显示审核详情 */}
                            {(request.status === 2 || request.status === 3) && (
                                <>
                                    <div className="flex items-start gap-2">
                                        <span className="text-muted-foreground min-w-[80px]">
                                            审核时间:
                                        </span>
                                        <span>
                                            {request.confirm_time
                                                ? formatTime(
                                                    request.confirm_time,
                                                    TIME_STYLE.ABSOLUTE_TEXT
                                                )
                                                : "-"}
                                        </span>
                                    </div>
                                    {request.confirm_user_data && (
                                        <div className="flex items-start gap-2">
                                            <span className="text-muted-foreground min-w-[80px]">
                                                审核人:
                                            </span>
                                            <span>
                                                {request.confirm_user_data.user_nickname ||
                                                    request.confirm_user_data.user_account}
                                            </span>
                                        </div>
                                    )}
                                    {request.confirm_note && (
                                        <div className="flex items-start gap-2">
                                            <span className="text-muted-foreground min-w-[80px]">
                                                审核备注:
                                            </span>
                                            <div className="flex-1 p-2 bg-muted/50 rounded text-xs whitespace-pre-wrap break-words">
                                                {request.confirm_note}
                                            </div>
                                        </div>
                                    )}
                                </>
                            )}
                        </div>
                    </div>
                </div>
            </DrawerContent>
        </Drawer>
    );
}
