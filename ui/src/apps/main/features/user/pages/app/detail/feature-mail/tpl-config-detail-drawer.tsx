import { UserSenderMailerTplConfigItemType } from "@shared/apis/user/sender-mailer";
import { SenderTplConfigView } from "@apps/main/components/local/sender-config/tpl-config-view";
import { Drawer, DrawerContent, DrawerHeader, DrawerTitle } from "@apps/main/components/local/drawer";
import { type TypedDictData } from "@apps/main/hooks/use-dict-data";
import { cn, formatTime, TIME_STYLE } from "@shared/lib/utils";

interface TplConfigDetailDrawerProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    config: UserSenderMailerTplConfigItemType;
    dictData: TypedDictData<["user_sender_mailer"]>;
}

export function TplConfigDetailDrawer({
    open,
    onOpenChange,
    config,
    dictData,
}: TplConfigDetailDrawerProps) {
    return (
        <Drawer open={open} onOpenChange={onOpenChange}>
            <DrawerContent>
                <DrawerHeader className={cn("pb-6")}>
                    <DrawerTitle className={cn("text-xl")}>配置详细信息</DrawerTitle>
                </DrawerHeader>

                <div className="space-y-6">
                    {/* 基本信息块 */}
                    <div className="space-y-4">
                        <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">
                            基本信息
                        </h3>
                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">ID</span>
                                <span className="text-sm font-medium">{config.id}</span>
                            </div>

                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">配置名称</span>
                                <span className="text-sm font-medium">{config.name}</span>
                            </div>

                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">模板Key</span>
                                <span className="text-sm font-medium">{config.tpl_key}</span>
                            </div>

                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">配置类型</span>
                                <span className="text-sm font-medium">
                                    {dictData.config_type?.getLabel(String(config.setting_key)) ||
                                        config.setting_key}
                                </span>
                            </div>

                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">SMTP配置名称</span>
                                <span className="text-sm font-medium">{config.setting_name}</span>
                            </div>
                        </div>
                    </div>

                    <div className="border-t" />

                    {/* 时间信息块 */}
                    <div className="space-y-4">
                        <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">
                            时间信息
                        </h3>
                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">修改时间</span>
                                <span className="text-sm font-medium">
                                    {config.change_time
                                        ? formatTime(config.change_time, TIME_STYLE.ABSOLUTE_TEXT)
                                        : "-"}
                                </span>
                            </div>
                        </div>
                    </div>

                    <div className="border-t" />

                    {/* 配置详情块 */}
                    <div className="space-y-4">
                        <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">
                            配置详情
                        </h3>
                        <div className="p-4 border rounded bg-muted/20">
                            <SenderTplConfigView
                                config_data={config.config_data}
                                setting_key={config.setting_key}
                                variant="detail"
                            />
                        </div>
                    </div>
                </div>
            </DrawerContent>
        </Drawer>
    );
}
