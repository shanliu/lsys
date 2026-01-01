import {
    Drawer,
    DrawerContent,
    DrawerHeader,
    DrawerTitle,
} from "@apps/main/components/local/drawer";
import { createStatusMapper } from "@apps/main/lib/status-utils";
import type { AuditDataType } from "@shared/apis/admin/rbac-base";
import CopyableText from "@shared/components/custom/text/copyable-text";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
} from "@shared/components/ui/collapsible";
import {
    cn,
    formatTime,
    TIME_STYLE,
} from "@shared/lib/utils";
import { DictList } from "@shared/types/apis-dict";
import { ChevronDown, ChevronRight, Shield } from "lucide-react";
import { useMemo, useState } from "react";

interface AuditLogDetailDrawerProps {
    audit: AuditDataType | null;
    open: boolean;
    onOpenChange: (open: boolean) => void;
    auditResultDict?: DictList;
}

export function AuditLogDetailDrawer({
    audit,
    open,
    onOpenChange,
    auditResultDict,
}: AuditLogDetailDrawerProps) {
    const [expandedDetails, setExpandedDetails] = useState<Set<number>>(
        new Set(),
    );

    // 审计结果状态映射器 - 必须在所有 hooks 之前，不能在条件返回之后
    // 字典: "1" -> 授权通过, "0" -> 授权失败
    const auditResultMapper = useMemo(
        () =>
            createStatusMapper<string>(
                { "1": "success", "0": "danger" },
                (status: string) => {
                    return auditResultDict?.getLabel(status) || status;
                },
            ),
        [auditResultDict],
    );

    if (!audit) return null;

    const toggleDetail = (detailId: number) => {
        const newExpanded = new Set(expandedDetails);
        if (newExpanded.has(detailId)) {
            newExpanded.delete(detailId);
        } else {
            newExpanded.add(detailId);
        }
        setExpandedDetails(newExpanded);
    };

    return (
        <Drawer open={open} onOpenChange={onOpenChange}>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>RBAC审计详情 #{audit.audit.id}</DrawerTitle>
                </DrawerHeader>
                <div className="space-y-6 mt-6">
                    {/* 审计主信息 */}
                    <div className="bg-muted p-4 rounded-lg">
                        <h3 className="font-semibold mb-3 flex items-center gap-2">
                            <Shield className={cn("h-4 w-4")} />
                            审计信息
                        </h3>
                        <div className="grid grid-cols-3 gap-4">
                            <div>
                                <label className="text-sm font-medium text-muted-foreground">
                                    审计结果
                                </label>
                                <div className="mt-1">
                                    <Badge
                                        variant="secondary"
                                        className={auditResultMapper.getClass(audit.audit.check_result ?? "")}
                                    >
                                        {auditResultMapper.getText(audit.audit.check_result ?? "")}
                                    </Badge>
                                </div>
                            </div>
                            <div>
                                <label className="text-sm font-medium text-muted-foreground">
                                    审计时间
                                </label>
                                <div className="mt-1 text-sm">
                                    {formatTime(
                                        audit.audit.add_time,
                                        TIME_STYLE.ABSOLUTE_TEXT,
                                    )}
                                </div>
                            </div>
                            <div>
                                <label className="text-sm font-medium text-muted-foreground">
                                    用户ID
                                </label>
                                <div className="mt-1 text-sm">{audit.audit.user_id}</div>
                            </div>
                            <div>
                                <label className="text-sm font-medium text-muted-foreground">
                                    用户IP
                                </label>
                                <div className="mt-1">
                                    <CopyableText
                                        value={audit.audit.user_ip || ""}
                                        className={cn("text-sm")}
                                    />
                                </div>
                            </div>
                            <div>
                                <label className="text-sm font-medium text-muted-foreground">
                                    设备名称
                                </label>
                                <div className="mt-1 text-xs text-muted-foreground truncate max-w-[200px]" title={audit.audit.device_name || ""}>
                                    {audit.audit.device_name || "-"}
                                </div>
                            </div>
                            <div>
                                <label className="text-sm font-medium text-muted-foreground">
                                    设备ID
                                </label>
                                <div className="mt-1">
                                    <CopyableText
                                        value={audit.audit.device_id || ""}
                                        className={cn("text-sm font-mono")}
                                    />
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* 详细审计数据 */}
                    <div>
                        <h3 className="font-semibold mb-3">
                            权限检查详情 ({audit.detail.length}项)
                        </h3>
                        <div className="space-y-2">
                            {audit.detail.map((detail) => (
                                <Collapsible
                                    key={detail.id || 0}
                                    open={expandedDetails.has(detail.id || 0)}
                                    onOpenChange={() => toggleDetail(detail.id || 0)}
                                >
                                    <CollapsibleTrigger asChild>
                                        <Button
                                            variant="outline"
                                            className={cn("w-full justify-between p-3 h-auto")}
                                        >
                                            <div className="flex items-center gap-3">
                                                <Badge
                                                    variant="secondary"
                                                    className={auditResultMapper.getClass(detail.check_result ?? "")}
                                                >
                                                    {auditResultMapper.getText(detail.check_result ?? "")}
                                                </Badge>
                                                <span className="text-sm">
                                                    {detail.res_type} - {detail.op_key}
                                                </span>
                                                <span className="text-xs text-muted-foreground">
                                                    {formatTime(
                                                        detail.add_time ?? null,
                                                        TIME_STYLE.RELATIVE_TEXT,
                                                    )}
                                                </span>
                                            </div>
                                            {expandedDetails.has(detail.id || 0) ? (
                                                <ChevronDown className={cn("h-4 w-4")} />
                                            ) : (
                                                <ChevronRight className={cn("h-4 w-4")} />
                                            )}
                                        </Button>
                                    </CollapsibleTrigger>
                                    <CollapsibleContent className={cn("mt-2")}>
                                        <div className="bg-muted p-4 rounded-lg">
                                            <div className="grid grid-cols-2 gap-4">
                                                <div>
                                                    <label className="text-sm font-medium text-muted-foreground">
                                                        资源类型
                                                    </label>
                                                    <div className="mt-1 text-sm">{detail.res_type}</div>
                                                </div>
                                                <div>
                                                    <label className="text-sm font-medium text-muted-foreground">
                                                        操作键
                                                    </label>
                                                    <div className="mt-1 text-sm">{detail.op_key}</div>
                                                </div>
                                                <div>
                                                    <label className="text-sm font-medium text-muted-foreground">
                                                        审计结果
                                                    </label>
                                                    <div className="mt-1">
                                                        <Badge
                                                            variant="secondary"
                                                            className={auditResultMapper.getClass(detail.check_result ?? "")}
                                                        >
                                                            {auditResultMapper.getText(detail.check_result ?? "")}
                                                        </Badge>
                                                    </div>
                                                </div>
                                                <div>
                                                    <label className="text-sm font-medium text-muted-foreground">
                                                        资源授权
                                                    </label>
                                                    <div className="mt-1 text-sm">{detail.res_auth}</div>
                                                </div>
                                                <div>
                                                    <label className="text-sm font-medium text-muted-foreground">
                                                        角色包含
                                                    </label>
                                                    <div className="mt-1">
                                                        <Badge
                                                            variant={
                                                                detail.is_role_include === "1"
                                                                    ? "default"
                                                                    : "secondary"
                                                            }
                                                        >
                                                            {detail.is_role_include === "1" ? "是" : "否"}
                                                        </Badge>
                                                    </div>
                                                </div>
                                                <div>
                                                    <label className="text-sm font-medium text-muted-foreground">
                                                        添加时间
                                                    </label>
                                                    <div className="mt-1 text-sm">
                                                        {formatTime(
                                                            detail.add_time ?? null,
                                                            TIME_STYLE.ABSOLUTE_TEXT,
                                                        )}
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </CollapsibleContent>
                                </Collapsible>
                            ))}
                        </div>
                    </div>
                </div>
            </DrawerContent>
        </Drawer>
    );
}
