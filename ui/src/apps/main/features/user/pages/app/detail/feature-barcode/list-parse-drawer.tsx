import { UserBarcodeParseRecordItemType } from '@shared/apis/user/barcode';
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import {
    Drawer,
    DrawerContent,
    DrawerDescription,
    DrawerHeader,
    DrawerTitle,
} from "@apps/main/components/local/drawer";
import { Textarea } from "@shared/components/ui/textarea";
import { useToast } from "@shared/contexts/toast-context";
import { type TypedDictData } from "@apps/main/hooks/use-dict-data";
import { cn, formatTime, TIME_STYLE } from "@shared/lib/utils";
import { createCopyWithToast } from "@shared/lib/utils/copy-utils";
import { Copy } from "lucide-react";
import React from "react";

interface BarcodeParseRecordDetailDrawerProps {
    record: UserBarcodeParseRecordItemType;
    dictData: TypedDictData<["user_barcode"]>;
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function BarcodeParseRecordDetailDrawer({
    record,
    dictData,
    open,
    onOpenChange,
}: BarcodeParseRecordDetailDrawerProps) {
    const toast = useToast();
    const copyWithToast = React.useMemo(
        () => createCopyWithToast(
            (message) => toast.success(message),
            (message) => toast.error(message)
        ),
        [toast]
    );

    const barcodeTypeLabel = dictData.barcode_type?.getLabel(record.bar_type) ?? record.bar_type;
    const parseStatusLabel = dictData.parse_status?.getLabel(String(record.status)) ?? String(record.status);
    const isSuccess = String(record.status) === "1";

    return (
        <Drawer open={open} onOpenChange={onOpenChange}>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>解析记录详情</DrawerTitle>
                    <DrawerDescription>记录 ID: {record.id}</DrawerDescription>
                </DrawerHeader>

                <div className="space-y-4 mt-6">
                    {/* 基础信息 */}
                    <div className="rounded-lg border p-4 space-y-3 bg-muted/50">
                        <h3 className="font-semibold text-sm">基础信息</h3>
                        <div className="grid grid-cols-2 gap-3 text-sm">
                            <div>
                                <span className="text-muted-foreground">记录 ID:</span>
                                <span className="ml-2 font-mono">{record.id}</span>
                            </div>
                            <div>
                                <span className="text-muted-foreground">应用 ID:</span>
                                <span className="ml-2 font-mono">{record.app_id}</span>
                            </div>
                            <div>
                                <span className="text-muted-foreground">条码类型:</span>
                                <span className="ml-2 font-mono">{barcodeTypeLabel}</span>
                            </div>
                            <div>
                                <span className="text-muted-foreground">解析时间:</span>
                                <span className="ml-2 text-xs">
                                    {formatTime(record.create_time, TIME_STYLE.ABSOLUTE_TEXT)}
                                </span>
                            </div>
                        </div>
                    </div>

                    {/* 解析状态 */}
                    <div className="rounded-lg border p-4 space-y-3">
                        <h3 className="font-semibold text-sm">解析状态</h3>
                        <div className="flex items-center gap-2">
                            <Badge className={cn(isSuccess ? "bg-green-600 hover:bg-green-700" : "bg-red-600 hover:bg-red-700")}>
                                {parseStatusLabel}
                            </Badge>
                            {!isSuccess && record.error && (
                                <span className="text-sm text-destructive">{record.error}</span>
                            )}
                        </div>
                    </div>

                    {/* 解析结果 */}
                    {isSuccess && (
                        <div className="rounded-lg border p-4 space-y-3">
                            <h3 className="font-semibold text-sm">解析结果</h3>
                            <div className="space-y-2">
                                <div>
                                    <label className="text-xs text-muted-foreground block mb-1.5">
                                        解析内容
                                    </label>
                                    <Textarea
                                        readOnly
                                        value={record.text}
                                        onDoubleClick={(e) => {
                                            const target = e.currentTarget as HTMLTextAreaElement;
                                            target.select();
                                        }}
                                        className="h-32 font-mono resize-none"
                                    />
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        className="mt-2"
                                        onClick={() => copyWithToast(record.text, "解析内容已复制")}
                                    >
                                        <Copy className="h-4 w-4 mr-2" />
                                        复制内容
                                    </Button>
                                </div>
                            </div>
                        </div>
                    )}

                    {/* 错误信息 */}
                    {!isSuccess && record.error && (
                        <div className="rounded-lg border p-4 space-y-3 bg-destructive/5">
                            <h3 className="font-semibold text-sm text-destructive">错误信息</h3>
                            <div className="bg-background p-3 rounded text-sm font-mono break-all max-h-40 overflow-y-auto text-destructive">
                                {record.error}
                            </div>
                        </div>
                    )}

                    {/* 哈希值 */}
                    <div className="rounded-lg border p-4 space-y-3 bg-muted/30">
                        <h3 className="font-semibold text-sm">哈希值</h3>
                        <div className="bg-background p-3 rounded text-xs font-mono text-muted-foreground break-words whitespace-pre-wrap word-break">
                            {record.hash}
                        </div>
                    </div>
                </div>
            </DrawerContent>
        </Drawer>
    );
}
