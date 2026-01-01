import type { SystemUserLoginHistoryItemType } from "@shared/apis/admin/user";
import {
    Drawer,
    DrawerContent,
    DrawerHeader,
    DrawerTitle,
} from "@apps/main/components/local/drawer";
import CopyableText from "@shared/components/custom/text/copyable-text";
import { Badge } from "@shared/components/ui/badge";
import {
    cn,
    formatTime,
    TIME_STYLE,
} from "@shared/lib/utils";

interface LoginLogDetailDrawerProps {
    login: SystemUserLoginHistoryItemType;
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function LoginLogDetailDrawer({
    login,
    open,
    onOpenChange,
}: LoginLogDetailDrawerProps) {
    let tokenData: any = {};
    try {
        tokenData = JSON.parse(login.token_data);
    } catch {
        tokenData = { raw: login.token_data };
    }

    return (
        <Drawer open={open} onOpenChange={onOpenChange}>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>登录详情 - 用户#{login.user_id}</DrawerTitle>
                </DrawerHeader>
                <div className="space-y-4 mt-6">
                    <div className="grid grid-cols-2 gap-4">
                        <div>
                            <label className="text-sm font-medium text-muted-foreground">
                                登录类型
                            </label>
                            <div className="mt-1">
                                <Badge variant="secondary">{login.login_type}</Badge>
                            </div>
                        </div>
                        <div>
                            <label className="text-sm font-medium text-muted-foreground">
                                登录状态
                            </label>
                            <div className="mt-1">
                                <Badge
                                    variant={login.status === 1 ? "default" : "secondary"}
                                >
                                    {login.status}
                                </Badge>
                            </div>
                        </div>
                        <div>
                            <label className="text-sm font-medium text-muted-foreground">
                                登录时间
                            </label>
                            <div className="mt-1 text-sm">
                                {formatTime(login.add_time, TIME_STYLE.RELATIVE_ELEMENT)}
                            </div>
                        </div>
                        <div>
                            <label className="text-sm font-medium text-muted-foreground">
                                过期时间
                            </label>
                            <div className="mt-1 text-sm">
                                {formatTime(login.expire_time, TIME_STYLE.ABSOLUTE_ELEMENT)}
                            </div>
                        </div>
                        <div>
                            <label className="text-sm font-medium text-muted-foreground">
                                登出时间
                            </label>
                            <div className="mt-1 text-sm">
                                {login.logout_time
                                    ? formatTime(login.logout_time, TIME_STYLE.ABSOLUTE_ELEMENT)
                                    : "未登出"}
                            </div>
                        </div>
                        <div>
                            <label className="text-sm font-medium text-muted-foreground">
                                登录IP
                            </label>
                            <div className="mt-1">
                                <CopyableText value={login.login_ip} className={cn("text-sm")} />
                            </div>
                        </div>
                        <div>
                            <label className="text-sm font-medium text-muted-foreground">
                                设备ID
                            </label>
                            <div className="mt-1">
                                <CopyableText value={login.device_id} className={cn("text-sm")} />
                            </div>
                        </div>
                        <div>
                            <label className="text-sm font-medium text-muted-foreground">
                                设备名称
                            </label>
                            <div className="mt-1 text-sm">{login.device_name}</div>
                        </div>
                        <div>
                            <label className="text-sm font-medium text-muted-foreground">
                                应用ID
                            </label>
                            <div className="mt-1 text-sm">{login.app_id}</div>
                        </div>
                        <div>
                            <label className="text-sm font-medium text-muted-foreground">
                                OAuth应用ID
                            </label>
                            <div className="mt-1 text-sm">{login.oauth_app_id}</div>
                        </div>
                    </div>

                    <div>
                        <label className="text-sm font-medium text-muted-foreground">
                            令牌数据
                        </label>
                        <div className="mt-1 p-3 bg-muted rounded-md">
                            <pre className="text-xs overflow-x-auto whitespace-pre-wrap">
                                {JSON.stringify(tokenData, null, 2)}
                            </pre>
                        </div>
                    </div>
                </div>
            </DrawerContent>
        </Drawer>
    );
}
