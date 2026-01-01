import { SubAppItemType, subAppList, SubAppListParamType } from '@shared/apis/admin/app'
import { UserDataTooltip } from '@apps/main/components/local/user-data-tooltip'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { PagePagination } from '@shared/components/custom/pagination/page'
import { DataTable, DataTableAction, DataTableActionItem } from '@shared/components/custom/table'
import { Badge } from '@shared/components/ui/badge'
import { Button } from '@shared/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@shared/components/ui/card'
import { Drawer, DrawerContent, DrawerHeader, DrawerTitle } from '@apps/main/components/local/drawer'
import { cn, formatTime, TIME_STYLE } from '@shared/lib/utils'
import { createStatusMapper } from '@apps/main/lib/status-utils'
import { useIsMobile } from '@shared/hooks/use-mobile'
import { type DictList } from '@shared/types/apis-dict'
import { useQuery } from '@tanstack/react-query'
import { ColumnDef } from '@tanstack/react-table'
import { Eye, Globe, Users } from 'lucide-react'
import { useCallback, useMemo, useState } from 'react'
import { SubAppDetailDrawer } from './list-sub-app-detail-drawer'
import { PageSkeletonTable } from '@shared/components/custom/page-placeholder/skeleton-table'

interface ListSubAppDrawerProps {
  /** 父应用ID */
  appId: number | null
  /** 是否打开抽屉 */
  open: boolean
  /** 关闭抽屉的回调 */
  onClose: () => void
  /** 应用状态字典 */
  appStatusDict: DictList
}

/**
 * 子应用列表抽屉组件
 * @description 显示指定应用的子应用列表
 */
export function ListSubAppDrawer({ appId, open, onClose, appStatusDict }: ListSubAppDrawerProps) {
  const isMobile = useIsMobile()
  
  // 分页状态
  const [pagination, setPagination] = useState({
    current: 1,
    pageSize: 10,
  })

  // 详细抽屉状态
  const [detailDrawer, setDetailDrawer] = useState({
    open: false,
    subApp: null as SubAppItemType | null,
  })

  // 构建查询参数
  const queryParams: SubAppListParamType = {
    app_id: appId || 0,
    page: {
      page: pagination.current,
      limit: pagination.pageSize
    },
    count_num: true,
  }

  // 获取子应用列表数据
  const { data: subAppData, isLoading, isError, error, refetch } = useQuery({
    queryKey: ['subAppList', appId, pagination.current, pagination.pageSize],
    queryFn: async ({ signal }) => {
      if (!appId) return null
      const result = await subAppList(queryParams, { signal })
      if (!result.status) {
        // 如果API返回失败，抛出完整的错误信息
        throw result
      }
      return result.response || null
    },
    enabled: open && !!appId,
  })

  // 状态样式映射
  const statusMapper = useMemo(
    () =>
      createStatusMapper(
        { 1: 'neutral', 2: 'success', 3: 'warning' },
        (status: number) => {
          return appStatusDict.getLabel(String(status)) || String(status)
        },
      ),
    [appStatusDict],
  )

  // 处理分页变化
  const handlePageChange = useCallback((page: number) => {
    setPagination(prev => ({ ...prev, current: page }))
  }, [])

  // 刷新数据
  const handleRefresh = useCallback(() => {
    refetch()
  }, [refetch])

  // 定义表格列
  const columns: ColumnDef<SubAppItemType>[] = [
    {
      accessorKey: 'id',
      header: 'ID',
      size: 80,
      cell: ({ getValue }) => (
        <div className="font-mono text-xs">{getValue<number>()}</div>
      ),
    },
    {
      accessorKey: 'name',
      header: '应用名称',
      cell: ({ getValue }) => {
        const name = getValue<string>()
        return (
          <div className="font-medium">
            {name}
          </div>
        )
      },
    },
    {
      accessorKey: 'client_id',
      header: '客户端ID',
      cell: ({ getValue }) => {
        const clientId = getValue<string | null | undefined>()
        return clientId ? (
          <div className="font-mono text-xs">{clientId}</div>
        ) : (
          <span className="text-muted-foreground">-</span>
        )
      },
    },
    {
      accessorKey: 'status',
      header: '状态',
      cell: ({ getValue }) => {
        const status = getValue<number>()
        return (
          <Badge
            variant="secondary"
            className={statusMapper.getClass(status)}
          >
            {statusMapper.getText(status)}
          </Badge>
        )
      },
    },
    {
      accessorKey: 'user_data',
      header: '所属用户',
      cell: ({ row }) => {
        return (
          <UserDataTooltip
            userData={row.original.user_data}
            className={cn("text-xs")}
          />
        )
      },
    },
    {
      accessorKey: 'change_time',
      header: '修改时间',
      cell: ({ getValue }) => {
        const date = getValue<Date>()
        return (
          <div className="text-xs text-muted-foreground">
            {formatTime(date, TIME_STYLE.ABSOLUTE_TEXT)}
          </div>
        )
      },
    },
    {
      id: 'actions',
      header: () => <div className="text-center">详细</div>,
      size: 80,
      cell: ({ row }) => {
        const subApp = row.original
        return (
          <DataTableAction className={cn(isMobile ? "justify-end" : "justify-center")}>
            <DataTableActionItem mobileDisplay="display" desktopDisplay="display">
              <Button
                variant="ghost"
                size="sm"
                className={cn("h-7 px-2")}
                onClick={() => setDetailDrawer({ open: true, subApp: subApp })}
                title="查看详情"
              >
                <Eye className=" h-4 w-4" />
                {isMobile ? <span className="ml-2">查看详情</span> : null}
              </Button>
            </DataTableActionItem>
          </DataTableAction>
        )
      },
    },
  ]

  const apps = subAppData?.data || []
  const total = subAppData?.total || 0

  return (
    <Drawer open={open} onOpenChange={(open) => !open && onClose()}>
      <DrawerContent 
        className="!w-[1200px] !max-w-[95vw]"
        contentClassName="py-0"
      >
        <DrawerHeader className="pt-6">
          <DrawerTitle className={cn("flex items-center gap-2")}>
            <Users className={cn("h-5 w-5")} />
            子应用列表
          </DrawerTitle>
        </DrawerHeader>

        <div className="mt-6 space-y-4 pb-6">
          {isLoading && (
            <PageSkeletonTable variant="content" />
          )}

          {isError && (
            <CenteredError
              variant="content"
              error={error}
              onReset={() => refetch()}
            />
          )}

          {!isLoading && !isError && (
            <>
              {total === 0 ? (
                <div className="flex flex-col items-center justify-center py-12">
                  <Globe className={cn("h-12 w-12 text-muted-foreground/50 mb-4")} />
                  <h3 className="text-lg font-medium text-muted-foreground mb-2">暂无子应用</h3>
                  <p className="text-sm text-muted-foreground">该应用还没有创建任何子应用</p>
                </div>
              ) : (
                <>
                  {/* 数据表格 */}
                  {isMobile ? (
                    // 移动端：不使用 Card 包装，避免重复边框
                    <div className={cn("space-y-2")}>
                      <div className="flex items-center justify-between">
                        <span className="text-sm text-muted-foreground">
                          共 {total} 个子应用
                        </span>
                      </div>
                      <DataTable
                        data={apps}
                        columns={columns}
                        loading={isLoading}
                        error={isError ? <CenteredError error={error} variant="content" /> : null}
                        className={cn("[&_.data-table-row]:h-12 [&_td]:py-2 [&_th]:py-2 border-0 rounded-none -mx-4")}
                      />
                      <PagePagination
                        currentPage={pagination.current}
                        pageSize={pagination.pageSize}
                        total={total}
                        loading={isLoading}
                        onChange={handlePageChange}
                        onRefresh={handleRefresh}
                        showRefresh={true}
                      />
                    </div>
                  ) : (
                    // 桌面端：使用 Card 包装
                    <Card>
                      <CardHeader className={cn("pb-3")}>
                        <CardTitle className={cn("flex items-center justify-between text-base")}>
                          <span>子应用列表</span>
                          <span className="text-sm font-normal text-muted-foreground">
                            共 {total} 个子应用
                          </span>
                        </CardTitle>
                      </CardHeader>
                      <CardContent className={cn("space-y-4")}>
                        <DataTable
                          data={apps}
                          columns={columns}
                          loading={isLoading}
                          error={isError ? <CenteredError error={error} variant="content" /> : null}
                          className={cn("[&_.data-table-row]:h-12 [&_td]:py-2 [&_th]:py-2")}
                        />
                        <PagePagination
                          currentPage={pagination.current}
                          pageSize={pagination.pageSize}
                          total={total}
                          loading={isLoading}
                          onChange={handlePageChange}
                          onRefresh={handleRefresh}
                          showRefresh={true}
                        />
                      </CardContent>
                    </Card>
                  )}
                </>
              )}
            </>
          )}
        </div>
      </DrawerContent>

      {/* 子应用详情抽屉 - 嵌套在子应用列表抽屉之上 */}
      <SubAppDetailDrawer
        subApp={detailDrawer.subApp}
        open={detailDrawer.open}
        onClose={() => setDetailDrawer({ open: false, subApp: null })}
        appStatusDict={appStatusDict}
      />
    </Drawer>
  )
}
