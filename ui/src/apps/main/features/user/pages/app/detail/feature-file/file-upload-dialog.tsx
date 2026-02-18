import { userFileDelete, userFileUploadCreate, userFileUploadData, userFileUploadByMd5, type UserFileMappingResType } from '@shared/apis/user/file';
import { ContentDialog } from '@shared/components/custom/dialog/content-dialog';
import { ConfirmDialog } from '@shared/components/custom/dialog/confirm-dialog';
import { Button } from '@shared/components/ui/button';
import { Progress } from '@shared/components/ui/progress';
import { useToast } from '@shared/contexts/toast-context';
import { cn, formatFileSize, calculateFileMd5 } from '@shared/lib/utils';
import { calculateFileMd5WithWorker, terminateWorker, isWorkerSupported } from '@shared/lib/utils/md5-worker-utils';
import { Loader2, Upload, CheckCircle2, XCircle, AlertCircle, Hash } from 'lucide-react';
import { useMutation } from '@tanstack/react-query';
import type { AxiosProgressEvent } from 'axios';
import React, { useCallback, useEffect, useRef, useState } from 'react';

type UploadStage = 'idle' | 'hashing' | 'uploading' | 'paused' | 'success' | 'error' | 'closing';

interface FileUploadDialogProps {
    children: React.ReactNode;
    appId: number;
    uploadConfig: UserFileMappingResType;
    onSuccess?: () => void;
    maxConcurrentChunks?: number;
}

interface ChunkTask {
    index: number;
    offset: number;
    len: number;
    status: 'pending' | 'uploading' | 'done' | 'error';
}

export function FileUploadDialog({
    children,
    appId,
    uploadConfig,
    onSuccess,
    maxConcurrentChunks = 3,
}: FileUploadDialogProps) {
    const { success: showSuccess, error: showError } = useToast();
    const [open, setOpen] = useState(false);
    const [stage, setStage] = useState<UploadStage>('idle');
    const [selectedFile, setSelectedFile] = useState<File | null>(null);
    const [fileId, setFileId] = useState<number | null>(null);
    const [progress, setProgress] = useState(0);
    const [errorMsg, setErrorMsg] = useState('');
    const [chunkTasks, setChunkTasks] = useState<ChunkTask[]>([]);
    const abortRef = useRef<AbortController | null>(null);
    const isPausedRef = useRef(false);
    const fileInputRef = useRef<HTMLInputElement>(null);
    const [hashProgress, setHashProgress] = useState(0);
    const dropZoneRef = useRef<HTMLDivElement>(null);
    const [isDragOver, setIsDragOver] = useState(false);
    const uploadedSizeRef = useRef(0);
    const workerSupportedRef = useRef<boolean | null>(null);
    const uploadControllerRef = useRef<AbortController | null>(null);
    const isUploadingRef = useRef(false); // 标记是否正在处理上传，避免重复触发

    // React Query Mutations
    const uploadByMd5Mutation = useMutation({
        mutationFn: (params: { app_id: number; file_md5: string }) =>
            userFileUploadByMd5(params),
    });

    const uploadCreateMutation = useMutation({
        mutationFn: (params: Parameters<typeof userFileUploadCreate>[0]) =>
            userFileUploadCreate(params),
    });

    const deleteMutation = useMutation({
        mutationFn: (params: { app_id: number; file_id: number }) =>
            userFileDelete(params),
    });

    // 组件卸载时清理 Worker 资源
    useEffect(() => {
        // 检测 Worker 支持情况
        if (workerSupportedRef.current === null) {
            workerSupportedRef.current = isWorkerSupported();
        }

        return () => {
            terminateWorker();
        };
    }, []);

    const resetState = useCallback(() => {
        setStage('idle');
        setSelectedFile(null);
        setFileId(null);
        setProgress(0);
        setHashProgress(0);
        setErrorMsg('');
        setChunkTasks([]);
        isPausedRef.current = false;
        uploadedSizeRef.current = 0;
        // 中止所有进行中的上传请求
        abortRef.current?.abort();
        abortRef.current = null;
        uploadControllerRef.current?.abort();
        uploadControllerRef.current = null;
        // 组件卸载时清理 Worker
        terminateWorker();
    }, []);

    // 并发上传分片（带并发控制和完整的取消支持）
    const uploadChunksWithConcurrency = useCallback(async (
        file: File,
        fileId: number,
        tasks: ChunkTask[],
        totalSize: number,
        concurrency: number
    ) => {
        // 初始化已上传大小（恢复上传场景下）
        const completedSize = tasks.filter(t => t.status === 'done').reduce((acc, t) => acc + t.len, 0);
        uploadedSizeRef.current = completedSize;

        const pendingTasks = tasks.filter(t => t.status !== 'done');

        // 创建一个上传控制器用于全局取消
        const uploadController = new AbortController();
        uploadControllerRef.current = uploadController;

        // 创建一个队列来管理并发
        let currentIndex = 0;
        const uploadPromises: Promise<void>[] = [];

        const uploadTask = async () => {
            while (currentIndex < pendingTasks.length) {
                // 检查暂停或全局取消信号
                if (isPausedRef.current) {
                    throw new Error('PAUSED');
                }
                if (uploadController.signal.aborted) {
                    throw new Error('CANCELED');
                }

                const index = currentIndex++;
                const task = pendingTasks[index];

                task.status = 'uploading';
                setChunkTasks([...tasks]);

                try {
                    const blob = file.slice(task.offset, task.offset + task.len);
                    const controller = new AbortController();
                    abortRef.current = controller;

                    // 使用分片上传 API（支持进度回调）
                    await userFileUploadData(fileId, task.index, blob, {
                        signal: controller.signal,
                        onUploadProgress: (event: AxiosProgressEvent) => {
                            if (event.total) {
                                const chunkProgress = event.loaded / event.total;
                                // 基于已上传大小和当前分片进度计算总进度
                                const currentProgress = (uploadedSizeRef.current + task.len * chunkProgress) / totalSize * 100;
                                setProgress(prev => Math.min(Math.max(currentProgress, prev), 99));
                            }
                        },
                    });

                    task.status = 'done';
                    // 更新已上传大小
                    uploadedSizeRef.current += task.len;
                    setChunkTasks([...tasks]);
                } catch (err: any) {
                    if (err?.message === 'PAUSED') {
                        task.status = 'pending';
                        setChunkTasks([...tasks]);
                        throw err;
                    }
                    if (err?.message === 'CANCELED') {
                        task.status = 'pending';
                        setChunkTasks([...tasks]);
                        throw err;
                    }
                    if (err?.name === 'CanceledError' || err?.name === 'AbortError') {
                        task.status = 'pending';
                        setChunkTasks([...tasks]);
                        throw new Error('PAUSED');
                    }
                    task.status = 'error';
                    setChunkTasks([...tasks]);
                    throw err;
                }
            }
        };

        // 启动指定数量的并发任务
        for (let i = 0; i < Math.min(concurrency, pendingTasks.length); i++) {
            uploadPromises.push(uploadTask());
        }

        // 等待所有上传任务完成
        try {
            await Promise.all(uploadPromises);
        } finally {
            // 清理上传控制器
            uploadControllerRef.current = null;
        }
    }, []);


    const handleOpenChange = useCallback((newOpen: boolean) => {
        if (!newOpen && (stage === 'uploading' || stage === 'paused' || stage === 'hashing')) {
            // 不允许直接关闭，由关闭按钮触发
            return;
        }
        if (!newOpen) {
            resetState();
        }
        setOpen(newOpen);
    }, [stage, resetState]);

    // 构建分片信息
    const buildChunks = useCallback((fileSize: number) => {
        const { chunk_threshold, default_chunk_size, min_chunk_size } = uploadConfig;
        if (fileSize <= chunk_threshold) {
            // 小文件直接单分片
            return [{ offset: 0, len: fileSize }];
        }
        // 大文件分片
        const chunkSize = Math.max(default_chunk_size, min_chunk_size);
        const chunks: { offset: number; len: number }[] = [];
        let offset = 0;
        while (offset < fileSize) {
            const len = Math.min(chunkSize, fileSize - offset);
            chunks.push({ offset, len });
            offset += len;
        }
        return chunks;
    }, [uploadConfig]);

    // 恢复上传（重新上传剩余分片）
    const resumeUpload = useCallback(async () => {
        if (!selectedFile || !fileId) return;

        setStage('uploading');
        isPausedRef.current = false;
        uploadedSizeRef.current = 0;

        try {
            const tasks = [...chunkTasks];
            const totalSize = selectedFile.size;

            try {
                await uploadChunksWithConcurrency(selectedFile, fileId, tasks, totalSize, maxConcurrentChunks);
            } catch (err: any) {
                if (err?.message === 'PAUSED') {
                    setStage('paused');
                    return;
                }
                throw err;
            }

            setProgress(100);
            setStage('success');
            showSuccess('文件上传成功');
            onSuccess?.();
        } catch (err: any) {
            const msg = err?.data?.message || err?.message || '上传失败';
            setErrorMsg(msg);
            setStage('error');
        }
    }, [selectedFile, fileId, chunkTasks, showSuccess, onSuccess, uploadChunksWithConcurrency, maxConcurrentChunks]);

    // 暂停上传
    const handlePause = useCallback(() => {
        isPausedRef.current = true;
        abortRef.current?.abort();
        // 中止所有待处理的上传请求
        uploadControllerRef.current?.abort();
    }, []);

    // 关闭上传（删除文件）
    const handleClose = useCallback(async () => {
        setStage('closing');
        // 立即中止所有进行中的请求
        abortRef.current?.abort();
        uploadControllerRef.current?.abort();
        
        if (fileId) {
            try {
                await deleteMutation.mutateAsync({ app_id: appId, file_id: fileId });
                resetState();
                setOpen(false);
            } catch (err: any) {
                const msg = err?.data?.message || err?.message || '删除文件失败';
                showError(msg);
                resetState();
                setOpen(false);
            }
        } else {
            resetState();
            setOpen(false);
        }
    }, [fileId, appId, resetState, showError, deleteMutation]);

    // 文件选择处理 - 只做状态更新，让 UI 立即响应
    const handleFileSelect = useCallback((file: File) => {
        setSelectedFile(file);
        setOpen(true);
        setStage('hashing');
        setProgress(0);
        setHashProgress(0);
        setErrorMsg('');
        setChunkTasks([]);
        isPausedRef.current = false;
        uploadedSizeRef.current = 0;
        isUploadingRef.current = false; // 标记为未开始处理
    }, []);

    // 监听文件选择状态变化，执行上传处理
    useEffect(() => {
        if (!selectedFile || stage !== 'hashing' || isUploadingRef.current) {
            return;
        }

        isUploadingRef.current = true;

        const performUpload = async () => {
            try {
                // 校验文件大小
                if (selectedFile.size > uploadConfig.max_upload_size) {
                    throw new Error(`文件大小 ${formatFileSize(selectedFile.size)} 超过最大限制 ${formatFileSize(uploadConfig.max_upload_size)}`);
                }

                // 计算文件 MD5
                const fileMd5 = workerSupportedRef.current
                    ? await calculateFileMd5WithWorker(selectedFile, {
                        onProgress: setHashProgress,
                    })
                    : await calculateFileMd5(selectedFile, setHashProgress);

                // 检查 MD5 是否已存在（秒传）
                const md5Res = await uploadByMd5Mutation.mutateAsync({ app_id: appId, file_md5: fileMd5 });
                if (md5Res.status === true && md5Res.response?.matched === true) {
                    setProgress(100);
                    setStage('success');
                    showSuccess('文件秒传成功（已存在相同文件）');
                    onSuccess?.();
                    return;
                }

                // 未匹配，继续正常上传流程
                setProgress(0);
                setStage('uploading');
                const chunks = buildChunks(selectedFile.size);

                // 创建上传任务
                const createRes = await uploadCreateMutation.mutateAsync({
                    app_id: appId,
                    file_name: selectedFile.name,
                    chunks: chunks.map(c => ({ offset: c.offset, len: c.len })),
                });

                if (!createRes.status || !createRes.response) {
                    throw new Error(createRes.message || '创建上传任务失败');
                }

                const fId = createRes.response.file_id;
                setFileId(fId);

                // 初始化分片任务
                const tasks: ChunkTask[] = chunks.map((c, i) => ({
                    index: i,
                    offset: c.offset,
                    len: c.len,
                    status: 'pending' as const,
                }));
                setChunkTasks(tasks);

                // 并发上传分片
                const totalSize = selectedFile.size;
                try {
                    await uploadChunksWithConcurrency(selectedFile, fId, tasks, totalSize, maxConcurrentChunks);
                } catch (err: any) {
                    if (err?.message === 'PAUSED') {
                        setStage('paused');
                        return;
                    }
                    throw err;
                }

                // 全部上传完成
                setProgress(100);
                setStage('success');
                showSuccess('文件上传成功');
                onSuccess?.();
            } catch (err: any) {
                const msg = err?.data?.message || err?.message || '上传失败';
                setErrorMsg(msg);
                setStage('error');
            } finally {
                isUploadingRef.current = false;
            }
        };

        performUpload();
    }, [selectedFile, stage, appId, uploadConfig.max_upload_size, uploadByMd5Mutation, showSuccess, onSuccess, buildChunks, uploadCreateMutation, uploadChunksWithConcurrency, maxConcurrentChunks]);

    const handleDrop = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        setIsDragOver(false);
        const files = e.dataTransfer.files;
        if (files.length > 0) {
            handleFileSelect(files[0]);
        }
    }, [handleFileSelect]);

    const handleDragOver = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        setIsDragOver(true);
    }, []);

    const handleDragLeave = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        setIsDragOver(false);
    }, []);

    const stageIcon = {
        idle: <Upload className="h-10 w-10 text-muted-foreground" />,
        hashing: <Hash className="h-10 w-10 text-primary animate-pulse" />,
        uploading: <Loader2 className="h-10 w-10 text-primary animate-spin" />,
        paused: <AlertCircle className="h-10 w-10 text-yellow-500" />,
        success: <CheckCircle2 className="h-10 w-10 text-green-500" />,
        error: <XCircle className="h-10 w-10 text-red-500" />,
        closing: <Loader2 className="h-10 w-10 text-muted-foreground animate-spin" />,
    };

    const content = (
        <div className="space-y-4">
            {stage === 'idle' ? (
                <>
                    {/* 拖拽上传区域 */}
                    <div
                        ref={dropZoneRef}
                        onDrop={handleDrop}
                        onDragOver={handleDragOver}
                        onDragLeave={handleDragLeave}
                        onClick={() => fileInputRef.current?.click()}
                        className={cn(
                            "border-2 border-dashed rounded-lg p-6 cursor-pointer transition-colors flex items-center gap-4",
                            isDragOver
                                ? "border-primary bg-primary/5"
                                : "border-muted-foreground/25 hover:border-primary/50 hover:bg-muted/50"
                        )}
                    >
                        <div className="flex-shrink-0">
                            {stageIcon.idle}
                        </div>
                        <div className="min-w-0">
                            <p className="text-sm text-muted-foreground">
                                拖拽文件到此处，或点击选择文件
                            </p>
                            <p className="mt-1 text-xs text-muted-foreground">
                                最大 {formatFileSize(uploadConfig.max_upload_size)}，超过 {formatFileSize(uploadConfig.chunk_threshold)} 将自动分片上传
                            </p>
                        </div>
                    </div>
                    <input
                        ref={fileInputRef}
                        type="file"
                        className="hidden"
                        onChange={(e) => {
                            const file = e.target.files?.[0];
                            if (file) handleFileSelect(file);
                            e.target.value = '';
                        }}
                    />
                </>
            ) : (
                <div className="space-y-4">
                    {/* 状态图标和文件信息 */}
                    <div className="flex items-center gap-3">
                        {stageIcon[stage]}
                        <div className="flex-1 min-w-0">
                            <p className="text-sm font-medium truncate">{selectedFile?.name}</p>
                            <p className="text-xs text-muted-foreground">
                                {selectedFile && formatFileSize(selectedFile.size)}
                                {chunkTasks.length > 1 && ` · ${chunkTasks.length} 个分片`}
                            </p>
                        </div>
                    </div>

                    {/* MD5 计算进度 */}
                    {stage === 'hashing' && (
                        <div className="space-y-3">
                            <div className="border border-border rounded p-3 bg-muted/50 text-xs text-foreground">
                                <p className="font-medium">
                                    {workerSupportedRef.current ? '后台计算中' : '文件指纹计算中'}
                                </p>
                                <p className="mt-2 text-muted-foreground">
                                    {workerSupportedRef.current
                                        ? "系统正在后台扫描文件并计算指纹（用于秒传验证），UI 保持响应。支持大文件，请耐心等待。"
                                        : "系统正在计算文件指纹（用于秒传验证）。请耐心等待，期间 UI 可能会有短暂卡顿。"}
                                </p>
                            </div>
                            <div className="space-y-2">
                                <Progress value={hashProgress} className="h-2" />
                                <div className="flex justify-between text-xs text-muted-foreground">
                                    <span className="flex items-center gap-1">
                                        <span className="inline-block w-2 h-2 bg-primary rounded-full animate-pulse"></span>
                                        {Math.round(hashProgress)}%
                                    </span>
                                </div>
                            </div>
                        </div>
                    )}

                    {/* 进度条 */}
                    {(stage === 'uploading' || stage === 'paused') && (
                        <div className="space-y-3">
                            {stage === 'uploading' && (
                                <div className="border border-border rounded p-3 bg-muted/50 text-xs text-foreground">
                                    <p className="font-medium">上传进行中</p>
                                    <p className="mt-2 text-muted-foreground">文件分片已上传至服务器进行处理。</p>
                                </div>
                            )}
                            {stage === 'paused' && (
                                <div className="border border-border rounded p-3 bg-muted/50 text-xs text-foreground">
                                    <p className="font-medium flex items-center gap-2">
                                        <AlertCircle className="w-4 h-4" />
                                        已暂停
                                    </p>
                                    <p className="mt-2 text-muted-foreground">您可以继续上传或关闭此任务。</p>
                                </div>
                            )}
                            <div className="space-y-2">
                                <Progress value={progress} className="h-2" />
                                <div className="flex justify-between text-xs text-muted-foreground">
                                    <span className="flex items-center gap-1">
                                        {stage === 'paused' ? (
                                            '已暂停'
                                        ) : (
                                            <>
                                                <span className="inline-block w-2 h-2 bg-primary rounded-full animate-pulse"></span>
                                                上传中
                                            </>
                                        )}
                                        {chunkTasks.length > 1 && ` (${chunkTasks.filter(t => t.status === 'done').length}/${chunkTasks.length})`}
                                    </span>
                                    <span>{Math.round(progress)}%</span>
                                </div>
                            </div>
                        </div>
                    )}

                    {/* 成功提示 */}
                    {stage === 'success' && (
                        <p className="text-sm text-green-600">文件上传成功！</p>
                    )}

                    {/* 错误提示 */}
                    {stage === 'error' && (
                        <p className="text-sm text-red-600">{errorMsg}</p>
                    )}

                    {/* 关闭中 */}
                    {stage === 'closing' && (
                        <p className="text-sm text-muted-foreground">正在关闭并清理文件...</p>
                    )}
                </div>
            )}
        </div>
    );

    const footer = (closeDialog: () => void) => {
        if (stage === 'idle') {
            return (
                <Button variant="outline" onClick={closeDialog}>
                    取消
                </Button>
            );
        }

        if (stage === 'hashing') {
            return (
                <Button variant="outline" disabled>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    {workerSupportedRef.current
                        ? '准备中：后台计算文件指纹...'
                        : '准备中：计算文件指纹...'}
                </Button>
            );
        }

        if (stage === 'uploading') {
            return (
                <div className="flex gap-2 w-full justify-end">
                    <ConfirmDialog
                        title="确认暂停上传"
                        description="暂停后可以继续上传剩余分片。确认要暂停吗？"
                        onConfirm={async () => handlePause()}
                    >
                        <Button variant="outline">暂停上传</Button>
                    </ConfirmDialog>
                    <ConfirmDialog
                        title="确认关闭上传"
                        description="关闭将取消当前上传并删除已上传的文件数据。确认要关闭吗？"
                        onConfirm={handleClose}
                    >
                        <Button variant="destructive" className="text-destructive-foreground">关闭上传</Button>
                    </ConfirmDialog>
                </div>
            );
        }

        if (stage === 'paused') {
            return (
                <div className="flex gap-2 w-full justify-end">
                    <Button onClick={resumeUpload}>重新上传</Button>
                    <ConfirmDialog
                        title="确认关闭上传"
                        description="关闭将取消当前上传并删除已上传的文件数据。确认要关闭吗？"
                        onConfirm={handleClose}
                    >
                        <Button variant="destructive" className="text-destructive-foreground">关闭上传</Button>
                    </ConfirmDialog>
                </div>
            );
        }

        if (stage === 'error') {
            return (
                <div className="flex gap-2 w-full justify-end">
                    <Button onClick={resumeUpload}>重新上传</Button>
                    <ConfirmDialog
                        title="确认关闭上传"
                        description="关闭将取消当前上传并删除已上传的文件数据。确认要关闭吗？"
                        onConfirm={handleClose}
                    >
                        <Button variant="destructive" className="text-destructive-foreground">关闭上传</Button>
                    </ConfirmDialog>
                </div>
            );
        }

        if (stage === 'success') {
            return (
                <Button onClick={() => {
                    resetState();
                    setOpen(false);
                }}>
                    完成
                </Button>
            );
        }

        if (stage === 'closing') {
            return (
                <Button disabled>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    关闭中...
                </Button>
            );
        }

        return null;
    };

    return (
        <ContentDialog
            title="上传文件"
            content={content}
            footer={footer}
            open={open}
            onOpenChange={handleOpenChange}
        >
            {children}
        </ContentDialog>
    );
}
