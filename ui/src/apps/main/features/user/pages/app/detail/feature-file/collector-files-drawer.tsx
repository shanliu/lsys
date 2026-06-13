import {
    Drawer,
    DrawerContent,
    DrawerDescription,
    DrawerHeader,
    DrawerTitle,
} from "@apps/main/components/local/drawer";
import { PostDownload } from "@apps/main/components/local/post-download";
import { CursorPagination, useLimitCountNum } from "@apps/main/lib/pagination-utils";
import {
    userCollectorFileList,
    type CollectorFileItemType,
    type CollectorScriptItemType,
} from "@shared/apis/user/collector";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { CenteredLoading } from "@shared/components/custom/page-placeholder/centered-loading";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import { cn, formatFileSize, formatTime, getQueryResponseCursor, getQueryResponseData, TIME_STYLE } from "@shared/lib/utils";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Download } from "lucide-react";
import { useCallback, useEffect, useState } from "react";

interface CollectorFilesDrawerProps {
    appId: number;
    script: CollectorScriptItemType;
    isOpen: boolean;
    onOpenChange: (open: boolean) => void;
}

export function CollectorFilesDrawer({
    appId,
    script,
    isOpen,
    onOpenChange,
}: CollectorFilesDrawerProps) {
    const queryClient = useQueryClient();
    const [cursorParams, setCursorParams] = useState<{ pos: number | null; forward: boolean }>({ pos: null, forward: true });
    const pageSize = 10;

    const countNumManager = useLimitCountNum({});
    const { reset: resetCountNum } = countNumManager;

    useEffect(() => {
        setCursorParams({ pos: null, forward: true });
        resetCountNum();
    }, [script.id, resetCountNum]);

    const { data: filesData, isSuccess, isLoading, isError, error } = useQuery({
        queryKey: ["collectorFileList", appId, script.id, cursorParams.pos, cursorParams.forward],
        queryFn: ({ signal }) =>
            userCollectorFileList(
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

    isSuccess && countNumManager.handleQueryResult(filesData);
    const cursorData = getQueryResponseCursor(filesData);

    const files = getQueryResponseData<CollectorFileItemType[]>(filesData, []);

    const refreshData = useCallback(() => {
        queryClient.refetchQueries({ queryKey: ["collectorFileList", appId, script.id] });
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

    return (
        <Drawer open={isOpen} onOpenChange={handleOpenChange}>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>采集文件</DrawerTitle>
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
                        ) : files.length === 0 ? (
                            <div className="text-center text-muted-foreground py-8">暂无采集文件</div>
                        ) : (
                            files.map((file) => (
                                <div key={file.file_id} className="border rounded-lg p-4 space-y-2 bg-card">
                                    <div className="flex items-center justify-between">
                                        <div className="flex items-center gap-2 min-w-0 flex-1">
                                            <span className="text-sm font-medium truncate" title={file.file_name}>
                                                {file.file_name || "-"}
                                            </span>
                                        </div>
                                        {file.file_key && (
                                            <PostDownload
                                                url="/api/user/app_file/read"
                                                body={{ key: file.file_key }}
                                            >
                                                {({ onClick, isLoading }) => (
                                                    <Button
                                                        variant="ghost"
                                                        size="sm"
                                                        className="h-auto p-1 flex-shrink-0"
                                                        title="下载文件"
                                                        onClick={onClick}
                                                        disabled={isLoading}
                                                    >
                                                        <Download className="h-4 w-4" />
                                                    </Button>
                                                )}
                                            </PostDownload>
                                        )}
                                    </div>

                                    <div className="grid grid-cols-2 gap-2">
                                        <div className="flex items-center gap-1.5">
                                            <span className="text-xs text-muted-foreground">大小:</span>
                                            <span className="text-xs font-medium">{formatFileSize(file.file_size)}</span>
                                        </div>
                                        <div className="flex items-center gap-1.5">
                                            <span className="text-xs text-muted-foreground">类型:</span>
                                            <span className="text-xs">{file.content_type || "-"}</span>
                                        </div>
                                    </div>

                                    <div className="grid grid-cols-2 gap-2">
                                        <div className="flex items-center gap-1.5">
                                            <span className="text-xs text-muted-foreground">存储:</span>
                                            <span className="text-xs">{file.storage_type || "-"}</span>
                                        </div>
                                        <div className="flex items-center gap-1.5">
                                            <span className="text-xs text-muted-foreground">MD5:</span>
                                            <span className="text-xs font-mono truncate" title={file.file_md5}>
                                                {file.file_md5 ? file.file_md5.substring(0, 12) + "..." : "-"}
                                            </span>
                                        </div>
                                    </div>

                                    {file.tags && file.tags.length > 0 && (
                                        <div className="flex items-center gap-1.5 flex-wrap">
                                            <span className="text-xs text-muted-foreground">标签:</span>
                                            {file.tags.map((tag, idx) => (
                                                <Badge key={idx} variant="secondary" className="text-xs">
                                                    {tag.tag_name}
                                                </Badge>
                                            ))}
                                        </div>
                                    )}

                                    <div className="flex items-center gap-1.5">
                                        <span className="text-xs text-muted-foreground">时间:</span>
                                        <span className="text-xs">
                                            {file.add_time ? formatTime(file.add_time, TIME_STYLE.ABSOLUTE_TEXT) : "-"}
                                        </span>
                                    </div>
                                </div>
                            ))
                        )}
                    </div>

                    <div className="flex justify-end">
                        <CursorPagination
                            limit={pageSize}
                            cursorData={cursorData}
                            searchGo={localSearchGo}
                            totalInfo={countNumManager.getTotalInfo()}
                            currentPageSize={files.length}
                            loading={isLoading}
                            showPageSize={false}
                        />
                    </div>
                </div>
            </DrawerContent>
        </Drawer>
    );
}
