import { type UserFileItemType } from '@shared/apis/user/file';
import { Badge } from '@shared/components/ui/badge';
import { Drawer, DrawerContent, DrawerHeader, DrawerTitle } from '@apps/main/components/local/drawer';
import { useToast } from '@shared/contexts/toast-context';
import { type TypedDictData } from '@apps/main/hooks/use-dict-data';
import { cn, formatTime, TIME_STYLE } from '@shared/lib/utils';
import { createCopyWithToast } from '@shared/lib/utils/copy-utils';
import { createStatusMapper } from '@apps/main/lib/status-utils';

interface FileDetailDrawerProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    file: UserFileItemType;
    dictData: TypedDictData<['user_file']>;
}

export function FileDetailDrawer({
    open,
    onOpenChange,
    file,
    dictData,
}: FileDetailDrawerProps) {
    const { success: showSuccess, error: showError } = useToast();
    const copyText = createCopyWithToast(showSuccess, showError);

    // 状态映射 — 基于 FileStatus 枚举
    // 1=Normal, 2=Deleted, 3=Unfinished, 4=Failed
    const fileStatus = createStatusMapper(
        {
            1: 'success',   // Normal   - 正常
            2: 'danger',    // Deleted  - 已删除
            3: 'info',      // Unfinished - 未完成（上传中）
            4: 'danger',    // Failed   - 失败
        },
        (status) => dictData.file_status?.getLabel(String(status)) || String(status),
    );

    const formatFileSize = (size: number): string => {
        if (size === 0) return '0 B';
        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(size) / Math.log(1024));
        return parseFloat((size / Math.pow(1024, i)).toFixed(2)) + ' ' + units[i];
    };

    // 获取来源类型标签
    const getSourceTypeLabel = (sourceType?: string | null): string => {
        if (!sourceType) return '-';
        return dictData.file_source_type?.getLabel(sourceType) || sourceType;
    };

    // 获取存储类型标签
    const getStorageTypeLabel = (storageType?: string | null): string => {
        return storageType || '-';
    };

    return (
        <Drawer open={open} onOpenChange={onOpenChange}>
            <DrawerContent>
                <DrawerHeader className={cn("pb-6")}>
                    <DrawerTitle className={cn("text-xl")}>文件详细信息</DrawerTitle>
                </DrawerHeader>

                <div className="space-y-6">
                    {/* 基本信息 */}
                    <div className="space-y-4">
                        <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">基本信息</h3>
                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">文件ID</span>
                                <span className="text-sm font-medium">{file.id}</span>
                            </div>
                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">用户文件ID</span>
                                <span className="text-sm font-medium">{file.file_user_id}</span>
                            </div>
                            <div className="flex flex-col space-y-1 sm:col-span-2">
                                <span className="text-xs text-muted-foreground">文件名</span>
                                <span className="text-sm font-medium break-all">{file.file_name || '-'}</span>
                            </div>
                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">状态</span>
                                <div>
                                    <Badge className={cn(fileStatus.getClass(file.status))}>
                                        {fileStatus.getText(file.status)}
                                    </Badge>
                                </div>
                            </div>
                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">存储类型</span>
                                <span className="text-sm font-medium">{getStorageTypeLabel(file.storage_type)}</span>
                            </div>
                        </div>
                    </div>

                    <div className="border-t" />

                    {/* 文件属性 */}
                    <div className="space-y-4">
                        <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">文件属性</h3>
                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">文件大小</span>
                                <span className="text-sm font-medium">{formatFileSize(file.file_size)}</span>
                            </div>
                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">内容类型</span>
                                <span className="text-sm font-medium">{file.content_type || '-'}</span>
                            </div>
                            <div className="flex flex-col space-y-1 sm:col-span-2">
                                <span className="text-xs text-muted-foreground">文件 MD5</span>
                                <button
                                    className="text-sm font-mono text-left break-all hover:text-primary transition-colors"
                                    onClick={() => file.file_md5 && copyText(file.file_md5, 'MD5 已复制')}
                                    title="点击复制"
                                >
                                    {file.file_md5 || '-'}
                                </button>
                            </div>
                        </div>
                    </div>

                    <div className="border-t" />

                    {/* URL 信息 */}
                    <div className="space-y-4">
                        <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">URL 信息</h3>
                        <div className="grid grid-cols-1 gap-4">
                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">访问 URL</span>
                                {file.url ? (
                                    <a
                                        href={file.url}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        className="text-sm font-medium text-primary break-all hover:underline"
                                    >
                                        {file.url}
                                    </a>
                                ) : (
                                    <span className="text-sm font-medium text-muted-foreground">-</span>
                                )}
                            </div>
                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">来源 URL</span>
                                {file.source_url ? (
                                    <a
                                        href={file.source_url}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        className="text-sm font-medium text-primary break-all hover:underline"
                                    >
                                        {file.source_url}
                                    </a>
                                ) : (
                                    <span className="text-sm font-medium text-muted-foreground">-</span>
                                )}
                            </div>
                        </div>
                    </div>

                    <div className="border-t" />

                    {/* 本地存储属性 */}
                    {(file.local_id || file.local_path) && (
                        <>
                            <div className="space-y-4">
                                <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">本地存储</h3>
                                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                                    {file.local_id != null && (
                                        <div className="flex flex-col space-y-1">
                                            <span className="text-xs text-muted-foreground">本地 ID</span>
                                            <span className="text-sm font-medium">{file.local_id}</span>
                                        </div>
                                    )}
                                    {file.source_type && (
                                        <div className="flex flex-col space-y-1">
                                            <span className="text-xs text-muted-foreground">来源类型</span>
                                            <span className="text-sm font-medium">{getSourceTypeLabel(file.source_type)}</span>
                                        </div>
                                    )}
                                    {file.local_path && (
                                        <div className="flex flex-col space-y-1 sm:col-span-2">
                                            <span className="text-xs text-muted-foreground">本地路径</span>
                                            <span className="text-sm font-mono break-all">{file.local_path}</span>
                                        </div>
                                    )}
                                    {file.file_chunk_total !== undefined && (
                                        <>
                                            <div className="flex flex-col space-y-1">
                                                <span className="text-xs text-muted-foreground">分片总数</span>
                                                <span className="text-sm font-medium">{file.file_chunk_total}</span>
                                            </div>
                                            <div className="flex flex-col space-y-1">
                                                <span className="text-xs text-muted-foreground">成功分片数</span>
                                                <span className="text-sm font-medium">{file.file_chunk_succ}</span>
                                            </div>
                                            <div className="flex flex-col space-y-1">
                                                <span className="text-xs text-muted-foreground">分片大小</span>
                                                <span className="text-sm font-medium">{formatFileSize(file.file_chunk_size || 0)}</span>
                                            </div>
                                        </>
                                    )}
                                </div>
                            </div>
                            <div className="border-t" />
                        </>
                    )}

                    {/* OSS 存储属性 */}
                    {(file.oss_id || file.object_key) && (
                        <>
                            <div className="space-y-4">
                                <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">OSS 存储</h3>
                                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                                    {file.oss_id != null && (
                                        <div className="flex flex-col space-y-1">
                                            <span className="text-xs text-muted-foreground">OSS ID</span>
                                            <span className="text-sm font-medium">{file.oss_id}</span>
                                        </div>
                                    )}
                                    {file.bucket && (
                                        <div className="flex flex-col space-y-1">
                                            <span className="text-xs text-muted-foreground">存储桶</span>
                                            <span className="text-sm font-medium">{file.bucket}</span>
                                        </div>
                                    )}
                                    {file.region && (
                                        <div className="flex flex-col space-y-1">
                                            <span className="text-xs text-muted-foreground">地域</span>
                                            <span className="text-sm font-medium">{file.region}</span>
                                        </div>
                                    )}
                                    {file.object_key && (
                                        <div className="flex flex-col space-y-1 sm:col-span-2">
                                            <span className="text-xs text-muted-foreground">对象 Key</span>
                                            <span className="text-sm font-mono break-all">{file.object_key}</span>
                                        </div>
                                    )}
                                    {file.object_url && (
                                        <div className="flex flex-col space-y-1 sm:col-span-2">
                                            <span className="text-xs text-muted-foreground">对象 URL</span>
                                            <a
                                                href={file.object_url}
                                                target="_blank"
                                                rel="noopener noreferrer"
                                                className="text-sm font-medium text-primary break-all hover:underline"
                                            >
                                                {file.object_url}
                                            </a>
                                        </div>
                                    )}
                                    {file.oss_size !== undefined && (
                                        <div className="flex flex-col space-y-1">
                                            <span className="text-xs text-muted-foreground">OSS 大小</span>
                                            <span className="text-sm font-medium">{formatFileSize(file.oss_size || 0)}</span>
                                        </div>
                                    )}
                                </div>
                            </div>
                            <div className="border-t" />
                        </>
                    )}

                    {/* 其他信息 */}
                    <div className="space-y-4">
                        <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">其他信息</h3>
                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">用户 ID</span>
                                <span className="text-sm font-medium">{file.user_id}</span>
                            </div>
                            <div className="flex flex-col space-y-1">
                                <span className="text-xs text-muted-foreground">添加时间</span>
                                <span className="text-sm font-medium">
                                    {file.add_time ? formatTime(file.add_time, TIME_STYLE.ABSOLUTE_ELEMENT) : '-'}
                                </span>
                            </div>
                        </div>
                    </div>
                </div>
            </DrawerContent>
        </Drawer>
    );
}
