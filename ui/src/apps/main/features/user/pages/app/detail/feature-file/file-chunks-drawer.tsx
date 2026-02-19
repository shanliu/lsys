import { type UserFileItemType, type UserFileChunkItemType, userFileChunks } from '@shared/apis/user/file'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { CenteredLoading } from '@shared/components/custom/page-placeholder/centered-loading'
import { PagePagination, useCountNumManager } from '@apps/main/lib/pagination-utils'
import { Drawer, DrawerContent, DrawerDescription, DrawerHeader, DrawerTitle } from '@apps/main/components/local/drawer'
import { type TypedDictData } from '@apps/main/hooks/use-dict-data'
import { cn, formatTime, getQueryResponseData, TIME_STYLE, formatFileSize } from '@shared/lib/utils'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useCallback, useState, useEffect } from 'react'

interface FileChunksDrawerProps {
    appId: number
    file: UserFileItemType
    isOpen: boolean
    onOpenChange: (open: boolean) => void
    dictData: TypedDictData<['user_file']>
}

export function FileChunksDrawer({
    appId,
    file,
    isOpen,
    onOpenChange,
    dictData,
}: FileChunksDrawerProps) {
    const queryClient = useQueryClient()
    // 分页状态
    const [page, setPage] = useState(1)
    const pageSize = 10

    // count_num 优化管理器
    const countNumManager = useCountNumManager({})

    // 当文件切换时重置分页和计数管理器
    useEffect(() => {
        setPage(1)
        countNumManager.reset()
    }, [file.id])

    // 获取分片数据 - 只有在抽屉打开时才启用查询
    const { data: chunksData, isSuccess, isLoading, isError, error } = useQuery({
        queryKey: ['userFileChunks', appId, file.id, page],
        queryFn: ({ signal }) => userFileChunks({
            app_id: appId,
            file_id: file.id,
            page: {
                page: page,
                limit: pageSize,
            },
            count_num: countNumManager.getCountNum(),
        }, { signal }),
        enabled: isOpen, // 只有在抽屉打开时才查询
    })

    // 处理 Page 分页查询结果（自动提取 total）
    isSuccess && countNumManager.handlePageQueryResult(chunksData)

    const chunks = getQueryResponseData<UserFileChunkItemType[]>(chunksData, [])

    // 刷新数据
    const refreshData = useCallback(() => {
        queryClient.refetchQueries({ queryKey: ['userFileChunks', appId, file.id] })
    }, [queryClient, appId, file.id])

    // 重置分页当抽屉关闭时
    const handleOpenChange = (open: boolean) => {
        onOpenChange(open)
        if (!open) {
            setPage(1) // 关闭时重置分页
        }
    }

    return (
        <Drawer open={isOpen} onOpenChange={handleOpenChange}>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>文件分片数据</DrawerTitle>
                    <DrawerDescription className={cn("space-y-1")}>
                        <div>文件ID: {file.id}</div>
                        <div className="flex items-center gap-1.5">
                            <span>文件名:</span>
                            <span className="font-medium">{file.file_name || '-'}</span>
                        </div>
                        <div className="flex items-center gap-1.5">
                            <span>文件大小:</span>
                            <span className="font-medium">{formatFileSize(file.file_size || 0)}</span>
                        </div>
                    </DrawerDescription>
                </DrawerHeader>

                <div className="mt-6 space-y-4 flex flex-col flex-1 min-h-0">
                    {/* 分片列表 */}
                    <div className="flex-1 overflow-y-auto space-y-3">
                        {isLoading ? (
                            <CenteredLoading variant="content" iconSize="md" />
                        ) : isError ? (
                            <CenteredError error={error} variant="content" onReset={refreshData} />
                        ) : chunks.length === 0 ? (
                            <div className="text-center text-muted-foreground py-8">暂无分片数据</div>
                        ) : (
                            chunks.map((chunk) => (
                                <div key={chunk.id} className="border rounded-lg p-4 space-y-3 bg-card">
                                    {/* 第一行：分片索引和分片ID */}
                                    <div className="grid grid-cols-2 gap-2">
                                        <div className="flex items-center gap-1.5">
                                            <span className="text-xs text-muted-foreground">分片索引:</span>
                                            <span className="text-xs font-medium">{chunk.chunk_index}</span>
                                        </div>
                                        <div className="flex items-center gap-1.5">
                                            <span className="text-xs text-muted-foreground">分片ID:</span>
                                            <span className="text-xs font-medium">{chunk.id}</span>
                                        </div>
                                    </div>

                                    {/* 第二行：分片大小和已完成大小 */}
                                    <div className="grid grid-cols-2 gap-2">
                                        <div className="flex items-center gap-1.5">
                                            <span className="text-xs text-muted-foreground">分片大小:</span>
                                            <span className="text-xs font-medium">{formatFileSize(chunk.file_size || 0)}</span>
                                        </div>
                                        <div className="flex items-center gap-1.5">
                                            <span className="text-xs text-muted-foreground">已完成:</span>
                                            <span className="text-xs font-medium">{formatFileSize(chunk.complete_size || 0)}</span>
                                        </div>
                                    </div>

                                    {/* 第三行：起始偏移和状态 */}
                                    <div className="grid grid-cols-2 gap-2">
                                        <div className="flex items-center gap-1.5">
                                            <span className="text-xs text-muted-foreground">起始偏移:</span>
                                            <span className="text-xs font-medium">{chunk.start_offset}</span>
                                        </div>
                                        <div className="flex items-center gap-1.5">
                                            <span className="text-xs text-muted-foreground">状态:</span>
                                            <span className="text-xs font-medium">{dictData.file_chunk_status?.getLabel(String(chunk.status)) || String(chunk.status)}</span>
                                        </div>
                                    </div>

                                    {/* 第四行：MD5信息 */}
                                    {(chunk.chunk_md5 || chunk.upload_md5) && (
                                        <div className="space-y-1.5">
                                            {chunk.chunk_md5 && (
                                                <div className="flex gap-1.5">
                                                    <span className="text-xs text-muted-foreground whitespace-nowrap">分片MD5:</span>
                                                    <span className="text-xs break-words flex-1 font-mono text-muted-foreground">{chunk.chunk_md5}</span>
                                                </div>
                                            )}
                                            {chunk.upload_md5 && (
                                                <div className="flex gap-1.5">
                                                    <span className="text-xs text-muted-foreground whitespace-nowrap">上传MD5:</span>
                                                    <span className="text-xs break-words flex-1 font-mono text-muted-foreground">{chunk.upload_md5}</span>
                                                </div>
                                            )}
                                        </div>
                                    )}

                                    {/* 第五行：时间信息 */}
                                    <div className="grid grid-cols-2 gap-2 text-xs">
                                        <div className="flex items-center gap-1.5">
                                            <span className="text-muted-foreground">添加时间:</span>
                                            <span>{formatTime(chunk.add_time, TIME_STYLE.ABSOLUTE_TEXT)}</span>
                                        </div>
                                        {chunk.change_time && (
                                            <div className="flex items-center gap-1.5">
                                                <span className="text-muted-foreground">更新时间:</span>
                                                <span>{formatTime(chunk.change_time, TIME_STYLE.ABSOLUTE_TEXT)}</span>
                                            </div>
                                        )}
                                    </div>

                                    {/* 分片路径 */}
                                    {chunk.chunk_path && (
                                        <div className="flex gap-1.5">
                                            <span className="text-xs text-muted-foreground whitespace-nowrap">分片路径:</span>
                                            <span className="text-xs break-words flex-1 font-mono text-muted-foreground">{chunk.chunk_path}</span>
                                        </div>
                                    )}
                                </div>
                            ))
                        )}
                    </div>

                    {/* 分页 */}
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
    )
}
