import {
  appRbacResList,
  appRbacResTypeOpData,
  appRbacRolePermAdd,
  appRbacRolePermData,
  appRbacRolePermDelete,
  type AppRbacResDataItemType,
  type AppRbacResTypeOpDataItemType,
  type AppRbacRoleDataItemType,
  type AppRbacRolePermDataResItemType,
} from '@shared/apis/user/rbac'
import { ConfirmDialog } from '@shared/components/custom/dialog/confirm-dialog'
import { Badge } from '@shared/components/ui/badge'
import { Button } from '@shared/components/ui/button'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@shared/components/ui/select'
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
} from '@apps/main/components/local/drawer'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@shared/components/ui/table'
import { useToast } from '@shared/contexts/toast-context'
import type { RbacUserModeContext } from '@apps/main/features/user/components/ui/filter-user-mode'
import { cn, formatServerError, getQueryResponseData } from '@shared/lib/utils'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Loader2, Plus, Shield, Trash2 } from 'lucide-react'
import React from 'react'

interface RolePermsDrawerProps {
  /** 角色数据 */
  role: AppRbacRoleDataItemType
  /** 是否打开 */
  open: boolean
  /** 打开状态变化回调 */
  onOpenChange: (open: boolean) => void
  /** 应用ID */
  appId: number
  /** 用户模式 */
  userMode: RbacUserModeContext
}

export function RolePermsDrawer({
  role,
  open,
  onOpenChange,
  appId,
  userMode,
}: RolePermsDrawerProps) {
  const toast = useToast()
  const queryClient = useQueryClient()
  const [selectedResId, setSelectedResId] = React.useState<number | null>(null)
  const [selectedOpId, setSelectedOpId] = React.useState<number | null>(null)

  // 获取角色权限列表
  const { data: permsData, isLoading } = useQuery({
    queryKey: ['rbac-role-perms', role.id],
    queryFn: async ({ signal }) => {
      const result = await appRbacRolePermData(
        {
          role_id: role.id,
          page: { page: 1, limit: 100 },
          count_num: false,
        },
        { signal }
      )
      return result
    },
    enabled: open,
  })

  // 获取资源列表
  const { data: resListData, isLoading: resLoading } = useQuery({
    queryKey: ['rbac-res-list', appId, userMode.use_app_user, userMode.user_param],
    queryFn: async ({ signal }) => {
      const result = await appRbacResList(
        {
          app_id: appId,
          use_app_user: userMode.use_app_user,
          user_param: userMode.user_param,
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
  const selectedRes = getQueryResponseData<AppRbacResDataItemType[]>(resListData, []).find(
    (r) => r.id === selectedResId
  )

  const { data: opsData, isLoading: opsLoading } = useQuery({
    queryKey: ['rbac-res-ops', appId, selectedRes?.res_type],
    queryFn: async ({ signal }) => {
      if (!selectedRes) return { result: { data: [] } }
      const result = await appRbacResTypeOpData(
        {
          app_id: appId,
          use_app_user: userMode.use_app_user,
          user_param: userMode.user_param,
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

  const perms = getQueryResponseData<AppRbacRolePermDataResItemType[]>(permsData, [])
  const resources = getQueryResponseData<AppRbacResDataItemType[]>(resListData, [])
  const operations = getQueryResponseData<AppRbacResTypeOpDataItemType[]>(opsData, [])

  // 添加权限
  const addPermMutation = useMutation({
    mutationFn: () => {
      if (!selectedResId || !selectedOpId) {
        throw new Error('请选择资源和操作')
      }
      return appRbacRolePermAdd({
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
      toast.success('权限添加成功')
      queryClient.invalidateQueries({ queryKey: ['rbac-role-perms', role.id] })
      setSelectedResId(null)
      setSelectedOpId(null)
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  // 删除权限
  const deletePermMutation = useMutation({
    mutationFn: (perm: AppRbacRolePermDataResItemType) =>
      appRbacRolePermDelete({
        role_id: role.id,
        perm_data: [
          {
            res_id: perm.res_id,
            op_id: perm.op_id,
          },
        ],
      }),
    onSuccess: () => {
      toast.success('权限移除成功')
      queryClient.invalidateQueries({ queryKey: ['rbac-role-perms', role.id] })
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  const handleAddPerm = () => {
    addPermMutation.mutate()
  }

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent>
        <DrawerHeader>
          <DrawerTitle className={cn("flex items-center gap-2")}>
            <Shield className="h-5 w-5" />
            角色权限管理
          </DrawerTitle>
          <DrawerDescription>
            管理角色「{role.role_name}」的资源操作权限
          </DrawerDescription>
        </DrawerHeader>

        <div className="mt-6 space-y-4">
          {/* 添加权限区域 */}
          <div className="flex gap-2 items-end">
            <div className="flex-1 space-y-1">
              <label className="text-sm font-medium">选择资源</label>
              <Select
                value={selectedResId ? String(selectedResId) : ''}
                onValueChange={(value) => {
                  setSelectedResId(Number(value))
                  setSelectedOpId(null)
                }}
                disabled={resLoading}
              >
                <SelectTrigger>
                  <SelectValue placeholder="请选择资源" />
                </SelectTrigger>
                <SelectContent className="max-h-[300px]">
                  {resources.map((res) => (
                    <SelectItem key={res.id} value={String(res.id)}>
                      {res.res_name} ({res.res_type}:{res.res_data})
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="flex-1 space-y-1">
              <label className="text-sm font-medium">选择操作</label>
              <Select
                value={selectedOpId ? String(selectedOpId) : ''}
                onValueChange={(value) => setSelectedOpId(Number(value))}
                disabled={!selectedResId || opsLoading}
              >
                <SelectTrigger>
                  <SelectValue placeholder="请选择操作" />
                </SelectTrigger>
                <SelectContent className="max-h-[300px]">
                  {operations.map((op) => (
                    <SelectItem key={op.op_data.id} value={String(op.op_data.id)}>
                      {op.op_data.op_name} ({op.op_data.op_key})
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <Button
              onClick={handleAddPerm}
              disabled={!selectedResId || !selectedOpId || addPermMutation.isPending}
            >
              {addPermMutation.isPending ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <>
                  <Plus className=" h-4 w-4" />
                <span className="ml-2">添加</span>
                </>
              )}
            </Button>
          </div>

          {/* 权限列表 */}
          <div className="border rounded-lg">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>资源</TableHead>
                  <TableHead>类型</TableHead>
                  <TableHead>操作</TableHead>
                  <TableHead className="w-[80px] text-center">移除</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {isLoading ? (
                  <TableRow>
                    <TableCell colSpan={4} className="text-center py-8">
                      <Loader2 className="h-6 w-6 animate-spin mx-auto" />
                    </TableCell>
                  </TableRow>
                ) : perms.length === 0 ? (
                  <TableRow>
                    <TableCell
                      colSpan={4}
                      className="text-center py-8 text-muted-foreground"
                    >
                      暂无权限配置
                    </TableCell>
                  </TableRow>
                ) : (
                  perms.map((perm) => (
                    <TableRow key={perm.id}>
                      <TableCell>
                        <div className="flex flex-col">
                          <span className="font-medium">{perm.res_name || '-'}</span>
                          <span className="text-xs text-muted-foreground">
                            {perm.res_data}
                          </span>
                        </div>
                      </TableCell>
                      <TableCell>
                        <Badge variant="outline">{perm.res_type}</Badge>
                      </TableCell>
                      <TableCell>
                        <div className="flex flex-col">
                          <span>{perm.op_name || '-'}</span>
                          <span className="text-xs text-muted-foreground font-mono">
                            {perm.op_key}
                          </span>
                        </div>
                      </TableCell>
                      <TableCell className="text-center">
                        <ConfirmDialog
                          title="移除权限"
                          description={`确定要移除权限「${perm.res_name || perm.res_type} - ${perm.op_name || perm.op_key}」吗？`}
                          onConfirm={async () => {
                            await deletePermMutation.mutateAsync(perm)
                          }}
                        >
                          <Button
                            variant="ghost"
                            size="sm"
                            className="h-8 w-8 p-0 "
                          >
                            <Trash2 className="h-4 w-4" />
                          </Button>
                        </ConfirmDialog>
                      </TableCell>
                    </TableRow>
                  ))
                )}
              </TableBody>
            </Table>
          </div>
        </div>
      </DrawerContent>
    </Drawer>
  )
}
