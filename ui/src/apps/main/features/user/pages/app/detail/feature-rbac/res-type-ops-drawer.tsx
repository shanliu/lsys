import {
  appRbacOpList,
  appRbacResTypeOpAdd,
  appRbacResTypeOpData,
  appRbacResTypeOpDel,
  type AppRbacOpDataItemType,
  type AppRbacResTypeOpDataItemType,
} from '@shared/apis/user/rbac'
import { ConfirmDialog } from '@shared/components/custom/dialog/confirm-dialog'
import { Button } from '@shared/components/ui/button'
import { Checkbox } from '@shared/components/ui/checkbox'
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
import { Loader2, Plus, Settings, Trash2 } from 'lucide-react'
import React from 'react'

interface ResTypeOpsDrawerProps {
  /** 资源类型 */
  resType: string
  /** 是否打开 */
  open: boolean
  /** 打开状态变化回调 */
  onOpenChange: (open: boolean) => void
  /** 应用ID */
  appId: number
  /** 用户模式 */
  userMode: RbacUserModeContext
  /** 操作成功回调 */
  onSuccess?: () => void
}

export function ResTypeOpsDrawer({
  resType,
  open,
  onOpenChange,
  appId,
  userMode,
  onSuccess,
}: ResTypeOpsDrawerProps) {
  const toast = useToast()
  const queryClient = useQueryClient()
  const [selectedOpIds, setSelectedOpIds] = React.useState<number[]>([])

  // 获取资源类型已关联的操作列表
  const { data: resOpsData, isLoading: resOpsLoading } = useQuery({
    queryKey: ['rbac-res-type-ops', appId, resType, userMode.use_app_user, userMode.user_param],
    queryFn: async ({ signal }) => {
      const result = await appRbacResTypeOpData(
        {
          app_id: appId,
          use_app_user: userMode.use_app_user,
          user_param: userMode.user_param,
          res_type: resType,
          page: { page: 1, limit: 100 },
          count_num: false,
        },
        { signal }
      )
      return result
    },
    enabled: open && !!resType,
  })

  // 获取所有可用的操作列表
  const { data: allOpsData, isLoading: allOpsLoading } = useQuery({
    queryKey: ['rbac-op-list', appId, userMode.use_app_user, userMode.user_param],
    queryFn: async ({ signal }) => {
      const result = await appRbacOpList(
        {
          app_id: appId,
          use_app_user: userMode.use_app_user,
          user_param: userMode.user_param,
          page: { page: 1, limit: 100 },
          res_type_count:true,
          check_role_use:true,
          count_num: false,
        },
        { signal }
      )
      return result
    },
    enabled: open,
  })

  const resOps = getQueryResponseData<AppRbacResTypeOpDataItemType[]>(resOpsData, [])
  const allOps = getQueryResponseData<AppRbacOpDataItemType[]>(allOpsData, [])

  // 过滤出未关联的操作（从 op_data 中提取 id，统一转为 number 类型比较）
  const resOpIds = resOps.map((op) => Number(op.op_data.id))
  const availableOps = allOps.filter((op) => !resOpIds.includes(Number(op.id)))

  // 添加操作关联
  const addOpMutation = useMutation({
    mutationFn: () =>
      appRbacResTypeOpAdd({
        app_id: appId,
        use_app_user: userMode.use_app_user,
        user_param: userMode.user_param,
        res_type: resType,
        op_ids: selectedOpIds,
      }),
    onSuccess: () => {
      toast.success('操作关联成功')
      queryClient.invalidateQueries({
        queryKey: ['rbac-res-type-ops', appId, resType],
      })
      setSelectedOpIds([])
      onSuccess?.()
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  // 删除操作关联
  const deleteOpMutation = useMutation({
    mutationFn: (opId: number) =>
      appRbacResTypeOpDel({
        app_id: appId,
        use_app_user: userMode.use_app_user,
        user_param: userMode.user_param,
        res_type: resType,
        op_ids: [opId],
      }),
    onSuccess: () => {
      toast.success('操作移除成功')
      queryClient.invalidateQueries({
        queryKey: ['rbac-res-type-ops', appId, resType],
      })
      onSuccess?.()
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  const handleToggleOp = (opId: number) => {
    setSelectedOpIds((prev) =>
      prev.includes(opId) ? prev.filter((id) => id !== opId) : [...prev, opId]
    )
  }

  const handleAddOps = () => {
    if (selectedOpIds.length === 0) {
      toast.error('请选择要关联的操作')
      return
    }
    addOpMutation.mutate()
  }

  const isLoading = resOpsLoading || allOpsLoading

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent>
        <DrawerHeader>
          <DrawerTitle className={cn("flex items-center gap-2")}>
            <Settings className="h-5 w-5" />
            资源类型操作管理
          </DrawerTitle>
          <DrawerDescription>
            管理资源类型「{resType}」可用的操作
          </DrawerDescription>
        </DrawerHeader>

        <div className="mt-6 space-y-6">
          {/* 已关联的操作 */}
          <div className="space-y-2">
            <h4 className="text-sm font-medium">已关联操作</h4>
            <div className="border rounded-lg">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>操作名称</TableHead>
                    <TableHead>操作标识</TableHead>
                    <TableHead className="w-[80px] text-center">移除</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {isLoading ? (
                    <TableRow>
                      <TableCell colSpan={3} className="text-center py-8">
                        <Loader2 className="h-6 w-6 animate-spin mx-auto" />
                      </TableCell>
                    </TableRow>
                  ) : resOps.length === 0 ? (
                    <TableRow>
                      <TableCell
                        colSpan={3}
                        className="text-center py-8 text-muted-foreground"
                      >
                        暂无关联操作
                      </TableCell>
                    </TableRow>
                  ) : (
                    resOps.map((op) => (
                      <TableRow key={op.op_data.id}>
                        <TableCell>{op.op_data.op_name}</TableCell>
                        <TableCell>
                          <code className="text-sm bg-muted px-1.5 py-0.5 rounded">
                            {op.op_data.op_key}
                          </code>
                        </TableCell>
                        <TableCell className="text-center">
                          <ConfirmDialog
                            title="移除操作"
                            description={`确定要将操作「${op.op_data.op_name}」从该资源类型中移除吗？`}
                            onConfirm={async () => {
                              await deleteOpMutation.mutateAsync(op.op_data.id)
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

          {/* 可添加的操作 */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <h4 className="text-sm font-medium">添加操作</h4>
              <Button
                size="sm"
                onClick={handleAddOps}
                disabled={selectedOpIds.length === 0 || addOpMutation.isPending}
              >
                {addOpMutation.isPending ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <>
                    <Plus className=" h-4 w-4" />
                  <span className='ml-2'>添加选中 ({selectedOpIds.length})</span>
                  </>
                )}
              </Button>
            </div>
            <div className="border rounded-lg">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-[50px]">选择</TableHead>
                    <TableHead>操作名称</TableHead>
                    <TableHead>操作标识</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {allOpsLoading ? (
                    <TableRow>
                      <TableCell colSpan={3} className="text-center py-8">
                        <Loader2 className="h-6 w-6 animate-spin mx-auto" />
                      </TableCell>
                    </TableRow>
                  ) : availableOps.length === 0 ? (
                    <TableRow>
                      <TableCell
                        colSpan={3}
                        className="text-center py-8 text-muted-foreground"
                      >
                        暂无可添加的操作
                      </TableCell>
                    </TableRow>
                  ) : (
                    availableOps.map((op) => (
                      <TableRow key={op.id}>
                        <TableCell>
                          <Checkbox
                            checked={selectedOpIds.includes(op.id)}
                            onCheckedChange={() => handleToggleOp(op.id)}
                          />
                        </TableCell>
                        <TableCell>{op.op_name}</TableCell>
                        <TableCell>
                          <code className="text-sm bg-muted px-1.5 py-0.5 rounded">
                            {op.op_key}
                          </code>
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
            </div>
          </div>
        </div>
      </DrawerContent>
    </Drawer>
  )
}
