import {
  rolePermData,
  rolePermAdd,
  rolePermDelete,
  type RoleItemType,
  type PermissionItemType,
} from '@shared/apis/admin/rbac-role'
import {
  resourceList,
  resourceTypeOpData,
  type ResourceItemType,
  type ResourceOpItemType,
} from '@shared/apis/admin/rbac-res'
import { ConfirmDialog } from '@shared/components/custom/dialog/confirm-dialog'
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
} from '@apps/main/components/local/drawer'
import {
  DEFAULT_PAGE_SIZE,
  PagePagination,
  useCountNumManager,
} from '@apps/main/lib/pagination-utils'
import { Badge } from '@shared/components/ui/badge'
import { Button } from '@shared/components/ui/button'
import { Card, CardContent } from '@shared/components/ui/card'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@shared/components/ui/select'
import { useToast } from '@shared/contexts/toast-context'
import { useIsMobile } from '@shared/hooks/use-mobile'
import { formatServerError, getQueryResponseData } from '@shared/lib/utils'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Loader2, Plus, Shield, Trash2 } from 'lucide-react'
import React, { useState } from 'react'

// 移动端/PC端自适应列表视图组件
interface RolePermsListViewProps {
  perms: PermissionItemType[]
  onDelete: (perm: PermissionItemType) => Promise<void>
}

function RolePermsListView({ perms, onDelete }: RolePermsListViewProps) {
  const isMobile = useIsMobile()

  return (
    <div className={isMobile ? "space-y-3" : "grid grid-cols-2 gap-3"}>
      {perms.map((perm) => (
        <Card key={`${perm.res_id}-${perm.op_id}`} className="p-0">
          <CardContent className={isMobile ? "p-4 space-y-3" : "p-3 space-y-2"}>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <span className="text-xs text-muted-foreground">ID:</span>
                <span className="text-sm font-mono">{perm.res_id}</span>
              </div>
              <ConfirmDialog
                title="删除资源"
                description={`确定要删除资源「${perm.res_name || perm.res_type} - ${perm.op_name || perm.op_key}」吗？`}
                onConfirm={async () => {
                  await onDelete(perm)
                }}
              >
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-7 px-2"
                >
                  <Trash2 className="h-4 w-4" />
                </Button>
              </ConfirmDialog>
            </div>
            <div className="flex items-start gap-3">
              <div className="text-xs font-medium text-muted-foreground whitespace-nowrap min-w-[60px] shrink-0 pt-0.5">
                资源类型
              </div>
              <div className="text-sm font-medium">{perm.res_name || perm.res_type}</div>
            </div>
            <div className="flex items-start gap-3">
              <div className="text-xs font-medium text-muted-foreground whitespace-nowrap min-w-[60px] shrink-0 pt-0.5">
                资源数据
              </div>
              <div className="text-sm flex-1 min-w-0">
                <code className="text-xs text-muted-foreground bg-muted px-1 py-0.5 rounded break-all">
                  {perm.res_data}
                </code>
              </div>
            </div>
            <div className="flex items-start gap-3">
              <div className="text-xs font-medium text-muted-foreground whitespace-nowrap min-w-[60px] shrink-0 pt-0.5">
                操作
              </div>
              <div className="text-sm">
                <Badge variant="secondary">{perm.op_name || perm.op_key}</Badge>
              </div>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  )
}

interface RolePermsDrawerProps {
  /** 角色数据 */
  role: RoleItemType
  /** 是否打开 */
  open: boolean
  /** 打开状态变化回调 */
  onOpenChange: (open: boolean) => void
}

export function RolePermsDrawer({
  role,
  open,
  onOpenChange,
}: RolePermsDrawerProps) {
  const toast = useToast()
  const queryClient = useQueryClient()
  const [selectedResId, setSelectedResId] = React.useState<number | null>(null)
  const [selectedOpId, setSelectedOpId] = React.useState<number | null>(null)

  // 分页状态
  const [pagination, setPagination] = useState({
    page: 1,
    limit: DEFAULT_PAGE_SIZE,
  })

  // count_num 优化管理器
  const countNumManager = useCountNumManager()

  // 获取角色权限列表
  const { data: permsData, isSuccess, isLoading } = useQuery({
    queryKey: ['admin-rbac-role-perms', role.id, pagination.page, pagination.limit],
    queryFn: async ({ signal }) => {
      const result = await rolePermData(
        {
          role_id: role.id,
          page: {
            page: pagination.page,
            limit: pagination.limit,
          },
          count_num: countNumManager.getCountNum(),
        },
        { signal }
      )
      return result
    },
    placeholderData: (previousData) => previousData,
    enabled: open,
  })

  // 处理分页查询结果
  if (isSuccess) {
    countNumManager.handlePageQueryResult(permsData)
  }

  // 获取资源列表
  const { data: resListData } = useQuery({
    queryKey: ['admin-rbac-res-list-for-perm'],
    queryFn: async ({ signal }) => {
      const result = await resourceList(
        {
          page: { page: 1, limit: 100 },
          count_num: false,
        },
        { signal }
      )
      return result
    },
    enabled: open,
  })

  // 获取选中资源的操作列表
  const selectedRes = getQueryResponseData<ResourceItemType[]>(resListData, []).find(
    (r) => r.id === selectedResId
  )

  const { data: opsData, isLoading: opsLoading } = useQuery({
    queryKey: ['admin-rbac-res-ops', selectedRes?.res_type],
    queryFn: async ({ signal }) => {
      if (!selectedRes) return { result: { data: [] } }
      const result = await resourceTypeOpData(
        {
          res_type: selectedRes.res_type,
          page: { page: 1, limit: 100 },
          count_num: false,
        },
        { signal }
      )
      return result
    },
    enabled: open && !!selectedRes,
  })

  const perms = getQueryResponseData<PermissionItemType[]>(permsData, [])
  const resources = getQueryResponseData<ResourceItemType[]>(resListData, [])
  const operations = getQueryResponseData<ResourceOpItemType[]>(opsData, [])

  // 添加资源
  const addPermMutation = useMutation({
    mutationFn: () => {
      if (!selectedResId || !selectedOpId) {
        throw new Error('请选择资源和操作')
      }
      return rolePermAdd({
        role_id: role.id,
        perm_data: [
          {
            res_id: selectedResId,
            op_id: selectedOpId,
          },
        ],
      })
    },
    onSuccess: () => {
      toast.success('资源添加成功')
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-role-perms', role.id] })
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-role-list'], refetchType: 'all' })
      setSelectedResId(null)
      setSelectedOpId(null)
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  // 删除资源
  const deletePermMutation = useMutation({
    mutationFn: (perm: PermissionItemType) =>
      rolePermDelete({
        role_id: role.id,
        perm_data: [
          {
            res_id: perm.res_id,
            op_id: perm.op_id,
          },
        ],
      }),
    onSuccess: () => {
      toast.success('资源删除成功')
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-role-perms', role.id] })
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-role-list'], refetchType: 'all' })
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  // 资源变化时清空操作选择
  React.useEffect(() => {
    setSelectedOpId(null)
  }, [selectedResId])

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent className="w-[95%] md:w-[720px]">
        <DrawerHeader>
          <DrawerTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            角色资源管理
          </DrawerTitle>
          <DrawerDescription>
            管理角色「{role.role_name}」的资源配置
          </DrawerDescription>
        </DrawerHeader>

        <div className="mt-6 space-y-6">
          {/* 添加资源区域 */}
          <div className="space-y-3 p-4 border rounded-lg bg-muted/50">
            <h4 className="font-medium">添加资源</h4>
            <div className="flex flex-wrap gap-3 items-end">
              <div className="flex-1 min-w-[150px]">
                <label className="text-sm text-muted-foreground mb-1.5 block">选择资源</label>
                <Select
                  value={selectedResId?.toString() || ''}
                  onValueChange={(value) => setSelectedResId(Number(value))}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="选择资源" />
                  </SelectTrigger>
                  <SelectContent className="max-h-[300px]">
                    {resources.map((res) => (
                      <SelectItem key={res.id} value={res.id.toString()}>
                        {res.res_name} ({res.res_type})
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div className="flex-1 min-w-[150px]">
                <label className="text-sm text-muted-foreground mb-1.5 block">选择操作</label>
                <Select
                  value={selectedOpId?.toString() || ''}
                  onValueChange={(value) => setSelectedOpId(Number(value))}
                  disabled={!selectedResId || opsLoading}
                >
                  <SelectTrigger>
                    <SelectValue placeholder={opsLoading ? '加载中...' : '选择操作'} />
                  </SelectTrigger>
                  <SelectContent className="max-h-[300px]">
                    {operations.map((op) => (
                      <SelectItem key={op.op_data.id} value={op.op_data.id.toString()}>
                        {op.op_data.op_name} ({op.op_data.op_key})
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <Button
                onClick={() => addPermMutation.mutate()}
                disabled={!selectedResId || !selectedOpId || addPermMutation.isPending}
              >
                {addPermMutation.isPending && (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                )}
                <Plus className="mr-2 h-4 w-4" />
                添加资源
              </Button>
            </div>
          </div>

          {/* 资源列表 */}
          <div className="space-y-3">
            <h4 className="font-medium">已配置资源 ({countNumManager.getTotal() ?? perms.length})</h4>
            {isLoading ? (
              <div className="flex items-center justify-center py-8">
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
              </div>
            ) : perms.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                暂无资源配置
              </div>
            ) : (
              <RolePermsListView
                perms={perms}
                onDelete={async (perm) => {
                  await deletePermMutation.mutateAsync(perm)
                }}
              />
            )}

            {/* 分页 */}
            <PagePagination
              currentPage={pagination.page}
              pageSize={pagination.limit}
              total={countNumManager.getTotal() ?? 0}
              loading={isLoading}
              onChange={(page: number) => {
                setPagination((prev) => ({ ...prev, page }))
              }}
              onPageSizeChange={(limit: number) => {
                setPagination({ page: 1, limit })
              }}
            />
          </div>
        </div>
      </DrawerContent>
    </Drawer>
  )
}
