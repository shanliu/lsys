import {
    Drawer,
    DrawerContent,
    DrawerDescription,
    DrawerHeader,
    DrawerTitle,
} from "@apps/main/components/local/drawer";
import { NotifyKeyLink } from "@apps/main/features/user/components/ui/notify-key-link";
import { type TypedDictData } from "@apps/main/hooks/use-dict-data";
import { type StatusMapper } from "@apps/main/lib/status-utils";
import { type AppNotifyListItemType } from "@shared/apis/user/app";
import CopyableText from "@shared/components/custom/text/copyable-text";
import { Badge } from "@shared/components/ui/badge";
import { cn, formatTime, TIME_STYLE } from "@shared/lib/utils";

interface NotifyDetailDrawerProps {
    notify: AppNotifyListItemType | null;
    isOpen: boolean;
    onOpenChange: (open: boolean) => void;
    dictData: TypedDictData<["user_app"]>;
    notifyStatusMapper: StatusMapper<string>;
    appId: string;
}

export function NotifyDetailDrawer({
    notify,
    isOpen,
    onOpenChange,
    dictData,
    notifyStatusMapper,
    appId,
}: NotifyDetailDrawerProps) {
    if (!notify) return null;

    const notifyMethodLabel = dictData.notify_method.getLabel(notify.notify_method) || notify.notify_method;

    return (
        <Drawer open={isOpen} onOpenChange={onOpenChange}>
            <DrawerContent
                className={cn("w-full sm:w-[600px] sm:max-w-[90vw]")}
            >
                <DrawerHeader>
                    <DrawerTitle>回调详情</DrawerTitle>
                    <DrawerDescription>回调ID: {notify.id}</DrawerDescription>
                </DrawerHeader>

                <div className="mt-6 space-y-6">
                    {/* 回调信息 */}
                    <div className="space-y-4">
                        <h3 className="text-sm font-semibold border-b pb-2">回调信息</h3>
                        <div className="space-y-3 text-sm">
                            <div className={cn("flex items-start gap-2")}>
                                <span className={cn("text-muted-foreground min-w-[80px]")}>
                                    回调方法:
                                </span>
                                <Badge variant="outline">
                                    {notifyMethodLabel}
                                </Badge>
                            </div>
                            <div className={cn("flex items-start gap-2")}>
                                <span className={cn("text-muted-foreground min-w-[80px]")}>
                                    回调状态:
                                </span>
                                <Badge
                                    className={notifyStatusMapper.getClass(String(notify.status))}
                                >
                                    {notifyStatusMapper.getText(String(notify.status))}
                                </Badge>
                            </div>
                            <div className={cn("flex items-start gap-2")}>
                                <span className={cn("text-muted-foreground min-w-[80px]")}>
                                    回调类型:
                                </span>
                                <span className={cn("font-medium")}>
                                    {notify.notify_type || "-"}
                                </span>
                            </div>
                            <div className={cn("flex items-start gap-2")}>
                                <span className={cn("text-muted-foreground min-w-[80px]")}>
                                    相关数据:
                                </span>
                                <NotifyKeyLink
                                    appId={appId}
                                    notifyMethod={notify.notify_method}
                                    notifyKey={notify.notify_key}
                                    className={cn("font-medium text-primary hover:underline")}
                                />
                            </div>
                            <div className={cn("flex flex-col gap-2")}>
                                <span className={cn("text-muted-foreground")}>
                                    回调地址:
                                </span>
                                <div>
                                    {notify.call_url ? (
                                        <div className="p-3 bg-muted/50 rounded-md">
                                            <CopyableText
                                                value={notify.call_url}
                                                message="回调地址已复制"
                                                showIcon={true}
                                                className={cn("text-xs break-all")}
                                            />
                                        </div>
                                    ) : (
                                        <span>-</span>
                                    )}
                                </div>
                            </div>
                            <div className={cn("flex flex-col gap-2")}>
                                <span className={cn("text-muted-foreground")}>
                                    回调数据:
                                </span>
                                <div>
                                    <div className={cn("p-3 bg-muted/50 rounded-md")}>
                                        <pre className={cn("text-xs whitespace-pre-wrap break-words")}>
                                            {notify.notify_payload}
                                        </pre>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* 时间信息 */}
                    <div className="space-y-4">
                        <h3 className="text-sm font-semibold border-b pb-2">时间信息</h3>
                        <div className="space-y-3 text-sm">
                            <div className={cn("flex items-start gap-2")}>
                                <span className={cn("text-muted-foreground min-w-[80px]")}>
                                    推送时间:
                                </span>
                                <span>
                                    {notify.publish_time
                                        ? formatTime(notify.publish_time, TIME_STYLE.ABSOLUTE_TEXT)
                                        : "-"}
                                </span>
                            </div>
                            <div className={cn("flex items-start gap-2")}>
                                <span className={cn("text-muted-foreground min-w-[80px]")}>
                                    下次尝试:
                                </span>
                                <span>
                                    {notify.next_time
                                        ? formatTime(notify.next_time, TIME_STYLE.ABSOLUTE_TEXT)
                                        : "-"}
                                </span>
                            </div>
                        </div>
                    </div>

                    {/* 重试信息 */}
                    <div className="space-y-4">
                        <h3 className="text-sm font-semibold border-b pb-2">重试信息</h3>
                        <div className="space-y-3 text-sm">
                            <div className={cn("flex items-start gap-2")}>
                                <span className={cn("text-muted-foreground min-w-[80px]")}>
                                    当前尝试:
                                </span>
                                <span className={cn("font-medium")}>
                                    {notify.try_num} / {notify.try_max}
                                </span>
                            </div>
                        </div>
                    </div>

                    {/* 回调结果 */}
                    {notify.result && (
                        <div className="space-y-4">
                            <h3 className="text-sm font-semibold border-b pb-2">回调结果</h3>
                            <div className={cn("p-3 bg-muted/50 rounded-md")}>
                                <pre className={cn("text-xs whitespace-pre-wrap break-words")}>
                                    {notify.result}
                                </pre>
                            </div>
                        </div>
                    )}
                </div>
            </DrawerContent>
        </Drawer>
    );
}
