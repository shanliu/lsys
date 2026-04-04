import { Drawer, DrawerContent, DrawerHeader, DrawerTitle } from '@apps/main/components/local/drawer';
import { useDictData } from '@apps/main/hooks/use-dict-data';
import { createStatusMapper } from '@apps/main/lib/status-utils';
import {
    EXPORT_TYPE_USER_APP_REQUEST,
    EXPORT_TYPE_USER_FILE_CHUNK,
    EXPORT_TYPE_USER_FILE_LIST,
    EXPORT_TYPE_USER_FILE_LOG,
    userFileExportList,
    type UserFileExportTaskType,
} from '@shared/apis/user/file';
import { Button } from '@shared/components/ui/button';
import { Badge } from '@shared/components/ui/badge';
import { cn, formatTime, TIME_STYLE } from '@shared/lib/utils';
import { useQuery } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { CheckCircle2, Clock, Download, ExternalLink, Loader2, XCircle } from 'lucide-react';

interface FileExportTasksDrawerProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    appId: number;
}

const EXPORT_STATUS_ICON: Record<number, typeof Clock> = {
    1: Clock,
    2: Loader2,
    3: CheckCircle2,
    4: XCircle,
    5: XCircle,
} as const;

const EXPORT_TYPE_LABELS: Record<string, string> = {
    [EXPORT_TYPE_USER_FILE_LIST]: '文件列表',
    [EXPORT_TYPE_USER_FILE_LOG]: '文件日志',
    [EXPORT_TYPE_USER_FILE_CHUNK]: '文件分片',
    [EXPORT_TYPE_USER_APP_REQUEST]: '应用请求',
};

export function FileExportTasksDrawer({
    open,
    onOpenChange,
    appId,
}: FileExportTasksDrawerProps) {
    const navigate = useNavigate();
    const { dictData: exportDict } = useDictData(['user_export'] as const);
    const exportStatusMapper = createStatusMapper(
        { 1: 'neutral', 2: 'info', 3: 'success', 4: 'danger', 5: 'neutral' } as const,
        (status) => exportDict?.export_task_status?.getLabel(String(status), '') ?? '',
    );

    const { data, isLoading } = useQuery({
        queryKey: ['userFileExportList', appId],
        queryFn: () =>
            userFileExportList({
                app_id: appId,
                page: { page: 1, limit: 30 },
            }),
        enabled: open,
        refetchInterval: (query) => {
            const tasks = query.state.data?.response?.data ?? [];
            const hasPending = tasks.some((t) => t.status === 1 || t.status === 2);
            return hasPending ? 3000 : false;
        },
    });

    const tasks = data?.response?.data ?? [];

    // 解析 export_params JSON 并生成摘要
    const formatParamsSummary = (task: UserFileExportTaskType): string => {
        try {
            const p = JSON.parse(task.export_params || '{}');
            const parts: string[] = [];
            if (p.status != null) parts.push(`状态=${p.status}`);
            if (p.file_md5) parts.push(`MD5=${String(p.file_md5).slice(0, 8)}...`);
            if (p.source_url) parts.push(`来源URL`);
            if (p.url) parts.push(`本地URL`);
            return parts.length > 0 ? parts.join(' · ') : '全部';
        } catch {
            return task.export_params || '-';
        }
    };

    // 完成的文件任务：跳转到文件列表并按 tag export_{id} 过滤
    const handleViewFile = (task: UserFileExportTaskType) => {
        const tag = `export_${task.id}`;
        if (task.export_type === EXPORT_TYPE_USER_FILE_LIST ||
            task.export_type === EXPORT_TYPE_USER_FILE_LOG ||
            task.export_type === EXPORT_TYPE_USER_FILE_CHUNK) {
            navigate({
                to: '/user/app/$appId/features-file/list',
                params: { appId},
                search: { tag_name: tag },
            });
            onOpenChange(false);
        }
    };

    return (
        <Drawer open={open} onOpenChange={onOpenChange}>
            <DrawerContent>
                <DrawerHeader className="pb-4">
                    <DrawerTitle className="text-xl">导出任务</DrawerTitle>
                </DrawerHeader>

                <div className="px-4 pb-6 space-y-3 overflow-y-auto">
                    {isLoading ? (
                        <div className="flex items-center justify-center py-10 text-muted-foreground">
                            <Loader2 className="h-5 w-5 animate-spin mr-2" />
                            加载中...
                        </div>
                    ) : tasks.length === 0 ? (
                        <div className="flex flex-col items-center justify-center py-10 text-muted-foreground">
                            <Download className="h-10 w-10 mb-3 opacity-30" />
                            <p className="text-sm">暂无导出任务</p>
                            <p className="text-xs mt-1 opacity-70">在列表页按条件过滤后点击"导出"可创建任务</p>
                        </div>
                    ) : (
                        tasks.map((task) => {
                            const StatusIcon = EXPORT_STATUS_ICON[task.status] ?? Clock;
                            const isProcessing = task.status === 1 || task.status === 2;
                            const isDone = task.status === 3;
                            const typeLabel = EXPORT_TYPE_LABELS[task.export_type] ?? task.export_type;

                            return (
                                <div
                                    key={task.id}
                                    className={cn(
                                        'rounded-lg border p-3 space-y-2',
                                        isDone && 'border-green-200 bg-green-50/50 dark:border-green-900 dark:bg-green-950/20',
                                        task.status === 4 && 'border-destructive/30 bg-destructive/5',
                                    )}
                                >
                                    {/* ID + 类型 badge + 状态 + 时间 */}
                                    <div className="flex items-center justify-between gap-2">
                                        <div className="flex items-center gap-2 min-w-0 flex-wrap">
                                            <span className="text-xs text-muted-foreground flex-shrink-0">#{task.id}</span>
                                            <Badge variant="outline" className="text-xs px-1.5 py-0 flex-shrink-0">
                                                {typeLabel}
                                            </Badge>
                                            <Badge
                                                className={cn(
                                                    'text-xs flex items-center gap-1 flex-shrink-0',
                                                    exportStatusMapper.getClass(task.status),
                                                )}
                                            >
                                                <StatusIcon className={cn('h-3 w-3', isProcessing && 'animate-spin')} />
                                                {exportStatusMapper.getText(task.status)}
                                            </Badge>
                                        </div>
                                        <span className="text-xs text-muted-foreground flex-shrink-0">
                                            {task.add_time ? formatTime(task.add_time, TIME_STYLE.RELATIVE_ELEMENT) : '-'}
                                        </span>
                                    </div>

                                    {/* 过滤条件摘要 */}
                                    <div className="text-xs text-muted-foreground">
                                        条件：{formatParamsSummary(task)}
                                    </div>

                                    {/* 失败原因 */}
                                    {task.status === 4 && task.error_message && (
                                        <div className="text-xs text-destructive truncate">
                                            {task.error_message}
                                        </div>
                                    )}

                                    {/* 完成后：查看导出文件 */}
                                    {isDone && (
                                        <div className="flex justify-end pt-1">
                                            <Button
                                                size="sm"
                                                variant="default"
                                                className="h-7 text-xs"
                                                onClick={() => handleViewFile(task)}
                                            >
                                                <ExternalLink className="h-3 w-3 mr-1" />
                                                查看导出文件
                                            </Button>
                                        </div>
                                    )}
                                </div>
                            );
                        })
                    )}
                </div>
            </DrawerContent>
        </Drawer>
    );
}
