import { Drawer, DrawerContent, DrawerDescription, DrawerHeader, DrawerTitle } from '@apps/main/components/local/drawer'
import { PagePagination, usePageCountNum } from '@apps/main/lib/pagination-utils'
import { type UserFileItemType, type UserFileLogItemType, userFileLogs } from '@shared/apis/user/file'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { CenteredLoading } from '@shared/components/custom/page-placeholder/centered-loading'
import { cn, formatTime, getQueryResponseData, TIME_STYLE } from '@shared/lib/utils'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useCallback, useEffect, useState } from 'react'

interface FileLogsDrawerProps {
    appId: number
    file: UserFileItemType
    isOpen: boolean
    onOpenChange: (open: boolean) => void
}

export function FileLogsDrawer({
    appId,
    file,
    isOpen,
    onOpenChange,
}: FileLogsDrawerProps) {
    const queryClient = useQueryClient()
    // 分页状态
    const [page, setPage] = useState(1)
    const pageSize = 10

    // count_num 优化管理器
    const countNumManager = usePageCountNum({})
    const { reset: resetCountNum } = countNumManager

    // 当文件切换时重置分页和计数管理器
    useEffect(() => {
        setPage(1)
        resetCountNum()
    }, [file.id, resetCountNum])

    // 获取日志数据 - 只有在抽屉打开时才启用查询
    const { data: logsData, isSuccess, isLoading, isError, error } = useQuery({
        queryKey: ['userFileLogs', appId, file.id, page],
        queryFn: ({ signal }) => userFileLogs({
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
    isSuccess && countNumManager.handleQueryResult(logsData)

    const logs = getQueryResponseData<UserFileLogItemType[]>(logsData, [])

    // 刷新数据
    const refreshData = useCallback(() => {
        queryClient.refetchQueries({ queryKey: ['userFileLogs', appId, file.id] })
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
                    <DrawerTitle>文件操作日志</DrawerTitle>
                    <DrawerDescription className={cn("space-y-1")}>
                        <div>文件ID: {file.id}</div>
                        <div className="flex items-center gap-1.5">
                            <span>文件名:</span>
                            <span className="font-medium">{file.file_name || '-'}</span>
                        </div>
                    </DrawerDescription>
                </DrawerHeader>

                <div className="mt-6 space-y-4 flex flex-col flex-1 min-h-0">
                    {/* 日志列表 */}
                    <div className="flex-1 overflow-y-auto space-y-3">
                        {isLoading ? (
                            <CenteredLoading variant="content" iconSize="md" />
                        ) : isError ? (
                            <CenteredError error={error} variant="content" onReset={refreshData} />
                        ) : logs.length === 0 ? (
                            <div className="text-center text-muted-foreground py-8">暂无日志</div>
                        ) : (
                            logs.map((log) => (
                                <div key={log.id} className="border rounded-lg p-4 space-y-2 bg-card">
                                    {/* 第一行：分片ID和操作用户 */}
                                    <div className="grid grid-cols-2 gap-2">
                                        <div className="flex items-center gap-1.5">
                                            <span className="text-xs text-muted-foreground">分片ID:</span>
                                            <span className="text-xs font-medium">{log.file_chunk_id || '-'}</span>
                                        </div>
                                        <div className="flex items-center gap-1.5">
                                            <span className="text-xs text-muted-foreground">操作用户:</span>
                                            <span className="text-xs font-medium">{log.user_data?.user_nickname || log.user_data?.user_account || '-'}</span>
                                        </div>
                                    </div>

                                    {/* 第二行：时间 */}
                                    <div className="flex items-center gap-1.5">
                                        <span className="text-xs text-muted-foreground">时间:</span>
                                        <span className="text-xs">{formatTime(log.add_time, TIME_STYLE.ABSOLUTE_TEXT)}</span>
                                    </div>

                                    {/* 第三行：日志内容 */}
                                    <div className="flex gap-1.5">
                                        <span className="text-xs text-muted-foreground whitespace-nowrap">日志内容:</span>
                                        <span className="text-xs break-words flex-1">{log.message}</span>
                                    </div>
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
