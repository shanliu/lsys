import {
    Drawer,
    DrawerContent,
    DrawerDescription,
    DrawerHeader,
    DrawerTitle,
} from "@apps/main/components/local/drawer";
import { PagePagination, useCountNumManager } from "@apps/main/lib/pagination-utils";
import {
    userCollectorRecordLogList,
    type CollectorLogItemType,
    type CollectorRecordItemType,
} from "@shared/apis/user/collector";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { CenteredLoading } from "@shared/components/custom/page-placeholder/centered-loading";
import { Badge } from "@shared/components/ui/badge";
import { cn, formatTime, getQueryResponseData, TIME_STYLE } from "@shared/lib/utils";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useState } from "react";

// 日志级别映射
const logLevelMap: Record<number, { label: string; variant: string }> = {
    1: { label: "DEBUG", variant: "bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-300" },
    2: { label: "INFO", variant: "bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300" },
    3: { label: "WARN", variant: "bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300" },
    4: { label: "ERROR", variant: "bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300" },
};

interface CollectorRecordLogsDrawerProps {
    appId: number;
    record: CollectorRecordItemType;
    isOpen: boolean;
    onOpenChange: (open: boolean) => void;
}

export function CollectorRecordLogsDrawer({
    appId,
    record,
    isOpen,
    onOpenChange,
}: CollectorRecordLogsDrawerProps) {
    const queryClient = useQueryClient();
    const [page, setPage] = useState(1);
    const pageSize = 20;

    const countNumManager = useCountNumManager({});
    const { reset: resetCountNum } = countNumManager;

    useEffect(() => {
        setPage(1);
        resetCountNum();
    }, [record.request_id, resetCountNum]);

    const { data: logsData, isSuccess, isLoading, isError, error } = useQuery({
        queryKey: ["collectorRecordLogList", appId, record.request_id, page],
        queryFn: ({ signal }) =>
            userCollectorRecordLogList(
                {
                    app_id: appId,
                    request_id: record.request_id,
                    page: {
                        page,
                        limit: pageSize,
                    },
                    count_num: countNumManager.getCountNum(),
                },
                { signal }
            ),
        enabled: isOpen,
    });

    isSuccess && countNumManager.handlePageQueryResult(logsData);

    const logs = getQueryResponseData<CollectorLogItemType[]>(logsData, []);

    const refreshData = useCallback(() => {
        queryClient.refetchQueries({ queryKey: ["collectorRecordLogList", appId, record.request_id] });
    }, [queryClient, appId, record.request_id]);

    const handleOpenChange = (open: boolean) => {
        onOpenChange(open);
        if (!open) {
            setPage(1);
        }
    };

    const getLevelInfo = (level: number) => {
        return logLevelMap[level] || { label: String(level), variant: "bg-gray-100 text-gray-700" };
    };

    return (
        <Drawer open={isOpen} onOpenChange={handleOpenChange}>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>记录关联日志</DrawerTitle>
                    <DrawerDescription className={cn("space-y-1")}>
                        <div className="flex items-center gap-1.5">
                            <span>请求ID:</span>
                            <span className="font-mono text-xs">{record.request_id}</span>
                        </div>
                    </DrawerDescription>
                </DrawerHeader>

                <div className="mt-6 space-y-4 flex flex-col flex-1 min-h-0">
                    <div className="flex-1 overflow-y-auto space-y-2">
                        {isLoading ? (
                            <CenteredLoading variant="content" iconSize="md" />
                        ) : isError ? (
                            <CenteredError error={error} variant="content" onReset={refreshData} />
                        ) : logs.length === 0 ? (
                            <div className="text-center text-muted-foreground py-8">暂无关联日志</div>
                        ) : (
                            logs.map((log) => {
                                const levelInfo = getLevelInfo(log.level);
                                return (
                                    <div key={log.id} className="border rounded-lg p-3 space-y-1.5 bg-card">
                                        <div className="flex items-center justify-between gap-2">
                                            <div className="flex items-center gap-2">
                                                <Badge className={cn("text-xs font-mono", levelInfo.variant)}>
                                                    {levelInfo.label}
                                                </Badge>
                                            </div>
                                            <span className="text-xs text-muted-foreground flex-shrink-0">
                                                {log.add_time ? formatTime(log.add_time, TIME_STYLE.ABSOLUTE_TEXT) : "-"}
                                            </span>
                                        </div>

                                        <div className="text-xs break-words font-mono bg-muted/30 rounded px-2 py-1.5 whitespace-pre-wrap">
                                            {log.message}
                                        </div>
                                    </div>
                                );
                            })
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
    );
}
