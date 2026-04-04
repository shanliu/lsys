import {
    Drawer,
    DrawerContent,
    DrawerDescription,
    DrawerHeader,
    DrawerTitle,
} from "@apps/main/components/local/drawer";
import { useDictData } from "@apps/main/hooks/use-dict-data";
import { createStatusMapper } from "@apps/main/lib/status-utils";
import { CursorPagination, useLimitCountNum } from "@apps/main/lib/pagination-utils";
import {
    userCollectorLogList,
    type CollectorLogItemType,
    type CollectorScriptItemType,
} from "@shared/apis/user/collector";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { CenteredLoading } from "@shared/components/custom/page-placeholder/centered-loading";
import { Badge } from "@shared/components/ui/badge";
import { cn, formatTime, getQueryResponseCursor, getQueryResponseData, TIME_STYLE } from "@shared/lib/utils";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useState } from "react";

// 日志级别颜色映射（使用统一状态管理）
const logLevelMapper = createStatusMapper({
    1: 'neutral',   // DEBUG
    2: 'info',      // INFO
    3: 'warning',   // WARN
    4: 'danger',    // ERROR
    10: 'info',     // SYSTEM
});

interface CollectorLogsDrawerProps {
    appId: number;
    script: CollectorScriptItemType;
    isOpen: boolean;
    onOpenChange: (open: boolean) => void;
}

export function CollectorLogsDrawer({
    appId,
    script,
    isOpen,
    onOpenChange,
}: CollectorLogsDrawerProps) {
    const queryClient = useQueryClient();
    const [cursorParams, setCursorParams] = useState<{ pos: number | null; forward: boolean }>({ pos: null, forward: true });
    const pageSize = 20;
    const { dictData: collectorDict } = useDictData(['user_collector'] as const);

    const countNumManager = useLimitCountNum({});
    const { reset: resetCountNum } = countNumManager;

    useEffect(() => {
        setCursorParams({ pos: null, forward: true });
        resetCountNum();
    }, [script.id, resetCountNum]);

    const { data: logsData, isSuccess, isLoading, isError, error } = useQuery({
        queryKey: ["collectorLogList", appId, script.id, cursorParams.pos, cursorParams.forward],
        queryFn: ({ signal }) =>
            userCollectorLogList(
                {
                    app_id: appId,
                    script_id: script.id,
                    limit: {
                        pos: cursorParams.pos,
                        limit: pageSize,
                        forward: cursorParams.forward,
                    },
                    count_num: countNumManager.getCountNum(),
                },
                { signal }
            ),
        enabled: isOpen,
    });

    isSuccess && countNumManager.handleQueryResult(logsData);
    const cursorData = getQueryResponseCursor(logsData);

    const logs = getQueryResponseData<CollectorLogItemType[]>(logsData, []);

    const refreshData = useCallback(() => {
        queryClient.refetchQueries({ queryKey: ["collectorLogList", appId, script.id] });
    }, [queryClient, appId, script.id]);

    const localSearchGo = useCallback((param: { pos: number | null; forward?: boolean }) => {
        setCursorParams({ pos: param.pos ?? null, forward: param.forward ?? true });
    }, []);

    const handleOpenChange = (open: boolean) => {
        onOpenChange(open);
        if (!open) {
            setCursorParams({ pos: null, forward: true });
        }
    };

    const getLevelInfo = (level: number) => {
        const label = collectorDict?.log_level?.getLabel(String(level), '') ?? '';
        const cls = logLevelMapper.getClass(level);
        return { label, cls };
    };

    return (
        <Drawer open={isOpen} onOpenChange={handleOpenChange}>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>运行日志</DrawerTitle>
                    <DrawerDescription className={cn("space-y-1")}>
                        <div>脚本ID: {script.id}</div>
                        <div className="flex items-center gap-1.5">
                            <span>脚本名称:</span>
                            <span className="font-medium">{script.name}</span>
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
                            <div className="text-center text-muted-foreground py-8">暂无运行日志</div>
                        ) : (
                            logs.map((log) => {
                                const levelInfo = getLevelInfo(log.level);
                                return (
                                    <div key={log.id} className="border rounded-lg p-3 space-y-1.5 bg-card">
                                        <div className="flex items-center justify-between gap-2">
                                            <div className="flex items-center gap-2">
                                                <Badge className={cn("text-xs font-mono", levelInfo.cls)}>
                                                    {levelInfo.label}
                                                </Badge>
                                                <span className="text-xs font-mono text-muted-foreground" title={log.request_id}>
                                                    {log.request_id ? log.request_id.substring(0, 12) + "..." : "-"}
                                                </span>
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
                        <CursorPagination
                            limit={pageSize}
                            cursorData={cursorData}
                            searchGo={localSearchGo}
                            totalInfo={countNumManager.getTotalInfo()}
                            currentPageSize={logs.length}
                            loading={isLoading}
                            showPageSize={false}
                        />
                    </div>
                </div>
            </DrawerContent>
        </Drawer>
    );
}
