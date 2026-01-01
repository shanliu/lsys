import {
  appRbacRoleDelete,
  appRbacRoleList,
  type AppRbacRoleDataItemType,
} from '@shared/apis/user/rbac'
import { ConfirmDialog } from '@shared/components/custom/dialog/confirm-dialog'
import { FilterContainer } from '@apps/main/components/filter-container/container'
import { FilterActions } from '@apps/main/components/filter-container/filter-actions'
import { FilterInput } from '@apps/main/components/filter-container/filter-input'
import { FilterSelect } from '@apps/main/components/filter-container/filter-select'
import { FilterTotalCount } from '@apps/main/components/filter-container/filter-total-count'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { PageSkeletonTable } from '@shared/components/custom/page-placeholder/skeleton-table'
import {
  DEFAULT_PAGE_SIZE,
  PagePagination,
  useCountNumManager,
} from '@apps/main/lib/pagination-utils'
import { DataTable, DataTableAction, DataTableActionItem } from '@shared/components/custom/table'
import { Badge } from '@shared/components/ui/badge'
import { Button } from '@shared/components/ui/button'
import { useToast } from '@shared/contexts/toast-context'
import { AppDetailNavContainer } from '@apps/main/features/user/components/ui/app-detail-nav'
import { FilterUserMode, type RbacUserModeContext } from '@apps/main/features/user/components/ui/filter-user-mode'
import { useDictData } from '@apps/main/hooks/use-dict-data'
import { useIsMobile } from '@shared/hooks/use-mobile'
import {
  cn,
  formatServerError,
  getQueryResponseData,
} from '@shared/lib/utils'
import { Route } from '@apps/main/routes/_main/user/app/$appId/features-rbac/role'

import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { type ColumnDef } from '@tanstack/react-table'
import { Edit, Plus, Shield, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { featureRbacModuleConfig } from '../nav-info'
import { RoleDrawer } from './role-drawer'
import { RolePermsDrawer } from './role-perms-drawer'
import {
  RoleListFilterFormSchema
} from './role-schema'
import { RoleUsersDrawer } from './role-users-drawer'

export default function AppDetailFeatureRbacRolePage() {
  const { appId } = Route.useParams()
  const [drawerOpen, setDrawerOpen] = useState(false)

  const handleAdd = () => {
    setDrawerOpen(true)
  }

  const actions = (
    <Button variant="outline" size="sm" onClick={handleAdd}>
      <Plus className="mr-2 h-4 w-4" />
      新增角色
    </Button>
  )

  return (
    <AppDetailNavContainer {...featureRbacModuleConfig} actions={actions}>
      <RoleListContent appId={appId} drawerOpen={drawerOpen} setDrawerOpen={setDrawerOpen} />
    </AppDetailNavContainer>
  )
}

interface RoleListContentProps {
  appId: number
  drawerOpen: boolean
  setDrawerOpen: (open: boolean) => void
}

function RoleListContent({ appId, drawerOpen, setDrawerOpen }: RoleListContentProps) {
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const isMobile = useIsMobile()
  const toast = useToast()

  // 内部获取RBAC映射信息（用户范围、资源范围选项）
  const {
    dictData,
    isLoading: mappingLoading,
    isError: mappingError,
    errors: mappingErrors,
    refetch: refetchMapping,
  } = useDictData(['app_rbac'] as const)

  // Drawer 状态管理
  const [editingRole, setEditingRole] = useState<AppRbacRoleDataItemType | undefined>()
  const [usersDrawerOpen, setUsersDrawerOpen] = useState(false)
  const [permsDrawerOpen, setPermsDrawerOpen] = useState(false)
  const [selectedRole, setSelectedRole] = useState<AppRbacRoleDataItemType | undefined>()

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
    role_name: filterParam.role_name ?? null,
    role_key: filterParam.role_key ?? null,
    user_range: filterParam.user_range ?? null,
    res_range: filterParam.res_range ?? null,
  }

  // count_num 优化管理器
  const countNumManager = useCountNumManager(filters)

  // 获取角色列表
  const { data: rolesData, isSuccess, isLoading, isError, error } = useQuery({
    queryKey: [
      'rbac-role-list',
      appId,
      userMode.use_app_user,
      userMode.user_param,
      filterParam.role_name,
      filterParam.role_key,
      filterParam.user_range,
      filterParam.res_range,
      currentPage,
      currentLimit,
    ],
    queryFn: async ({ signal }) => {
      const result = await appRbacRoleList(
        {
          app_id: Number(appId),
          use_app_user: userMode.use_app_user,
          user_param: userMode.user_param,
          role_name: filterParam.role_name,
          role_key: filterParam.role_key,
          user_range: filterParam.user_range,
          res_range: filterParam.res_range,
          res_count: true,
          res_op_count: false,
          user_count: true,
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
  isSuccess && countNumManager.handlePageQueryResult(rolesData)

  // 从查询结果中提取数据
  const roles = getQueryResponseData<AppRbacRoleDataItemType[]>(rolesData, [])

  // 删除角色
  const deleteMutation = useMutation({
    mutationFn: (roleId: number) => appRbacRoleDelete({ role_id: roleId }),
    onSuccess: () => {
      toast.success('角色删除成功')
      queryClient.invalidateQueries({ queryKey: ['rbac-role-list'] })
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  // 刷新数据
  const refreshData = () => {
    queryClient.refetchQueries({ queryKey: ['rbac-role-list'] })
  }

  // 清除缓存并重新加载
  const clearCacheAndReload = () => {
    countNumManager.reset()
    queryClient.invalidateQueries({ queryKey: ['rbac-role-list'] })
  }

  // 打开新增/编辑抽屉
  const _handleAdd = () => {
    setEditingRole(undefined)
    setDrawerOpen(true)
  }

  // 打开编辑抽屉
  const handleEdit = (role: AppRbacRoleDataItemType) => {
    setEditingRole(role)
    setDrawerOpen(true)
  }

  // 打开用户管理抽屉
  const handleManageUsers = (role: AppRbacRoleDataItemType) => {
    setSelectedRole(role)
    setUsersDrawerOpen(true)
  }

  // 打开权限管理抽屉
  const handleManagePerms = (role: AppRbacRoleDataItemType) => {
    setSelectedRole(role)
    setPermsDrawerOpen(true)
  }

  // 定义表格列配置
  const columns: ColumnDef<AppRbacRoleDataItemType>[] = [
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
      accessorKey: 'role_name',
      header: '角色名称',
      cell: ({ getValue }) => (
        <div className="font-medium">{getValue<string>()}</div>
      ),
    },
    {
      accessorKey: 'role_key',
      header: '角色标识',
      cell: ({ getValue }) => {
        const key = getValue<string>()
        return key ? (
          <code className="text-sm bg-muted px-1.5 py-0.5 rounded">{key}</code>
        ) : (
          <span className="text-muted-foreground">-</span>
        )
      },
    },
    {
      accessorKey: 'res_range',
      header: '资源范围',
      size: 180,
      cell: ({ row, getValue }) => {
        const value = getValue<number>()
        const label = dictData.role_res_range?.getLabel(String(value)) || String(value)
        const role = row.original
        const resCount = role.res_count ?? 0

        return (
          <div className="flex items-center gap-2">
            <Badge variant="secondary">{label}</Badge>
            {String(value) === '1' && (
              <button
                type="button"
                className="text-xs text-primary hover:underline cursor-pointer"
                onClick={() => handleManagePerms(role)}
              >
                资源:({resCount})
              </button>
            )}
          </div>
        )
      },
    },
    {
      accessorKey: 'user_range',
      header: '用户范围',
      size: 180,
      cell: ({ row, getValue }) => {
        const value = getValue<number>()
        const label = dictData.role_user_range?.getLabel(String(value)) || String(value)
        const role = row.original
        const userCount = role.user_count ?? 0

        return (
          <div className="flex items-center gap-2">
            <Badge variant="secondary">{label}</Badge>
            {String(value) === '1' && (
              <button
                type="button"
                className="text-xs text-primary hover:underline cursor-pointer"
                onClick={() => handleManageUsers(role)}
              >
                用户:({userCount})
              </button>
            )}
          </div>
        )
      },
    },

    {
      id: 'actions',
      header: () => <div className="text-center">操作</div>,
      size: 80,
      cell: ({ row }) => {
        const role = row.original

        return (
          <DataTableAction className={cn(isMobile ? 'justify-end' : 'justify-center')}>
            {String(role.res_range) === '1' && (
              <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
                <Button
                  variant="ghost"
                  size="sm"
                  className={cn('h-7 px-2')}
                  onClick={() => handleManagePerms(role)}
                >
                  <Shield className="h-4 w-4" />
                  <span className="ml-2">权限</span>
                </Button>
              </DataTableActionItem>
            )}
            <DataTableActionItem mobileDisplay="collapsed" desktopDisplay="collapsed">
              <Button
                variant="ghost"
                size="sm"
                className={cn('h-7 px-2')}
                onClick={() => handleEdit(role)}
              >
                <Edit className="h-4 w-4" />
                <span className="ml-2">编辑</span>
              </Button>
            </DataTableActionItem>
            <DataTableActionItem mobileDisplay="collapsed" desktopDisplay="collapsed">
              <ConfirmDialog
                title="删除角色"
                description={`确定要删除角色「${role.role_name}」吗？删除后无法恢复。`}
                onConfirm={async () => {
                  await deleteMutation.mutateAsync(role.id)
                }}
              >
                <Button
                  variant="ghost"
                  size="sm"
                  className={cn('h-7 px-2 ')}
                >
                  <Trash2 className="h-4 w-4" />
                  <span className="ml-2">删除</span>
                </Button>
              </ConfirmDialog>
            </DataTableActionItem>
          </DataTableAction>
        )
      },
    },
  ]

  // 用户范围选项
  const userRangeOptions = dictData.role_user_range?.map((item) => ({
    value: item.key,
    label: item.val,
  })) || []

  // 资源范围选项
  const resRangeOptions = dictData.role_res_range?.map((item) => ({
    value: item.key,
    label: item.val,
  })) || []

  // 如果映射数据加载中，显示骨架屏
  if (mappingLoading) {
    return <PageSkeletonTable variant="content"/>
  }

  // 如果映射数据加载失败，显示错误页面
  if (mappingError) {
    return (
      <CenteredError
        variant="content"
        error={mappingErrors}
        onReset={refetchMapping}
      />
    )
  }

  return (
    <div className="flex flex-col min-h-0 space-y-3">
      {/* 过滤器 */}
      <div className="flex-shrink-0 mb-1 sm:mb-4">
        <FilterContainer
          defaultValues={{
            role_name: filterParam.role_name,
            role_key: filterParam.role_key,
            user_range: filterParam.user_range?.toString(),
            res_range: filterParam.res_range?.toString(),
            }}
            resolver={zodResolver(RoleListFilterFormSchema) as any}
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
                  name="role_name"
                  placeholder="输入角色名称"
                  label="角色名称"
                  disabled={isLoading}
                  layoutParams={layoutParams}
                />

                <FilterInput
                  name="role_key"
                  placeholder="输入角色标识"
                  label="角色标识"
                  disabled={isLoading}
                  layoutParams={layoutParams}
                />

                <FilterSelect
                  name="user_range"
                  placeholder="选择用户范围"
                  label="用户范围"
                  disabled={isLoading}
                  options={userRangeOptions}
                  allLabel="全部"
                  layoutParams={layoutParams}
                />

                <FilterSelect
                  name="res_range"
                  placeholder="选择资源范围"
                  label="资源范围"
                  disabled={isLoading}
                  options={resRangeOptions}
                  allLabel="全部"
                  layoutParams={layoutParams}
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
              data={roles}
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
        <RoleDrawer
          role={editingRole}
          open={drawerOpen}
          onOpenChange={setDrawerOpen}
          appId={appId}
        />

        {selectedRole && (
          <>
            <RoleUsersDrawer
              role={selectedRole}
              open={usersDrawerOpen}
              onOpenChange={setUsersDrawerOpen}
              appId={appId}
            />

            <RolePermsDrawer
              role={selectedRole}
              open={permsDrawerOpen}
              onOpenChange={setPermsDrawerOpen}
              appId={appId}
              userMode={userMode}
            />
          </>
        )}
      </div>
  )
}

// 导出 schema 供路由使用
export { RoleListFilterParamSchema } from './role-schema'

