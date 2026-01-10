import {
  appRbacResDelete,
  appRbacResList,
  type AppRbacResDataItemType,
} from '@shared/apis/user/rbac'
import { ConfirmDialog } from '@shared/components/custom/dialog/confirm-dialog'
import { FilterContainer } from '@apps/main/components/filter-container/container'
import { FilterActions } from '@apps/main/components/filter-container/filter-actions'
import { FilterInput } from '@apps/main/components/filter-container/filter-input'
import { FilterTotalCount } from '@apps/main/components/filter-container/filter-total-count'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import {
  DEFAULT_PAGE_SIZE,
  PagePagination,
  useCountNumManager,
} from '@apps/main/lib/pagination-utils'
import { DataTable, DataTableAction, DataTableActionItem } from '@shared/components/custom/table'
import { Badge } from '@shared/components/ui/badge'
import { Button } from '@shared/components/ui/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@shared/components/ui/tooltip'
import { useToast } from '@shared/contexts/toast-context'
import { AppDetailNavContainer } from '@apps/main/features/user/components/ui/app-detail-nav'
import { FilterUserMode, type RbacUserModeContext } from '@apps/main/features/user/components/ui/filter-user-mode'
import { useIsMobile } from '@shared/hooks/use-mobile'
import {
  cn,
  formatServerError,
  getQueryResponseData,
} from '@shared/lib/utils'
import { Route } from '@apps/main/routes/_main/user/app/$appId/features-rbac/resource'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { type ColumnDef } from '@tanstack/react-table'
import { Cog, Edit, Plus, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { featureRbacModuleConfig } from '../nav-info'
import { ResDrawer } from './res-drawer'
import { ResOpListDrawer } from './res-op-list-drawer'
import {
  ResListFilterFormSchema
} from './res-schema'
import { ResTypeOpsDrawer } from './res-type-ops-drawer'
import { UserDataTooltip } from '@/apps/main/components/local/user-data-tooltip'
import { UserDataResType } from '@/shared/types/base-schema'

export default function AppDetailFeatureRbacResPage() {
  const { appId } = Route.useParams()
  const [drawerOpen, setDrawerOpen] = useState(false)
  const [opListDrawerOpen, setOpListDrawerOpen] = useState(false)

  const handleAdd = () => {
    setDrawerOpen(true)
  }

  const actions = (
    <>
      <Button variant="outline" size="sm" onClick={() => setOpListDrawerOpen(true)}>
        <Cog className="mr-2 h-4 w-4" />
        操作管理
      </Button>
      <Button variant="outline" size="sm" onClick={handleAdd}>
        <Plus className="mr-2 h-4 w-4" />
        新增资源
      </Button>
    </>
  )

  return (
    <AppDetailNavContainer {...featureRbacModuleConfig} actions={actions}>
      <ResListContent 
        appId={appId} 
        drawerOpen={drawerOpen} 
        setDrawerOpen={setDrawerOpen}
        opListDrawerOpen={opListDrawerOpen}
        setOpListDrawerOpen={setOpListDrawerOpen}
      />
    </AppDetailNavContainer>
  )
}

// ============= 资源列表组件 =============
interface ResListContentProps {
  appId: number
  drawerOpen: boolean
  setDrawerOpen: (open: boolean) => void
  opListDrawerOpen: boolean
  setOpListDrawerOpen: (open: boolean) => void
}

function ResListContent({ appId, drawerOpen, setDrawerOpen, opListDrawerOpen, setOpListDrawerOpen }: ResListContentProps) {
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const isMobile = useIsMobile()
  const toast = useToast()

  // 状态管理 - 全部在内部定义
  const [editingRes, setEditingRes] = useState<AppRbacResDataItemType | undefined>()
  const [opsDrawerOpen, setOpsDrawerOpen] = useState(false)
  const [selectedResType, setSelectedResType] = useState('')

  // 用户模式状态
  const [userMode, setUserMode] = useState<RbacUserModeContext>({
    use_app_user: true,
    user_param: '',
  })

  // 从 URL 搜索参数获取过滤条件
  const filterParam = Route.useSearch()
  const currentPage = filterParam.page || 1
  const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE

  // 过滤条件
  const filters = {
    res_name: filterParam.res_name ?? null,
    res_type: filterParam.res_type ?? null,
    res_data: filterParam.res_data ?? null,
  }

  // count_num 优化管理器
  const countNumManager = useCountNumManager(filters)

  // 获取资源列表
  const { data: resData, isSuccess, isLoading, isError, error } = useQuery({
    queryKey: [
      'rbac-res-list',
      appId,
      userMode.use_app_user,
      userMode.user_param,
      filterParam.res_name,
      filterParam.res_type,
      filterParam.res_data,
      currentPage,
      currentLimit,
    ],
    queryFn: async ({ signal }) => {
      const result = await appRbacResList(
        {
          app_id: Number(appId),
          use_app_user: userMode.use_app_user,
          user_param: userMode.user_param,
          res_name: filterParam.res_name,
          res_type: filterParam.res_type,
          res_data: filterParam.res_data,
          page: {
            page: currentPage,
            limit: currentLimit,
          },
          count_num: countNumManager.getCountNum(),
        },
        { signal }
      )
      return result
    },
    placeholderData: (previousData) => previousData,
  })

  // 处理分页查询结果
  isSuccess && countNumManager.handlePageQueryResult(resData)

  // 从查询结果中提取数据
  const resources = getQueryResponseData<AppRbacResDataItemType[]>(resData, [])

  // 删除资源
  const deleteMutation = useMutation({
    mutationFn: (resId: number) => appRbacResDelete({ res_id: resId }),
    onSuccess: () => {
      toast.success('资源删除成功')
      countNumManager.reset()
      queryClient.invalidateQueries({ queryKey: ['rbac-res-list'] })
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  // 刷新数据
  const refreshData = () => {
    queryClient.refetchQueries({ queryKey: ['rbac-res-list'] })
  }

  // 清除缓存并重新加载
  const clearCacheAndReload = () => {
    countNumManager.reset()
    queryClient.invalidateQueries({ queryKey: ['rbac-res-list'] })
  }

  // 打开编辑抽屉
  const handleEdit = (res: AppRbacResDataItemType) => {
    setEditingRes(res)
    setDrawerOpen(true)
  }

  // 打开操作管理抽屉
  const handleManageOps = (resType: string) => {
    setSelectedResType(resType)
    setOpsDrawerOpen(true)
  }

  // 定义表格列配置
  const columns: ColumnDef<AppRbacResDataItemType>[] = [
    {
      accessorKey: 'id',
      header: () => <div className={cn(isMobile ? '' : 'text-right')}>ID</div>,
      size: 80,
      cell: ({ getValue }) => (
        <div className={cn('font-mono text-sm', isMobile ? '' : 'text-right')}>
          {getValue<number>()}
        </div>
      ),
    },
    {
      accessorKey: 'res_name',
      header: '资源名称',
      cell: ({ getValue }) => (
        <div className="font-medium">{getValue<string>()}</div>
      ),
    },
    {
      accessorKey: 'res_type',
      header: '资源类型',
      cell: ({ getValue }) => (
        <Badge variant="outline">{getValue<string>()}</Badge>
      ),
    },
    {
      accessorKey: 'res_data',
      header: '资源数据',
      cell: ({ getValue }) => {
        const value = getValue<string>()
        return value ? (
          <code className="text-sm bg-muted px-1.5 py-0.5 rounded">
            {value}
          </code>
        ) : (
          <span className="text-muted-foreground">-</span>
        )
      },
    },
    {
      accessorKey: 'op_count',
      header: () => <div className="text-center">已关联操作</div>,
      size: 80,
      cell: ({ getValue, row }) => {
        const count = getValue<number | null | undefined>()
        const res = row.original
        return (
          <div className="text-center">
            <Tooltip>
              <TooltipTrigger asChild>
                <Badge
                  variant="secondary"
                  className="font-mono cursor-pointer hover:bg-secondary/80"
                  onClick={() => handleManageOps(res.res_type)}
                >
                  {count ?? 0}
                </Badge>
              </TooltipTrigger>
              <TooltipContent>关联操作</TooltipContent>
            </Tooltip>
          </div>
        )
      },
    },
    {
      accessorKey: 'perm_count',
      header: () => <div className="text-center">已关联权限</div>,
      size: 80,
      cell: ({ getValue }) => {
        const count = getValue<number | null | undefined>()
        return (
          <div className="text-center">
            <Badge variant="secondary" className="font-mono">
              {count ?? 0}
            </Badge>
          </div>
        )
      },
    },
    {
      accessorKey: 'user_data',
      header: '创建者',
      size: 100,
      cell: ({ getValue }) => {
        const userData = getValue<UserDataResType | null>()
        return <UserDataTooltip userData={userData} />
      },
    },
    {
      id: 'actions',
      header: () => <div className="text-center">操作</div>,
      size: 160,
      cell: ({ row }) => {
        const res = row.original

        return (
          <DataTableAction className={cn(isMobile ? 'justify-end' : 'justify-center')}>
            <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
              <Button
                variant="ghost"
                size="sm"
                className={cn('h-7 px-2')}
                onClick={() => handleEdit(res)}
              >
                <Edit className="h-4 w-4" />
                <span className="ml-2">编辑信息</span>
              </Button>
            </DataTableActionItem>
            <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
              <ConfirmDialog
                title="删除资源"
                description={`确定要删除资源「${res.res_name}」吗？删除后无法恢复。`}
                onConfirm={async () => {
                  await deleteMutation.mutateAsync(res.id)
                }}
              >
                <Button
                  variant="ghost"
                  size="sm"
                  className={cn('h-7 px-2 ')}
                >
                  <Trash2 className="h-4 w-4" />
                  <span className="ml-2">删除资源</span>
                </Button>
              </ConfirmDialog>
            </DataTableActionItem>
          </DataTableAction>
        )
      },
    },
  ]

  return (
    <div className="flex flex-col min-h-0 space-y-3">
      {/* 过滤器 */}
      <div className="flex-shrink-0 mb-1 sm:mb-4">
        <FilterContainer
          defaultValues={{
            res_name: filterParam.res_name,
            res_type: filterParam.res_type,
            res_data: filterParam.res_data,
          }}
          resolver={zodResolver(ResListFilterFormSchema) as any}
          onSubmit={(data) => {
            navigate({
              search: { ...data, page: 1, limit: currentLimit } as any,
            })
          }}
          onReset={() => {
            navigate({
              search: { page: 1, limit: currentLimit } as any,
            })
          }}
          countComponent={
            <FilterTotalCount total={countNumManager.getTotal() ?? 0} loading={isLoading} />
          }
          className="bg-card rounded-lg border shadow-sm relative"
        >
          {(layoutParams, form) => (
            <div className="flex-1 flex flex-wrap items-end gap-3">
              <FilterUserMode
                value={userMode}
                onChange={setUserMode}
                layoutParams={layoutParams}
              />

              <FilterInput
                name="res_name"
                placeholder="输入资源名称"
                label="资源名称"
                disabled={isLoading}
                layoutParams={layoutParams}
                className="w-[8.5rem]"
              />

              <FilterInput
                name="res_type"
                placeholder="输入资源类型"
                label="资源类型"
                disabled={isLoading}
                layoutParams={layoutParams}
                className="w-[8.5rem]"
              />

              <FilterInput
                name="res_data"
                placeholder="输入资源数据"
                label="资源数据"
                disabled={isLoading}
                layoutParams={layoutParams}
                className="w-[8.5rem]"
              />

              <FilterActions
                form={form}
                loading={isLoading}
                layoutParams={layoutParams}
                onRefreshSearch={clearCacheAndReload}
              />
            </div>
          )}
        </FilterContainer>
      </div>

      {/* 表格和分页 */}
      <div className="flex-1 flex flex-col min-h-0">
        <div className="flex-1 overflow-hidden">
          <DataTable
            data={resources}
            columns={columns}
            loading={isLoading}
            error={isError ? <CenteredError error={error} variant="content" onReset={refreshData} /> : null}
            scrollSnapDelay={300}
            className="[&_tr]:h-11 [&_td]:py-1 [&_th]:py-1 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b h-full"
            tableContainerClassName="h-full"
          />
        </div>

        <div className="flex-shrink-0 pt-4 pb-4">
          <PagePagination
            currentPage={currentPage}
            pageSize={currentLimit}
            total={countNumManager.getTotal() ?? 0}
            loading={isLoading}
            onChange={(page: number) => {
              navigate({
                search: { ...filterParam, page } as any,
              })
            }}
            onPageSizeChange={(limit: number) => {
              navigate({
                search: { ...filterParam, page: 1, limit } as any,
              })
            }}
          />
        </div>
      </div>

      {/* 抽屉组件 */}
      <ResDrawer
        resource={editingRes}
        open={drawerOpen}
        onOpenChange={setDrawerOpen}
        appId={appId}
      />

      <ResTypeOpsDrawer
        resType={selectedResType}
        open={opsDrawerOpen}
        onOpenChange={setOpsDrawerOpen}
        appId={appId}
        userMode={userMode}
        onSuccess={refreshData}
      />

      {/* 操作管理抽屉 */}
      <ResOpListDrawer
        open={opListDrawerOpen}
        onOpenChange={setOpListDrawerOpen}
        appId={appId}
      />
    </div>
  )
}

// 导出 schema 供路由使用
export { ResListFilterParamSchema } from './res-schema'
