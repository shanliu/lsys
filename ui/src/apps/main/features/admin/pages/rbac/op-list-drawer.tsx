import { FilterContainer } from '@apps/main/components/filter-container/container'
import { FilterActions } from '@apps/main/components/filter-container/filter-actions'
import { FilterInput } from '@apps/main/components/filter-container/filter-input'
import { FilterTotalCount } from '@apps/main/components/filter-container/filter-total-count'
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
} from '@apps/main/components/local/drawer'
import {
  DEFAULT_PAGE_SIZE,
  useCountNumManager,
} from '@apps/main/lib/pagination-utils'
import { zodResolver } from '@hookform/resolvers/zod'
import {
  opAdd,
  opDelete,
  opEdit,
  opList,
  type OpItemType,
} from '@shared/apis/admin/rbac-op'
import { ConfirmDialog } from '@shared/components/custom/dialog/confirm-dialog'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { PagePagination } from '@shared/components/custom/pagination'
import { DataTable, DataTableAction, DataTableActionItem } from '@shared/components/custom/table'
import { Badge } from '@shared/components/ui/badge'
import { Button } from '@shared/components/ui/button'
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@shared/components/ui/form'
import { Input } from '@shared/components/ui/input'
import { useToast } from '@shared/contexts/toast-context'
import { useIsMobile } from '@shared/hooks/use-mobile'
import { cn, formatServerError, getQueryResponseData } from '@shared/lib/utils'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Cog, Edit, Loader2, Plus, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { OpFormSchema, OpListFilterFormSchema, type OpFormType } from './res-schema'

interface OpListDrawerProps {
  /** 是否打开 */
  open: boolean
  /** 打开状态变化回调 */
  onOpenChange: (open: boolean) => void
}

export function OpListDrawer({
  open,
  onOpenChange,
}: OpListDrawerProps) {
  const toast = useToast()
  const queryClient = useQueryClient()
  const isMobile = useIsMobile()

  // 过滤条件
  const [filterParams, setFilterParams] = useState<{
    op_name?: string
    op_key?: string
    page: number
    limit: number
  }>({
    page: 1,
    limit: DEFAULT_PAGE_SIZE,
  })

  // 编辑/新增状态
  const [editingOp, setEditingOp] = useState<OpItemType | undefined>()
  const [isFormOpen, setIsFormOpen] = useState(false)

  // 过滤条件对象
  const filters = {
    op_name: filterParams.op_name ?? null,
    op_key: filterParams.op_key ?? null,
  }

  // count_num 优化管理器
  const countNumManager = useCountNumManager(filters)

  // 获取操作列表
  const { data: opData, isSuccess, isLoading, isError, error } = useQuery({
    queryKey: [
      'admin-rbac-op-list',
      filterParams.op_name,
      filterParams.op_key,
      filterParams.page,
      filterParams.limit,
    ],
    queryFn: async ({ signal }) => {
      const result = await opList(
        {
          op_name: filterParams.op_name,
          op_key: filterParams.op_key,
          res_type_count: true,
          check_role_use: true,
          page: {
            page: filterParams.page,
            limit: filterParams.limit,
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
    countNumManager.handlePageQueryResult(opData)
  }

  // 从查询结果中提取数据
  const operations = getQueryResponseData<OpItemType[]>(opData, [])

  // 删除操作
  const deleteMutation = useMutation({
    mutationFn: (opId: number) => opDelete({ op_id: opId }),
    onSuccess: () => {
      toast.success('操作删除成功')
      countNumManager.reset()
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-op-list'] })
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  // 刷新数据
  const refreshData = () => {
    queryClient.refetchQueries({ queryKey: ['admin-rbac-op-list'] })
  }

  // 清除缓存并重新加载
  const clearCacheAndReload = () => {
    countNumManager.reset()
    queryClient.invalidateQueries({ queryKey: ['admin-rbac-op-list'] })
  }

  // 打开编辑表单
  const handleEdit = (op: OpItemType) => {
    setEditingOp(op)
    setIsFormOpen(true)
  }

  // 打开新增表单
  const handleAdd = () => {
    setEditingOp(undefined)
    setIsFormOpen(true)
  }

  // 表格列定义
  const columns = [
    {
      accessorKey: 'id',
      header: () => <div className={cn(isMobile ? '' : 'text-right')}>ID</div>,
      size: 80,
      cell: ({ getValue }: any) => (
        <div className={cn('font-mono text-sm', isMobile ? '' : 'text-right')}>{getValue()}</div>
      ),
    },
    {
      accessorKey: 'op_name',
      header: '操作名称',
      cell: ({ getValue }: any) => (
        <div className="font-medium">{getValue()}</div>
      ),
    },
    {
      accessorKey: 'op_key',
      header: '操作标识',
      cell: ({ getValue }: any) => (
        <code className="text-sm bg-muted px-1.5 py-0.5 rounded">{getValue()}</code>
      ),
    },
    {
      accessorKey: 'res_type_count',
      header: '已关联资源类型',
      size: 120,
      cell: ({ getValue }: any) => {
        const count = getValue()
        return count !== undefined ? (
          <Badge variant="secondary" className="text-xs font-normal">{count}个</Badge>
        ) : '-'
      },
    },
    {
      accessorKey: 'is_role_use',
      header: '是否关联角色',
      size: 100,
      cell: ({ getValue }: any) => {
        const isUsed = getValue()
        return isUsed !== undefined ? (
          <Badge variant={isUsed ? "default" : "outline"} className="text-xs font-normal">
            {isUsed ? "已关联" : "未关联"}
          </Badge>
        ) : '-'
      },
    },
    {
      id: 'actions',
      header: () => <div className={cn(isMobile ? '' : 'text-center')}>操作</div>,
      size: 120,
      cell: ({ row }: any) => {
        const op = row.original as OpItemType
        return (
          <DataTableAction className={cn(isMobile ? 'justify-end' : 'justify-center')}>
            <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
              <Button
                variant="ghost"
                size="sm"
                className="h-7 px-2"
                onClick={() => handleEdit(op)}
              >
                <Edit className="h-4 w-4" />
                <span className="ml-2">编辑</span>
              </Button>
            </DataTableActionItem>
            <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
              <ConfirmDialog
                title="删除操作"
                description={`确定要删除操作「${op.op_name}」吗？删除后无法恢复。`}
                onConfirm={async () => {
                  await deleteMutation.mutateAsync(op.id)
                }}
              >
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-7 px-2"
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

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent className={cn('overflow-hidden flex flex-col p-0 md:w-[900px]')} showCloseButton={false}>
        <DrawerHeader className="pt-4 md:pt-6">
          <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
            <div>
              <DrawerTitle className="flex items-center gap-2">
                <Cog className="h-5 w-5" />
                操作权限管理
              </DrawerTitle>
              <DrawerDescription>
                管理系统中的操作权限定义，如增删改查等
              </DrawerDescription>
            </div>
            <Button onClick={handleAdd} size="sm" className="self-end sm:self-auto">
              <Plus className="h-4 w-4 mr-1" />
              新增操作
            </Button>
          </div>
        </DrawerHeader>

        <div className="flex-1 overflow-hidden flex flex-col pb-4 md:pb-6">
          {/* 过滤器 */}
          <div className="flex-shrink-0 mt-4 mb-3">
            <FilterContainer
              defaultValues={{
                op_name: filterParams.op_name,
                op_key: filterParams.op_key,
              }}
              resolver={zodResolver(OpListFilterFormSchema) as any}
              onSubmit={(data) => {
                setFilterParams((prev) => ({
                  ...prev,
                  ...data,
                  page: 1,
                }))
              }}
              onReset={() => {
                setFilterParams({
                  page: 1,
                  limit: DEFAULT_PAGE_SIZE,
                })
              }}
              countComponent={
                <FilterTotalCount total={countNumManager.getTotal() ?? 0} loading={isLoading} />
              }
              className="bg-muted/50 rounded-lg border relative"
            >
              {(layoutParams, form) => (
                <div className="flex-1 flex flex-wrap items-end gap-2">
                  <FilterInput
                    name="op_name"
                    placeholder="操作名称"
                    label="名称"
                    disabled={isLoading}
                    layoutParams={layoutParams}
                  />

                  <FilterInput
                    name="op_key"
                    placeholder="操作标识"
                    label="标识"
                    disabled={isLoading}
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

          {/* 操作列表 - 表格模式 */}
          <div className="flex-1 min-h-0 overflow-hidden">
            <DataTable
              data={operations}
              columns={columns}
              loading={isLoading}
              error={isError ? <CenteredError error={error} variant="content" onReset={refreshData} /> : null}
              scrollSnapDelay={300}
              className="[&_tr]:h-11 [&_td]:py-1 [&_th]:py-1 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b h-full"
              tableContainerClassName="h-full"
            />
          </div>

          {/* 分页 */}
          <div className="flex-shrink-0 pt-3 border-t mt-3">
            <PagePagination
              currentPage={filterParams.page}
              pageSize={filterParams.limit}
              total={countNumManager.getTotal() ?? 0}
              loading={isLoading}
              onChange={(page: number) => {
                setFilterParams((prev) => ({ ...prev, page }))
              }}
              onPageSizeChange={(limit: number) => {
                setFilterParams((prev) => ({ ...prev, page: 1, limit }))
              }}
            />
          </div>

          {/* 新增/编辑表单抽屉 */}
          {isFormOpen && (
            <OpForm
              op={editingOp}
              onClose={() => setIsFormOpen(false)}
              onSuccess={() => {
                setIsFormOpen(false)
                countNumManager.reset()
                queryClient.invalidateQueries({ queryKey: ['admin-rbac-op-list'] })
              }}
            />
          )}
        </div>
      </DrawerContent>
    </Drawer>
  )
}

// 操作表单组件
interface OpFormProps {
  op?: OpItemType
  onClose: () => void
  onSuccess: () => void
}

function OpForm({ op, onClose, onSuccess }: OpFormProps) {
  const toast = useToast()
  const isEdit = !!op

  const form = useForm<OpFormType>({
    resolver: zodResolver(OpFormSchema),
    defaultValues: op
      ? {
        op_name: op.op_name,
        op_key: op.op_key,
      }
      : {
        op_name: '',
        op_key: '',
      },
  })

  const mutation = useMutation({
    mutationFn: async (data: OpFormType) => {
      if (isEdit) {
        return opEdit({
          op_id: op.id,
          op_name: data.op_name,
          op_key: data.op_key,
        })
      }
      return opAdd({
        op_name: data.op_name,
        op_key: data.op_key,
      })
    },
    onSuccess: () => {
      toast.success(isEdit ? '操作更新成功' : '操作添加成功')
      onSuccess()
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  const onSubmit = (data: OpFormType) => {
    mutation.mutate(data)
  }

  return (
    <Drawer open={true} onOpenChange={(open) => !open && onClose()}>
      <DrawerContent className={cn('flex flex-col md:w-[480px]')} showCloseButton={false}>
        <DrawerHeader className="px-4 pt-4 pb-3 md:px-6 md:pt-6 md:pb-4">
          <DrawerTitle>{isEdit ? '编辑操作' : '新增操作'}</DrawerTitle>
          <DrawerDescription>
            {isEdit ? '修改操作权限信息' : '添加新的操作权限'}
          </DrawerDescription>
        </DrawerHeader>

        <div className="flex-1 overflow-y-auto px-4 pb-4 md:px-6 md:pb-6">
          <Form {...form}>
            <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
              <FormField
                control={form.control}
                name="op_name"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>操作名称</FormLabel>
                    <FormControl>
                      <Input placeholder="如：查看、添加、编辑、删除" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="op_key"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>操作标识</FormLabel>
                    <FormControl>
                      <Input placeholder="如：view、add、edit、delete" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <div className="flex justify-end gap-3 pt-4">
                <Button type="button" variant="outline" onClick={onClose}>
                  取消
                </Button>
                <Button type="submit" disabled={mutation.isPending}>
                  {mutation.isPending && (
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  )}
                  {isEdit ? '保存修改' : '创建操作'}
                </Button>
              </div>
            </form>
          </Form>
        </div>
      </DrawerContent>
    </Drawer>
  )
}
