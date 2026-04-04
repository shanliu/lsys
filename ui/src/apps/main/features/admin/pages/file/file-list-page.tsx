import { FilterContainer } from '@apps/main/components/filter-container/container'
import { FilterActions } from '@apps/main/components/filter-container/filter-actions'
import { FilterContentSearch } from '@apps/main/components/filter-container/filter-content-search'
import { FilterTotalCount } from '@apps/main/components/filter-container/filter-total-count'
import {
    CursorPagination,
    DEFAULT_PAGE_SIZE,
    PAGE_SIZE_OPTIONS,
    useLimitCountNum,
    useSearchNavigate,
} from '@apps/main/lib/pagination-utils'
import { createStatusMapper } from '@apps/main/lib/status-utils'
import { Route } from '@apps/main/routes/_main/admin/file/list'
import { useDictData } from '@apps/main/hooks/use-dict-data'
import { zodResolver } from '@hookform/resolvers/zod'
import {
    adminFileDelete,
    adminFileList,
    type AdminFileItemType,
} from '@shared/apis/admin/file'
import { DataTable } from '@shared/components/custom//table'
import { ConfirmDialog } from '@shared/components/custom/dialog/confirm-dialog'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { DataTableAction, DataTableActionItem } from '@shared/components/custom/table'
import { Badge } from '@shared/components/ui/badge'
import { Button } from '@shared/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@shared/components/ui/tooltip'
import { useToast } from '@shared/contexts/toast-context'
import {
    cn,
    formatFileSize,
    formatTime,
    getQueryResponseCursor,
    getQueryResponseData,
    TIME_STYLE,
} from '@shared/lib/utils'
import { formatTotalCount } from '@shared/lib/utils/format-utils'
import { type LimitType } from '@shared/types/base-schema'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { type ColumnDef } from '@tanstack/react-table'
import { Cloud, Eye, Trash2 } from 'lucide-react'
import React, { useState } from 'react'
import { AdminFileDetailDrawer } from './file-detail-drawer'
import { AdminFileListFilterFormSchema, CONTENT_SEARCH_TYPES } from './file-list-schema'

export function AdminFileListPage() {
    const queryClient = useQueryClient()
    const { success: showSuccess, error: showError } = useToast()
    const navigate = useNavigate()

    const filterParam = Route.useSearch()
    const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE

    const [detailDrawerOpen, setDetailDrawerOpen] = useState(false)
    const [detailFile, setDetailFile] = useState<AdminFileItemType | null>(null)

    // 加载文件管理字典
    const { dictData: adminFileDict } = useDictData(['admin_file'] as const)

    const filters = {
        status: filterParam.status || null,
        content_type: filterParam.content_type || null,
        content_value: filterParam.content_value || null,
    }

    const pagination: LimitType = {
        pos: filterParam.pos || null,
        limit: filterParam.limit || DEFAULT_PAGE_SIZE,
        forward: filterParam.forward ?? true,
        more: true,
    }

    const searchGo = useSearchNavigate(navigate, filterParam)
    const countNumManager = useLimitCountNum(filters)

    const contentFilterParams = React.useMemo(() => {
        if (!filters.content_type || !filters.content_value) return {}
        const map: Record<string, string> = {
            file_md5: 'file_md5',
            source_url: 'source_url',
            url: 'url',
        }
        const key = map[filters.content_type]
        return key ? { [key]: filters.content_value } : {}
    }, [filters.content_type, filters.content_value])

    const { data: fileData, isSuccess, isLoading, isError, error } = useQuery({
        queryKey: [
            'adminFileList',
            pagination.pos, currentLimit, pagination.forward, pagination.more,
            filters.status, filters.content_type, filters.content_value,
        ],
        queryFn: ({ signal }) =>
            adminFileList(
                {
                    limit: pagination,
                    count_num: countNumManager.getCountNum(),
                    status: filters.status ?? undefined,
                    ...contentFilterParams,
                },
                { signal },
            ),
        placeholderData: (previousData) => previousData,
    })

    isSuccess && countNumManager.handleQueryResult(fileData)

    const files = getQueryResponseData<AdminFileItemType[]>(fileData, [])
    const cursorData = getQueryResponseCursor(fileData)

    const deleteFileMutation = useMutation({
        mutationFn: (params: { file_user_id: number }) =>
            adminFileDelete({ file_user_id: params.file_user_id }),
        onSuccess: () => {
            showSuccess('文件已删除')
            queryClient.invalidateQueries({ queryKey: ['adminFileList'] })
        },
        onError: (error: any) => {
            showError(error?.data?.message || error?.message || '删除文件失败')
        },
    })

    const handleDeleteFile = async (fileUserId: number) => {
        await deleteFileMutation.mutateAsync({ file_user_id: fileUserId })
    }

    const refreshData = () => {
        queryClient.refetchQueries({ queryKey: ['adminFileList'] })
    }

    const clearCacheAndReload = () => {
        countNumManager.reset()
        queryClient.invalidateQueries({ queryKey: ['adminFileList'] })
    }

    const fileStatus = createStatusMapper(
        {
            1: 'success',
            2: 'danger',
            3: 'info',
            4: 'danger',
        },
        (status) => adminFileDict?.file_status?.getLabel(String(status), '') ?? '',
    )

    const columns: ColumnDef<AdminFileItemType>[] = [
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
                const fileName = row.original.file_name
                const storageType = row.original.storage_type
                const isCloud = storageType && storageType !== 'local'
                return (
                    <div className="flex items-center gap-1 py-1 max-w-[220px]">
                        <span className="truncate text-sm" title={fileName}>
                            {fileName || '-'}
                        </span>
                        {isCloud && (
                            <TooltipProvider delayDuration={200}>
                                <Tooltip>
                                    <TooltipTrigger asChild>
                                        <Cloud className="h-3.5 w-3.5 flex-shrink-0 text-blue-400 cursor-pointer" />
                                    </TooltipTrigger>
                                    <TooltipContent side="top">
                                        <span>{storageType}</span>
                                    </TooltipContent>
                                </Tooltip>
                            </TooltipProvider>
                        )}
                    </div>
                )
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
                <div className="max-w-[120px] truncate py-1 text-xs text-muted-foreground" title={getValue<string>() || ''}>
                    {getValue<string>() || '-'}
                </div>
            ),
        },
        {
            accessorKey: 'user_id',
            header: '用户ID',
            size: 80,
            cell: ({ getValue }) => (
                <div className="py-1 text-xs text-muted-foreground">{getValue<number>()}</div>
            ),
        },
        {
            accessorKey: 'status',
            header: '状态',
            size: 80,
            cell: ({ getValue }) => {
                const status = getValue<number>()
                return (
                    <div className="py-1">
                        <Badge className={cn(fileStatus.getClass(status))}>
                            {fileStatus.getText(status)}
                        </Badge>
                    </div>
                )
            },
        },
        {
            accessorKey: 'add_time',
            header: '添加时间',
            size: 120,
            cell: ({ getValue }) => {
                const addTime = getValue<Date | null>()
                return (
                    <div className="text-xs py-1">
                        {addTime ? formatTime(addTime, TIME_STYLE.RELATIVE_ELEMENT) : '-'}
                    </div>
                )
            },
        },
        {
            id: 'actions',
            header: () => <div className="text-center py-1">操作</div>,
            size: 100,
            cell: ({ row }) => {
                const file = row.original
                return (
                    <DataTableAction className="justify-end sm:justify-center">
                        <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                            <Button variant="ghost" size="sm" className={cn('h-auto px-2 py-1')} title="详细信息"
                                onClick={() => { setDetailFile(file); setDetailDrawerOpen(true) }}>
                                <Eye className="h-3 w-3" />
                                <span className="text-xs ml-1">详细</span>
                            </Button>
                        </DataTableActionItem>
                        <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                            <ConfirmDialog
                                title="确认删除"
                                description={<>您确定要删除文件 <strong>{file.file_name}</strong> 吗？删除后将无法恢复。</>}
                                onConfirm={async () => await handleDeleteFile(file.id)}
                            >
                                <Button variant="ghost" size="sm"
                                    className={cn('h-auto px-2 py-1 text-destructive hover:text-destructive')} title="删除文件">
                                    <Trash2 className="h-3 w-3" />
                                    <span className="text-xs ml-1">删除</span>
                                </Button>
                            </ConfirmDialog>
                        </DataTableActionItem>
                    </DataTableAction>
                )
            },
        },
    ]

    return (
        <div className="container mx-auto p-4 lg:px-6 py-5 max-w-[1600px] flex flex-col min-h-0 space-y-5">
            <div className="flex-shrink-0 mb-1 sm:mb-4 space-y-3">
                <FilterContainer
                    defaultValues={{
                        status: filterParam.status?.toString(),
                        content_type: filterParam.content_type,
                        content_value: filterParam.content_value,
                    }}
                    resolver={zodResolver(AdminFileListFilterFormSchema) as any}
                    onSubmit={(data) => {
                        const transformedData = data as {
                            status?: number; content_type?: string; content_value?: string
                        }
                        searchGo({
                            status: transformedData.status,
                            content_type: transformedData.content_type,
                            content_value: transformedData.content_type ? transformedData.content_value : undefined,
                            pos: null, forward: true,
                        })
                    }}
                    onReset={() => {
                        searchGo({
                            pos: null, limit: currentLimit, forward: true,
                            status: undefined, content_type: undefined, content_value: undefined,
                        })
                    }}
                    countComponent={
                        <FilterTotalCount value={formatTotalCount(countNumManager.getTotalInfo())} loading={isLoading} />
                    }
                    className="bg-card rounded-lg border shadow-sm relative"
                >
                    {(layoutParams, form) => (
                        <div className="flex-1 flex flex-wrap items-end gap-3">
                            <FilterContentSearch typeName="content_type" valueName="content_value"
                                options={CONTENT_SEARCH_TYPES as unknown as { value: string; label: string }[]}
                                label="文件搜索" typePlaceholder="选择类型"
                                valuePlaceholder={(type) => {
                                    const placeholders: Record<string, string> = {
                                        file_md5: '输入文件MD5', source_url: '输入来源URL', url: '输入本地URL',
                                    }
                                    return placeholders[type] || '请输入...'
                                }}
                                disabled={isLoading} layoutParams={layoutParams} />
                            <div className={cn(layoutParams.isMobile ? 'w-full' : 'flex-shrink-0')}>
                                <FilterActions form={form} loading={isLoading} layoutParams={layoutParams}
                                    onRefreshSearch={clearCacheAndReload} />
                            </div>
                        </div>
                    )}
                </FilterContainer>
            </div>

            <div className="flex-1 flex flex-col min-h-0">
                <div className="flex-1 overflow-hidden">
                    <DataTable data={files} columns={columns} loading={isLoading}
                        error={isError ? <CenteredError error={error} variant="content" onReset={refreshData} /> : null}
                        scrollSnapDelay={300}
                        leftStickyColumns={[{ column: 0, minWidth: '60px', maxWidth: '60px' }]}
                        className="[&_tr]:h-11 [&_td]:py-1 [&_th]:py-1 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b h-full"
                        tableContainerClassName="h-full" />
                </div>
                <div className="flex-shrink-0 pt-4 pb-4">
                    {countNumManager.hasTotalInfo() && (
                        <CursorPagination limit={currentLimit} cursorData={cursorData}
                            searchGo={searchGo} totalInfo={countNumManager.getTotalInfo()}
                            currentPageSize={files.length} loading={isLoading} onRefresh={refreshData}
                            showRefresh={true} showPageSize={true} pageSizeOptions={PAGE_SIZE_OPTIONS}
                            onPageSizeChange={(pageSize) => { searchGo({ limit: pageSize, pos: null, forward: true }) }} />
                    )}
                </div>
            </div>

            {detailFile && (
                <AdminFileDetailDrawer open={detailDrawerOpen} onOpenChange={setDetailDrawerOpen} file={detailFile} />
            )}
        </div>
    )
}
