import { userFileFromUrl, type UserFileMappingResType } from '@shared/apis/user/file';
import { ContentDialog } from '@shared/components/custom/dialog/content-dialog';
import { ConfirmDialog } from '@shared/components/custom/dialog/confirm-dialog';
import { Button } from '@shared/components/ui/button';
import { Label } from '@shared/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@shared/components/ui/select';
import { Textarea } from '@shared/components/ui/textarea';
import { useToast } from '@shared/contexts/toast-context';
import { cn } from '@shared/lib/utils';
import { CheckCircle2, Download, Loader2, Pause, Play, XCircle } from 'lucide-react';
import { useMutation } from '@tanstack/react-query';
import React, { useCallback, useRef, useState } from 'react';

type DialogStage = 'input' | 'downloading' | 'paused' | 'done';

interface UrlRecord {
    url: string;
    status: 'pending' | 'downloading' | 'success' | 'error';
    message?: string;
    fileUserId?: number;
}

interface FileUrlDownloadDialogProps {
    children: React.ReactNode;
    appId: number;
    uploadConfig: UserFileMappingResType;
    onSuccess?: () => void;
}

// 从 max_upload_size 推导出合理的最大并发数范围
const MAX_CONCURRENCY_OPTIONS = [1, 2, 3, 4, 5, 6, 8, 10];

export function FileUrlDownloadDialog({
    children,
    appId,
    uploadConfig,
    onSuccess,
}: FileUrlDownloadDialogProps) {
    const { success: showSuccess, error: showError } = useToast();
    const [open, setOpen] = useState(false);
    const [stage, setStage] = useState<DialogStage>('input');
    const [urlText, setUrlText] = useState('');
    const [maxConcurrency, setMaxConcurrency] = useState(10);
    const [records, setRecords] = useState<UrlRecord[]>([]);
    const isPausedRef = useRef(false);
    const isAbortedRef = useRef(false);
    const downloadingRef = useRef(false);

    // React Query Mutation
    const downloadFileMutation = useMutation({
        mutationFn: (params: { app_id: number; source_url: string; max_concurrency: number }) =>
            userFileFromUrl(params),
    });

    const resetState = useCallback(() => {
        setStage('input');
        setUrlText('');
        setRecords([]);
        isPausedRef.current = false;
        isAbortedRef.current = false;
        downloadingRef.current = false;
    }, []);

    const handleOpenChange = useCallback((newOpen: boolean) => {
        if (!newOpen && (stage === 'downloading')) {
            return; // 下载中不允许直接关闭
        }
        if (!newOpen) {
            resetState();
        }
        setOpen(newOpen);
    }, [stage, resetState]);

    // 开始下载
    const startDownload = useCallback(async () => {
        const urls = urlText
            .split('\n')
            .map(u => u.trim())
            .filter(u => u.length > 0 && (u.startsWith('http://') || u.startsWith('https://')));

        if (urls.length === 0) {
            showError('请输入有效的 URL（以 http:// 或 https:// 开头）');
            return;
        }

        const initialRecords: UrlRecord[] = urls.map(url => ({
            url,
            status: 'pending',
        }));
        setRecords(initialRecords);
        setStage('downloading');
        isPausedRef.current = false;
        isAbortedRef.current = false;
        downloadingRef.current = true;

        await processRecords(initialRecords);
    }, [urlText, appId, maxConcurrency]);

    // 处理下载记录
    const processRecords = useCallback(async (currentRecords: UrlRecord[]) => {
        const updatedRecords = [...currentRecords];

        for (let i = 0; i < updatedRecords.length; i++) {
            if (isPausedRef.current || isAbortedRef.current) {
                downloadingRef.current = false;
                if (isAbortedRef.current) {
                    setStage('input');
                    setRecords([]);
                    setUrlText(updatedRecords.filter(r => r.status === 'pending').map(r => r.url).join('\n'));
                } else {
                    setStage('paused');
                    setRecords([...updatedRecords]);
                }
                return;
            }

            if (updatedRecords[i].status !== 'pending') continue;

            updatedRecords[i].status = 'downloading';
            setRecords([...updatedRecords]);

            try {
                // 使用 mutation 来处理下载请求
                const res = await downloadFileMutation.mutateAsync({
                    app_id: appId,
                    source_url: updatedRecords[i].url,
                    max_concurrency: maxConcurrency,
                });

                if (res.status && res.response) {
                    updatedRecords[i].status = 'success';
                    updatedRecords[i].fileUserId = res.response.file_user_id;
                    updatedRecords[i].message = '下载成功';
                } else {
                    updatedRecords[i].status = 'error';
                    updatedRecords[i].message = res.message || '下载失败';
                }
            } catch (err: any) {
                updatedRecords[i].status = 'error';
                updatedRecords[i].message = err?.data?.message || err?.message || '下载失败';
            }

            setRecords([...updatedRecords]);
        }

        downloadingRef.current = false;

        // 检查是否都完成了
        const allDone = updatedRecords.every(r => r.status === 'success' || r.status === 'error');
        if (allDone) {
            setStage('done');
            const successCount = updatedRecords.filter(r => r.status === 'success').length;
            if (successCount > 0) {
                showSuccess(`${successCount} 个文件下载成功`);
                onSuccess?.();
            }
        }
    }, [appId, maxConcurrency, showSuccess, onSuccess, downloadFileMutation]);

    // 暂停 / 继续
    const handlePause = useCallback(() => {
        isPausedRef.current = true;
    }, []);

    const handleResume = useCallback(async () => {
        setStage('downloading');
        isPausedRef.current = false;
        isAbortedRef.current = false;
        downloadingRef.current = true;
        await processRecords(records);
    }, [records, processRecords]);

    // 取消（退回输入框状态）
    const handleCancel = useCallback(() => {
        isPausedRef.current = true;
        isAbortedRef.current = true;
        if (!downloadingRef.current) {
            // 已经暂停，直接退回
            const pendingUrls = records.filter(r => r.status === 'pending').map(r => r.url).join('\n');
            setStage('input');
            setRecords([]);
            setUrlText(pendingUrls);
        }
    }, [records]);

    // 关闭
    const handleCloseDialog = useCallback(() => {
        isPausedRef.current = true;
        isAbortedRef.current = true;
        resetState();
        setOpen(false);
    }, [resetState]);

    const hasPending = records.some(r => r.status === 'pending');

    const content = (
        <div className="space-y-4">
            {stage === 'input' ? (
                <>
                    <div className="space-y-2">
                        <Label>URL 列表</Label>
                        <Textarea
                            value={urlText}
                            onChange={(e) => setUrlText(e.target.value)}
                            placeholder={'请输入要下载的文件 URL，每行一个\nhttps://example.com/file1.jpg\nhttps://example.com/file2.png'}
                            rows={8}
                            className="font-mono text-sm"
                        />
                        <p className="text-xs text-muted-foreground">
                            每行输入一个 URL，支持批量下载
                        </p>
                    </div>
                    <div className="flex items-center gap-3">
                        <Label className="flex-shrink-0">最大下载线程</Label>
                        <Select
                            value={String(maxConcurrency)}
                            onValueChange={(val) => setMaxConcurrency(Number(val))}
                        >
                            <SelectTrigger className="w-24">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                {MAX_CONCURRENCY_OPTIONS.map(n => (
                                    <SelectItem key={n} value={String(n)}>{n}</SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                </>
            ) : (
                <div className="space-y-2 max-h-[400px] overflow-y-auto">
                    {records.map((record, idx) => (
                        <div
                            key={idx}
                            className={cn(
                                "flex items-center gap-2 p-2 rounded border text-sm",
                                record.status === 'success' && "border-green-200 bg-green-50 dark:border-green-900 dark:bg-green-950",
                                record.status === 'error' && "border-red-200 bg-red-50 dark:border-red-900 dark:bg-red-950",
                                record.status === 'downloading' && "border-blue-200 bg-blue-50 dark:border-blue-900 dark:bg-blue-950",
                                record.status === 'pending' && "border-muted",
                            )}
                        >
                            <div className="flex-shrink-0">
                                {record.status === 'success' && <CheckCircle2 className="h-4 w-4 text-green-500" />}
                                {record.status === 'error' && <XCircle className="h-4 w-4 text-red-500" />}
                                {record.status === 'downloading' && <Loader2 className="h-4 w-4 text-blue-500 animate-spin" />}
                                {record.status === 'pending' && <Download className="h-4 w-4 text-muted-foreground" />}
                            </div>
                            <div className="flex-1 min-w-0">
                                <p className="truncate font-mono text-xs">{record.url}</p>
                                {record.message && (
                                    <p className={cn(
                                        "text-xs mt-0.5",
                                        record.status === 'success' ? "text-green-600 dark:text-green-400" : "text-red-600 dark:text-red-400"
                                    )}>
                                        {record.message}
                                    </p>
                                )}
                            </div>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );

    const footer = (closeDialog: () => void) => {
        if (stage === 'input') {
            return (
                <div className="flex gap-2 w-full justify-end">
                    <Button
                        onClick={startDownload}
                        disabled={!urlText.trim()}
                    >
                        <Download className="h-4 w-4 mr-1" />
                        下载
                    </Button>
                    <Button variant="outline" onClick={closeDialog}>
                        关闭
                    </Button>
                </div>
            );
        }

        if (stage === 'downloading') {
            return (
                <div className="flex gap-2 w-full justify-end">
                    {hasPending && (
                        <Button variant="outline" onClick={handlePause}>
                            <Pause className="h-4 w-4 mr-1" />
                            暂停
                        </Button>
                    )}
                    <Button variant="outline" onClick={handleCancel}>
                        取消
                    </Button>
                    <ConfirmDialog
                        title="确认关闭"
                        description="关闭将停止所有正在进行的下载。确认要关闭吗？"
                        onConfirm={async () => handleCloseDialog()}
                    >
                        <Button variant="destructive">关闭</Button>
                    </ConfirmDialog>
                </div>
            );
        }

        if (stage === 'paused') {
            return (
                <div className="flex gap-2 w-full justify-end">
                    <Button onClick={handleResume}>
                        <Play className="h-4 w-4 mr-1" />
                        继续
                    </Button>
                    <Button variant="outline" onClick={handleCancel}>
                        取消
                    </Button>
                    <ConfirmDialog
                        title="确认关闭"
                        description="关闭将停止所有正在进行的下载。确认要关闭吗？"
                        onConfirm={async () => handleCloseDialog()}
                    >
                        <Button variant="destructive">关闭</Button>
                    </ConfirmDialog>
                </div>
            );
        }

        // done stage
        return (
            <div className="flex gap-2 w-full justify-end">
                <Button variant="outline" onClick={handleCloseDialog}>关闭</Button>
            </div>
        );
    };

    return (
        <ContentDialog
            title="URL 下载"
            content={content}
            footer={footer}
            open={open}
            onOpenChange={handleOpenChange}
            className="sm:max-w-xl"
        >
            {children}
        </ContentDialog>
    );
}
