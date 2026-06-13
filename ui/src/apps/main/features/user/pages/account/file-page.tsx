import { FilterBar } from '@apps/main/components/filter-bar/container';
import { FilterActions } from '@apps/main/components/filter-bar/filter-actions/filter-actions';
import { FilterSearchButton } from '@apps/main/components/filter-bar/filter-actions/filter-search-button';
import { FilterResetButton } from '@apps/main/components/filter-bar/filter-actions/filter-reset-button';
import { FilterContentSearch, FilterTotalCount } from '@apps/main/components/filter-bar/filter-fields';
import { useFilterBarForm } from '@apps/main/hooks/use-filter-bar-form';
import { ExportButton, ExportMobileButton, ExportSplitButton } from '@apps/main/components/export-manager/export-buttons';
import { ExportDrawer } from '@apps/main/components/export-manager/export-drawer';
import { useUserExportAction } from '@apps/main/hooks/use-user-export-action';
import {
    useDictData,
    type TypedDictData,
} from '@apps/main/hooks/use-dict-data';
import {
    CursorPagination,
    DEFAULT_PAGE_SIZE,
    PAGE_SIZE_OPTIONS,
    useLimitCountNum,
    useSearchNavigate,
} from '@apps/main/lib/pagination-utils';
import { createStatusMapper } from '@apps/main/lib/status-utils';
import { PostDownload } from '@apps/main/components/local/post-download';
import { Route } from '@apps/main/routes/_main/user/account/file';
import { zodResolver } from '@hookform/resolvers/zod';
import {
    userSelfFileDelete,
    userSelfFileList,
    EXPORT_TYPE_USER_FILE_LIST,
    type UserFileItemType,
} from '@shared/apis/user/file';
import { DataTable } from '@shared/components/custom//table';
import { ConfirmDialog } from '@shared/components/custom/dialog/confirm-dialog';
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error';
import { PageSkeletonTable } from '@shared/components/custom/page-placeholder/skeleton-table';
import {
    DataTableAction,
    DataTableActionItem,
} from '@shared/components/custom/table';
import { Badge } from '@shared/components/ui/badge';
import { Button } from '@shared/components/ui/button';
import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
} from '@shared/components/ui/tooltip';
import { useToast } from '@shared/contexts/toast-context';
import { getUserFileShareUrl } from '@shared/lib/apis/api_read';
import {
    cn,
    formatFileSize,
    formatTime,
    getQueryResponseCursor,
    getQueryResponseData,
    TIME_STYLE,
} from '@shared/lib/utils';
import { formatTotalCount } from '@shared/lib/utils/format-utils';
import { type LimitType } from '@shared/types/base-schema';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { type ColumnDef } from '@tanstack/react-table';
import { Cloud, Download, Eye, HardDrive, Lock, Share2, ShieldCheck, Trash2 } from 'lucide-react';
import React, { useState } from 'react';
import { FileDetailDrawer } from '../app/detail/feature-file/file-detail-drawer';
import { FilePublicUrlDialog } from '../../components/ui/file-public-url-dialog';
import { CONTENT_SEARCH_TYPES, FileFilterFormSchema, type FileFilterParamType } from './file-schema';
import * as z from 'zod';

export function AccountFilePage() {
    const {
        dictData,
        isLoading: dictIsLoading,
        isError: dictError,
        errors: dictErrors,
        refetch: refetchDict,
    } = useDictData(['user_file', 'user_export'] as const);

    if (dictError && dictErrors.length > 0) {
        return (
            <CenteredError
                variant="page"
                error={dictErrors}
                onReset={refetchDict}
                className={cn('m-4 md:m-6')}
            />
        );
    }

    if (dictIsLoading) {
        return (
            <PageSkeletonTable
                variant="page"
                columns={6}
                rows={8}
                className={cn('m-4 md:m-6')}
            />
        );
    }

    return <AccountFileContent dictData={dictData} />;
}

interface AccountFileContentProps {
    dictData: TypedDictData<['user_file', 'user_export']>;
}

function AccountFileContent({ dictData }: AccountFileContentProps) {
    const queryClient = useQueryClient();
    const { success: showSuccess, error: showError } = useToast();
    const navigate = useNavigate();

    const filterParam = Route.useSearch() as FileFilterParamType;
    const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;

    const [detailDrawerOpen, setDetailDrawerOpen] = useState(false);
    const [detailFile, setDetailFile] = useState<UserFileItemType | null>(null);

    // 从字典数据中获取存储类型
    const storageTypes = React.useMemo(() => dictData?.storage_type || [], [dictData?.storage_type])
    
    // 创建 storage_type 到 is_private 的映射
    const storageTypePrivateMap = React.useMemo(() => {
        const map = new Map<string, boolean>();
        storageTypes.forEach(st => {
            // 本地存储类型：local_public 是公开的，其他是私有的
            if (st.key.startsWith('local_')) {
                map.set(st.key, st.key !== 'local_public');
            } else {
                // OSS 存储类型：根据 is_private 字段判断
                map.set(st.key, st.is_private ?? false);
            }
        });
        return map;
    }, [storageTypes]);
    
    // 创建 storage_type key 到名称的映射
    const storageTypeNameMap = React.useMemo(() => {
        const map = new Map<string, string>();
        storageTypes.forEach(st => {
            map.set(st.key, st.val);
        });
        return map;
    }, [storageTypes]);
    
    // 判断文件是否为公开存储
    const isPublicStorage = (storageType: string): boolean => {
        const isPrivate = storageTypePrivateMap.get(storageType);
        // 如果找不到配置，默认为私有（安全起见）
        return isPrivate === false;
    };
    
    // 获取存储类型的显示名称
    const getStorageTypeName = (storageType: string): string => {
        return storageTypeNameMap.get(storageType) || storageType;
    };

    const filters = {
        status: filterParam.status || null,
        tag_name: filterParam.tag_name || null,
        content_type: filterParam.content_type || null,
        content_value: filterParam.content_value || null,
    };

    const pagination: LimitType = {
        pos: filterParam.pos || null,
        limit: filterParam.limit || DEFAULT_PAGE_SIZE,
        forward: filterParam.forward ?? true,
        more: true,
    };

    const searchGo = useSearchNavigate(navigate, filterParam);
    const countNumManager = useLimitCountNum(filters);

    const contentFilterParams = React.useMemo(() => {
        if (!filters.content_type || !filters.content_value) return {};
        const map: Record<string, string> = {
            file_md5: 'file_md5',
            source_url: 'source_url',
            url: 'url',
        };
        const key = map[filters.content_type];
        return key ? { [key]: filters.content_value } : {};
    }, [filters.content_type, filters.content_value]);

    const exportAction = useUserExportAction({
        exportType: EXPORT_TYPE_USER_FILE_LIST,
        params: {
            status: filters.status ?? undefined,
            tag_names: filters.tag_name ? [filters.tag_name] : undefined,
            ...contentFilterParams,
        },
    });

    const { data: fileData, isSuccess, isLoading, isError, error } = useQuery({
        queryKey: [
            'userSelfFileList',
            pagination.pos,
            currentLimit,
            pagination.forward,
            pagination.more,
            filters.status,
            filters.tag_name,
            filters.content_type,
            filters.content_value,
        ],
        queryFn: ({ signal }) =>
            userSelfFileList(
                {
                    limit: pagination,
                    count_num: countNumManager.getCountNum(),
                    status: filters.status ?? undefined,
                    tag_names: filters.tag_name ? [filters.tag_name] : undefined,
                    ...contentFilterParams,
                },
                { signal },
            ),
        placeholderData: (previousData) => previousData,
    });

    isSuccess && countNumManager.handleQueryResult(fileData);

    const files = getQueryResponseData<UserFileItemType[]>(fileData, []);
    const cursorData = getQueryResponseCursor(fileData);

    const deleteFileMutation = useMutation({
        mutationFn: (params: { file_ref_id: number }) =>
            userSelfFileDelete({ file_ref_id: params.file_ref_id }),
        onSuccess: () => {
            showSuccess('文件已删除');
            queryClient.invalidateQueries({ queryKey: ['userSelfFileList'] });
        },
        onError: (error: any) => {
            showError(error?.data?.message || error?.message || '删除文件失败');
        },
    });

    const handleDeleteFile = async (fileRefId: number) => {
        await deleteFileMutation.mutateAsync({ file_ref_id: fileRefId });
    };

    const refreshData = () => {
        queryClient.refetchQueries({ queryKey: ['userSelfFileList'] });
    };

    const clearCacheAndReload = () => {
        countNumManager.reset();
        queryClient.invalidateQueries({ queryKey: ['userSelfFileList'] });
    };

    const fileStatus = createStatusMapper(
        {
            1: 'success',
            2: 'danger',
            3: 'info',
            4: 'danger',
        },
        (status) => dictData.file_status?.getLabel(String(status), '') ?? '',
    );

    const columns: ColumnDef<UserFileItemType>[] = [
        {
            accessorKey: 'id',
            header: 'ID',
            size: 60,
            cell: ({ getValue }) => (
                <div className="py-1 text-xs text-muted-foreground">{getValue<number>()}</div>
            ),
        },
        {
            accessorKey: 'file_name',
            header: '文件名',
            cell: ({ row }) => {
                const fileName = row.original.file_name;
                const storageType = row.original.storage_type;
                const isLocal = storageType?.startsWith("local");
                const isCloud = !!storageType && !isLocal;
                const isPrivate = !isPublicStorage(storageType);
                const isCrypto = storageType === "local_crypto";
                const storageName = getStorageTypeName(storageType);
                
                return (
                    <div className="flex items-center gap-1 py-1 max-w-[220px]">
                        <span className="truncate text-sm" title={fileName}>
                            {fileName || '-'}
                        </span>
                        {isLocal && (
                            <TooltipProvider delayDuration={200}>
                                <Tooltip>
                                    <TooltipTrigger asChild>
                                        <HardDrive className="h-3 w-3 flex-shrink-0 text-emerald-500 cursor-pointer" />
                                    </TooltipTrigger>
                                    <TooltipContent side="top">
                                        <span>{storageName}</span>
                                    </TooltipContent>
                                </Tooltip>
                            </TooltipProvider>
                        )}
                        {isCloud && (
                            <TooltipProvider delayDuration={200}>
                                <Tooltip>
                                    <TooltipTrigger asChild>
                                        <Cloud className="h-3 w-3 flex-shrink-0 text-blue-400 cursor-pointer" />
                                    </TooltipTrigger>
                                    <TooltipContent side="top">
                                        <span>{storageName}</span>
                                    </TooltipContent>
                                </Tooltip>
                            </TooltipProvider>
                        )}
                        {isCrypto && (
                            <TooltipProvider delayDuration={200}>
                                <Tooltip>
                                    <TooltipTrigger asChild>
                                        <ShieldCheck className="h-3 w-3 flex-shrink-0 text-amber-500/70 cursor-pointer" />
                                    </TooltipTrigger>
                                    <TooltipContent side="top">
                                        <span>加密存储</span>
                                    </TooltipContent>
                                </Tooltip>
                            </TooltipProvider>
                        )}
                        {isPrivate && (
                            <TooltipProvider delayDuration={200}>
                                <Tooltip>
                                    <TooltipTrigger asChild>
                                        <Lock className="h-3 w-3 flex-shrink-0 text-muted-foreground/50 cursor-pointer" />
                                    </TooltipTrigger>
                                    <TooltipContent side="top">
                                        <span>私有文件</span>
                                    </TooltipContent>
                                </Tooltip>
                            </TooltipProvider>
                        )}
                    </div>
                );
            },
        },
        {
            accessorKey: 'file_size',
            header: '文件大小',
            size: 100,
            cell: ({ getValue }) => (
                <div className="py-1 text-sm">{formatFileSize(getValue<number>())}</div>
            ),
        },
        {
            accessorKey: 'content_type',
            header: '内容类型',
            cell: ({ getValue }) => (
                <div
                    className="max-w-[120px] truncate py-1 text-xs text-muted-foreground"
                    title={getValue<string>() || ''}
                >
                    {getValue<string>() || '-'}
                </div>
            ),
        },
        {
            accessorKey: 'status',
            header: '状态',
            size: 80,
            cell: ({ getValue }) => {
                const status = getValue<number>();
                return (
                    <div className="py-1">
                        <Badge className={cn(fileStatus.getClass(status))}>
                            {fileStatus.getText(status)}
                        </Badge>
                    </div>
                );
            },
        },
        {
            accessorKey: 'add_time',
            header: '添加时间',
            size: 120,
            cell: ({ getValue }) => {
                const addTime = getValue<Date | null>();
                return (
                    <div className="text-xs py-1">
                        {addTime ? formatTime(addTime, TIME_STYLE.RELATIVE_ELEMENT) : '-'}
                    </div>
                );
            },
        },
        {
            id: 'actions',
            header: () => <div className="text-center py-1">操作</div>,
            size: 120,
            cell: ({ row }) => {
                const file = row.original;
                return (
                    <DataTableAction className="justify-end sm:justify-center">
                        <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                            <Button
                                variant="ghost"
                                size="sm"
                                className={cn('h-auto px-2 py-1')}
                                title="详细信息"
                                onClick={() => { setDetailFile(file); setDetailDrawerOpen(true); }}
                            >
                                <Eye className="h-3 w-3" />
                                <span className="text-xs ml-1">详细</span>
                            </Button>
                        </DataTableActionItem>

                        {file.file_key && file.status === 1 ? (
                            <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                                <PostDownload
                                    url="/api/user/file/read"
                                    body={{ key: file.file_key }}
                                >
                                    {({ onClick, isLoading: dlLoading }) => (
                                        <Button
                                            variant="ghost"
                                            size="sm"
                                            className={cn('h-auto px-2 py-1')}
                                            title="下载文件"
                                            onClick={onClick}
                                            disabled={dlLoading}
                                        >
                                            <Download className="h-3 w-3" />
                                            <span className="text-xs ml-1">{dlLoading ? '下载中...' : '下载'}</span>
                                        </Button>
                                    )}
                                </PostDownload>
                            </DataTableActionItem>
                        ) : null}

                        {file.file_key && file.status === 1 && isPublicStorage(file.storage_type) ? (
                            <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                                <FilePublicUrlDialog url={getUserFileShareUrl(file.file_key!)}>
                                    <Button
                                        variant="ghost"
                                        size="sm"
                                        className={cn('h-auto px-2 py-1')}
                                        title="公开链接"
                                    >
                                        <Share2 className="h-3 w-3" />
                                        <span className="text-xs ml-1">公开链接</span>
                                    </Button>
                                </FilePublicUrlDialog>
                            </DataTableActionItem>
                        ) : null}

                        <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                            <ConfirmDialog
                                title="确认删除"
                                description={<>您确定要删除文件 <strong>{file.file_name}</strong> 吗？删除后将无法恢复。</>}
                                onConfirm={async () => await handleDeleteFile(file.id)}
                            >
                                <Button
                                    variant="ghost"
                                    size="sm"
                                    className={cn('h-auto px-2 py-1 text-destructive hover:text-destructive')}
                                    title="删除文件"
                                >
                                    <Trash2 className="h-3 w-3" />
                                    <span className="text-xs ml-1">删除</span>
                                </Button>
                            </ConfirmDialog>
                        </DataTableActionItem>
                    </DataTableAction>
                );
            },
        },
    ];

    const filterForm = useFilterBarForm<z.infer<typeof FileFilterFormSchema>>({
        defaultValues: {
            content_type: filterParam.content_type,
            content_value: filterParam.content_value,
        },
        resolver: zodResolver(FileFilterFormSchema) as any,
        initValues: { content_type: undefined, content_value: undefined },
        onSubmit: (data) => {
            const transformed = data as { status?: number; content_type?: string; content_value?: string };
            searchGo({
                status: transformed.status,
                content_type: transformed.content_type,
                content_value: transformed.content_type ? transformed.content_value : undefined,
                pos: null,
                forward: true,
            });
        },
        onReset: () => {
            searchGo({ pos: null, limit: currentLimit, forward: true, status: undefined, content_type: undefined, content_value: undefined });
        },
    });

    return (
        <div className="container mx-auto p-4 lg:px-6 py-5 max-w-[1600px] flex flex-col min-h-0 space-y-5">
            <div className="flex-shrink-0 mb-1 sm:mb-4 space-y-3">
                <FilterBar form={filterForm} className="bg-card rounded-lg border shadow-sm relative">
                    <FilterBar.Summary>
                        <FilterTotalCount value={formatTotalCount(countNumManager.getTotalInfo())} loading={isLoading} />
                    </FilterBar.Summary>
                    <FilterBar.MobileExtra>
                        <ExportMobileButton activeCount={exportAction.activeCount} isLoading={exportAction.activeCount > 0} onClick={exportAction.openDrawer} />
                    </FilterBar.MobileExtra>
                    <FilterContentSearch
                        typeName="content_type"
                        valueName="content_value"
                        options={CONTENT_SEARCH_TYPES}
                        label="文件搜索"
                        typePlaceholder="选择类型"
                        valuePlaceholder={(type) => {
                            const placeholders: Record<string, string> = {
                                file_md5: '输入文件MD5',
                                source_url: '输入来源URL',
                                url: '输入本地URL',
                            };
                            return placeholders[type] || '请输入...';
                        }}
                        disabled={isLoading}
                    />
                    <FilterActions>
                        <FilterSearchButton loading={isLoading} onRefreshSearch={clearCacheAndReload} />
                        <FilterResetButton loading={isLoading} />
                        <FilterBar.DesktopOnly>
                            <ExportSplitButton activeCount={exportAction.activeCount} onSubmitExport={exportAction.submit}
                                onViewHistory={exportAction.openDrawer} isSubmitting={exportAction.isSubmitting} />
                        </FilterBar.DesktopOnly>
                    </FilterActions>
                    <FilterBar.MobileFooter>
                        {(closeDrawer) => (
                            <ExportButton isSubmitting={exportAction.isSubmitting}
                                onSubmitExport={() => void exportAction.submit().then(closeDrawer).catch(() => {})} />
                        )}
                    </FilterBar.MobileFooter>
                </FilterBar>
                <ExportDrawer
                    open={exportAction.drawerOpen}
                    onOpenChange={(open) => open ? exportAction.openDrawer() : exportAction.closeDrawer()}
                    statusDict={dictData.export_task_status!}
                    tasks={exportAction.tasks} totalCount={exportAction.totalCount}
                    currentPage={exportAction.currentPage} totalPages={exportAction.totalPages}
                    onPageChange={exportAction.setPage} isLoading={exportAction.isLoadingTasks}
                />
            </div>

            <div className="flex-1 flex flex-col min-h-0">
                <div className="flex-1 overflow-hidden">
                    <DataTable
                        data={files}
                        columns={columns}
                        loading={isLoading}
                        error={isError ? <CenteredError error={error} variant="content" onReset={refreshData} /> : null}
                        scrollSnapDelay={300}
                        leftStickyColumns={[{ column: 0, minWidth: '60px', maxWidth: '60px' }]}
                        className="[&_tr]:h-11 [&_td]:py-1 [&_th]:py-1 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b h-full"
                        tableContainerClassName="h-full"
                    />
                </div>
                <div className="flex-shrink-0 pt-4 pb-4">
                    {countNumManager.hasTotalInfo() && (
                        <CursorPagination
                            limit={currentLimit}
                            cursorData={cursorData}
                            searchGo={searchGo}
                            totalInfo={countNumManager.getTotalInfo()}
                            currentPageSize={files.length}
                            loading={isLoading}
                            onRefresh={refreshData}
                            showRefresh={true}
                            showPageSize={true}
                            pageSizeOptions={PAGE_SIZE_OPTIONS}
                            onPageSizeChange={(pageSize) => {
                                searchGo({ limit: pageSize, pos: null, forward: true });
                            }}
                        />
                    )}
                </div>
            </div>

            {detailFile && (
                <FileDetailDrawer
                    open={detailDrawerOpen}
                    onOpenChange={setDetailDrawerOpen}
                    file={detailFile}
                    dictData={dictData}
                />
            )}
        </div>
    );
}

export { FileFilterParamSchema } from './file-schema';
