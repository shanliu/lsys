import {
    Drawer,
    DrawerContent,
    DrawerHeader,
    DrawerTitle,
} from "@apps/main/components/local/drawer";
import type { SystemUserChangeLogItemType } from "@shared/apis/admin/user";
import CopyableText from "@shared/components/custom/text/copyable-text";
import { Badge } from "@shared/components/ui/badge";
import {
    cn,
    formatTime,
    TIME_STYLE,
} from "@shared/lib/utils";

interface ChangeLogDetailDrawerProps {
    log: SystemUserChangeLogItemType;
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function ChangeLogDetailDrawer({
    log,
    open,
    onOpenChange,
}: ChangeLogDetailDrawerProps) {
    let logData: any = {};
    try {
        logData = JSON.parse(log.log_data);
    } catch {
        logData = { raw: log.log_data };
    }

    return (
        <Drawer open={open} onOpenChange={onOpenChange}>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>变更日志详情 #{log.id}</DrawerTitle>
                </DrawerHeader>
                <div className="space-y-4 mt-6">
                    <div className="grid grid-cols-2 gap-4">
                        <div>
                            <label className="text-sm font-medium text-muted-foreground">
                                日志类型
                            </label>
                            <div className="mt-1">
                                <Badge variant="secondary">{log.log_type}</Badge>
                            </div>
                        </div>
                        <div>
                            <label className="text-sm font-medium text-muted-foreground">
                                操作时间
                            </label>
                            <div className="mt-1 text-sm">
                                {formatTime(log.add_time, TIME_STYLE.RELATIVE_ELEMENT)}
                            </div>
                        </div>
                        <div>
                            <label className="text-sm font-medium text-muted-foreground">
                                操作用户ID
                            </label>
                            <div className="mt-1 text-sm">{log.add_user_id}</div>
                        </div>
                        <div>
                            <label className="text-sm font-medium text-muted-foreground">
                                操作IP
                            </label>
                            <div className="mt-1">
                                <CopyableText value={log.add_user_ip} className={cn("text-sm")} />
                            </div>
                        </div>
                        <div>
                            <label className="text-sm font-medium text-muted-foreground">
                                来源ID
                            </label>
                            <div className="mt-1">
                                <CopyableText value={log.source_id} className={cn("text-sm")} />
                            </div>
                        </div>
                        <div className="col-span-2">
                            <label className="text-sm font-medium text-muted-foreground">
                                设备ID
                            </label>
                            <div className="mt-1">
                                <CopyableText value={log.device_id} className={cn("text-sm")} />
                            </div>
                        </div>
                        <div className="col-span-2">
                            <label className="text-sm font-medium text-muted-foreground">
                                请求ID
                            </label>
                            <div className="mt-1">
                                <CopyableText value={log.request_id} className={cn("text-sm")} />
                            </div>
                        </div>
                    </div>

                    <div>
                        <label className="text-sm font-medium text-muted-foreground">
                            操作描述
                        </label>
                        <div className="mt-1 p-3 bg-muted rounded-md text-sm">
                            {log.message}
                        </div>
                    </div>

                    <div>
                        <label className="text-sm font-medium text-muted-foreground">
                            User Agent
                        </label>
                        <div className="mt-1 p-3 bg-muted rounded-md text-sm break-all">
                            {log.request_user_agent}
                        </div>
                    </div>

                    <div>
                        <label className="text-sm font-medium text-muted-foreground">
                            变更数据
                        </label>
                        <div className="mt-1 p-3 bg-muted rounded-md">
                            <pre className="text-xs overflow-x-auto whitespace-pre-wrap">
                                {JSON.stringify(logData, null, 2)}
                            </pre>
                        </div>
                    </div>
                </div>
            </DrawerContent>
        </Drawer>
    );
}