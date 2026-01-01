import {
  resourceTypeOpData,
  resourceTypeOpAdd,
  resourceTypeOpDel,
  type ResourceOpItemType,
} from '@shared/apis/admin/rbac-res'
import {
  opList,
  type OpItemType,
} from '@shared/apis/admin/rbac-op'
import { ConfirmDialog } from '@shared/components/custom/dialog/confirm-dialog'
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
} from '@apps/main/components/local/drawer'
import { Button } from '@shared/components/ui/button'
import { Checkbox } from '@shared/components/ui/checkbox'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@shared/components/ui/table'
import { useToast } from '@shared/contexts/toast-context'
import { formatServerError, getQueryResponseData } from '@shared/lib/utils'
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
}

export function ResTypeOpsDrawer({
  resType,
  open,
  onOpenChange,
}: ResTypeOpsDrawerProps) {
  const toast = useToast()
  const queryClient = useQueryClient()
  const [selectedOpIds, setSelectedOpIds] = React.useState<number[]>([])

  // 获取资源类型已关联的操作列表
  const { data: resOpsData, isLoading: resOpsLoading } = useQuery({
    queryKey: ['admin-rbac-res-type-ops', resType],
    queryFn: async ({ signal }) => {
      const result = await resourceTypeOpData(
        {
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
    queryKey: ['admin-rbac-op-list-all'],
    queryFn: async ({ signal }) => {
      const result = await opList(
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

  const resOps = getQueryResponseData<ResourceOpItemType[]>(resOpsData, [])
  const allOps = getQueryResponseData<OpItemType[]>(allOpsData, [])

  // 过滤出未关联的操作（从 op_data.id 获取已关联的操作ID）
  const resOpIds = resOps.map((op) => op.op_data.id)
  const availableOps = allOps.filter((op) => !resOpIds.includes(op.id))

  // 添加操作关联
  const addOpMutation = useMutation({
    mutationFn: () =>
      resourceTypeOpAdd({
        res_type: resType,
        op_ids: selectedOpIds,
      }),
    onSuccess: () => {
      toast.success('操作关联成功')
      queryClient.invalidateQueries({
        queryKey: ['admin-rbac-res-type-ops', resType],
      })
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-op-list'] })
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-op-list-all'] })
      setSelectedOpIds([])
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  // 删除操作关联
  const deleteOpMutation = useMutation({
    mutationFn: (opId: number) =>
      resourceTypeOpDel({
        res_type: resType,
        op_ids: [opId],
      }),
    onSuccess: () => {
      toast.success('操作移除成功')
      queryClient.invalidateQueries({
        queryKey: ['admin-rbac-res-type-ops', resType],
      })
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-op-list'] })
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-op-list-all'] })
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

  const isLoading = resOpsLoading || allOpsLoading

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent>
        <DrawerHeader>
          <DrawerTitle className="flex items-center gap-2">
            <Settings className="h-5 w-5" />
            资源类型操作管理
          </DrawerTitle>
          <DrawerDescription>
            管理资源类型「{resType}」的操作关联
          </DrawerDescription>
        </DrawerHeader>

        <div className="mt-6 space-y-6">
          {/* 添加操作关联区域 */}
          {availableOps.length > 0 && (
            <div className="space-y-3 p-4 border rounded-lg bg-muted/50">
              <h4 className="font-medium">添加操作关联</h4>
              <div className="space-y-2 max-h-[200px] overflow-y-auto">
                {availableOps.map((op) => (
                  <div
                    key={op.id}
                    className="flex items-center space-x-3 p-2 rounded hover:bg-muted"
                  >
                    <Checkbox
                      id={`op-${op.id}`}
                      checked={selectedOpIds.includes(op.id)}
                      onCheckedChange={() => handleToggleOp(op.id)}
                    />
                    <label
                      htmlFor={`op-${op.id}`}
                      className="flex-1 text-sm cursor-pointer"
                    >
                      <span className="font-medium">{op.op_name}</span>
                      <code className="ml-2 text-xs text-muted-foreground bg-muted px-1 py-0.5 rounded">
                        {op.op_key}
                      </code>
                    </label>
                  </div>
                ))}
              </div>
              <Button
                onClick={() => addOpMutation.mutate()}
                disabled={selectedOpIds.length === 0 || addOpMutation.isPending}
                size="sm"
              >
                {addOpMutation.isPending && (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                )}
                <Plus className="mr-2 h-4 w-4" />
                关联选中操作 ({selectedOpIds.length})
              </Button>
            </div>
          )}

          {/* 已关联操作列表 */}
          <div className="space-y-3">
            <h4 className="font-medium">已关联操作 ({resOps.length})</h4>
            {isLoading ? (
              <div className="flex items-center justify-center py-8">
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
              </div>
            ) : resOps.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                暂无关联操作
              </div>
            ) : (
              <div className="border rounded-lg overflow-hidden">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>ID</TableHead>
                      <TableHead>操作名称</TableHead>
                      <TableHead>操作标识</TableHead>
                      <TableHead className="text-center">操作</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {resOps.map((op) => (
                      <TableRow key={op.op_res.id}>
                        <TableCell className="font-mono text-sm">{op.op_data.id}</TableCell>
                        <TableCell className="font-medium">{op.op_data.op_name}</TableCell>
                        <TableCell>
                          <code className="text-sm bg-muted px-1.5 py-0.5 rounded">
                            {op.op_data.op_key}
                          </code>
                        </TableCell>
                        <TableCell className="text-center">
                          <ConfirmDialog
                            title="移除操作关联"
                            description={`确定要从资源类型中移除操作「${op.op_data.op_name}」吗？`}
                            onConfirm={async () => {
                              await deleteOpMutation.mutateAsync(op.op_data.id)
                            }}
                          >
                            <Button
                              variant="ghost"
                              size="sm"
                              className="h-7 px-2 "
                            >
                              <Trash2 className="h-4 w-4" />
                            </Button>
                          </ConfirmDialog>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            )}
          </div>
        </div>
      </DrawerContent>
    </Drawer>
  )
}
