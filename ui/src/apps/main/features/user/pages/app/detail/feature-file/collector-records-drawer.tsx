import {
    Drawer,
    DrawerContent,
    DrawerDescription,
    DrawerHeader,
    DrawerTitle,
} from "@apps/main/components/local/drawer";
import { PagePagination, useCountNumManager } from "@apps/main/lib/pagination-utils";
import { createStatusMapper } from "@apps/main/lib/status-utils";
import {
    userCollectorRecordList,
    type CollectorRecordItemType,
    type CollectorScriptItemType,
} from "@shared/apis/user/collector";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { CenteredLoading } from "@shared/components/custom/page-placeholder/centered-loading";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import { cn, formatTime, getQueryResponseData, TIME_STYLE } from "@shared/lib/utils";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { FileText, FolderOpen } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { CollectorRecordFilesDrawer } from "./collector-record-files-drawer";
import { CollectorRecordLogsDrawer } from "./collector-record-logs-drawer";

// 采集记录状态映射
const recordStatusMapper = createStatusMapper(
    {
        1: "info",      // 等待中
        2: "warning",   // 执行中
        3: "success",   // 完成
        4: "danger",    // 失败
        5: "neutral",   // 取消
    },
    (status) => {
        switch (status) {
            case 1: return "等待中";
            case 2: return "执行中";
            case 3: return "完成";
            case 4: return "失败";
            case 5: return "取消";
            default: return String(status);
        }
    },
);

interface CollectorRecordsDrawerProps {
    appId: number;
    script: CollectorScriptItemType;
    isOpen: boolean;
    onOpenChange: (open: boolean) => void;
}

export function CollectorRecordsDrawer({
    appId,
    script,
    isOpen,
    onOpenChange,
}: CollectorRecordsDrawerProps) {
    const queryClient = useQueryClient();
    const [page, setPage] = useState(1);
    const pageSize = 10;

    // 记录关联文件/日志抽屉状态
    const [recordFilesOpen, setRecordFilesOpen] = useState(false);
    const [recordLogsOpen, setRecordLogsOpen] = useState(false);
    const [selectedRecord, setSelectedRecord] = useState<CollectorRecordItemType | null>(null);

    const countNumManager = useCountNumManager({});
    const { reset: resetCountNum } = countNumManager;

    useEffect(() => {
        setPage(1);
        resetCountNum();
    }, [script.id, resetCountNum]);

    const { data: recordsData, isSuccess, isLoading, isError, error } = useQuery({
        queryKey: ["collectorRecordList", appId, script.id, page],
        queryFn: ({ signal }) =>
            userCollectorRecordList(
                {
                    app_id: appId,
                    script_id: script.id,
                    limit: {
                        pos: (page - 1) * pageSize || null,
                        limit: pageSize,
                        forward: true,
                    },
                    count_num: countNumManager.getCountNum(),
                },
                { signal }
            ),
        enabled: isOpen,
    });

    isSuccess && countNumManager.handlePageQueryResult(recordsData);

    const records = getQueryResponseData<CollectorRecordItemType[]>(recordsData, []);

    const refreshData = useCallback(() => {
        queryClient.refetchQueries({ queryKey: ["collectorRecordList", appId, script.id] });
    }, [queryClient, appId, script.id]);

    const handleOpenChange = (open: boolean) => {
        onOpenChange(open);
        if (!open) {
            setPage(1);
        }
    };

    return (
        <>
            <Drawer open={isOpen} onOpenChange={handleOpenChange}>
                <DrawerContent>
                    <DrawerHeader>
                        <DrawerTitle>采集记录</DrawerTitle>
                        <DrawerDescription className={cn("space-y-1")}>
                            <div>脚本ID: {script.id}</div>
                            <div className="flex items-center gap-1.5">
                                <span>脚本名称:</span>
                                <span className="font-medium">{script.name}</span>
                            </div>
                        </DrawerDescription>
                    </DrawerHeader>

                    <div className="mt-6 space-y-4 flex flex-col flex-1 min-h-0">
                        <div className="flex-1 overflow-y-auto space-y-3">
                            {isLoading ? (
                                <CenteredLoading variant="content" iconSize="md" />
                            ) : isError ? (
                                <CenteredError error={error} variant="content" onReset={refreshData} />
                            ) : records.length === 0 ? (
                                <div className="text-center text-muted-foreground py-8">暂无采集记录</div>
                            ) : (
                                records.map((record) => (
                                    <div key={record.id} className="border rounded-lg p-4 space-y-2 bg-card">
                                        <div className="flex items-center justify-between">
                                            <span className="text-xs font-mono text-muted-foreground">#{record.id}</span>
                                            <Badge className={cn(recordStatusMapper.getClass(record.status))}>
                                                {recordStatusMapper.getText(record.status)}
                                            </Badge>
                                        </div>

                                        <div className="grid grid-cols-2 gap-2">
                                            <div className="flex items-center gap-1.5">
                                                <span className="text-xs text-muted-foreground">请求ID:</span>
                                                <span className="text-xs font-mono truncate" title={record.request_id}>
                                                    {record.request_id || "-"}
                                                </span>
                                            </div>
                                            <div className="flex items-center gap-1.5">
                                                <span className="text-xs text-muted-foreground">耗时:</span>
                                                <span className="text-xs font-medium">
                                                    {record.elapsed_ms > 0 ? `${record.elapsed_ms} ms` : "-"}
                                                </span>
                                            </div>
                                        </div>

                                        {record.exec_params && (
                                            <div className="flex gap-1.5">
                                                <span className="text-xs text-muted-foreground whitespace-nowrap">参数:</span>
                                                <span className="text-xs break-words flex-1 font-mono bg-muted/50 rounded px-2 py-1">
                                                    {record.exec_params}
                                                </span>
                                            </div>
                                        )}

                                        {record.error_message && (
                                            <div className="flex gap-1.5">
                                                <span className="text-xs text-muted-foreground whitespace-nowrap">错误:</span>
                                                <span className="text-xs break-words flex-1 text-destructive">
                                                    {record.error_message}
                                                </span>
                                            </div>
                                        )}

                                        <div className="flex items-center gap-1.5">
                                            <span className="text-xs text-muted-foreground">时间:</span>
                                            <span className="text-xs">
                                                {record.add_time ? formatTime(record.add_time, TIME_STYLE.ABSOLUTE_TEXT) : "-"}
                                            </span>
                                        </div>

                                        {/* 记录关联文件和日志按钮 */}
                                        <div className="flex items-center gap-2 pt-1 border-t">
                                            <Button
                                                variant="ghost"
                                                size="sm"
                                                className={cn("h-auto px-2 py-1")}
                                                title="查看关联文件"
                                                onClick={() => {
                                                    setSelectedRecord(record);
                                                    setRecordFilesOpen(true);
                                                }}
                                            >
                                                <FolderOpen className="h-3 w-3" />
                                                <span className="text-xs ml-1">文件</span>
                                            </Button>
                                            <Button
                                                variant="ghost"
                                                size="sm"
                                                className={cn("h-auto px-2 py-1")}
                                                title="查看关联日志"
                                                onClick={() => {
                                                    setSelectedRecord(record);
                                                    setRecordLogsOpen(true);
                                                }}
                                            >
                                                <FileText className="h-3 w-3" />
                                                <span className="text-xs ml-1">日志</span>
                                            </Button>
                                        </div>
                                    </div>
                                ))
                            )}
                        </div>

                        <div className="flex justify-end">
                            <PagePagination
                                currentPage={page}
                                pageSize={pageSize}
                                total={countNumManager.getTotal() ?? 0}
                                loading={isLoading}
                                onChange={(newPage) => setPage(newPage)}
                                showTotal={false}
                                showPageSize={false}
                                showSizeCount={5}
                            />
                        </div>
                    </div>
                </DrawerContent>
            </Drawer>

            {/* 记录关联文件抽屉 */}
            {selectedRecord && (
                <CollectorRecordFilesDrawer
                    appId={appId}
                    record={selectedRecord}
                    isOpen={recordFilesOpen}
                    onOpenChange={setRecordFilesOpen}
                />
            )}

            {/* 记录关联日志抽屉 */}
            {selectedRecord && (
                <CollectorRecordLogsDrawer
                    appId={appId}
                    record={selectedRecord}
                    isOpen={recordLogsOpen}
                    onOpenChange={setRecordLogsOpen}
                />
            )}
        </>
    );
}
