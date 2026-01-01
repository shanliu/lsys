import {
  hwSmsConfigDel,
  hwSmsConfigList
} from '@shared/apis/admin/sender-sms'
import { ConfirmDialog } from '@shared/components/custom/dialog/confirm-dialog'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { DataTable, DataTableAction, DataTableActionItem } from '@shared/components/custom/table'
import { Button } from '@shared/components/ui/button'
import { useToast } from '@shared/contexts/toast-context'
import { SmsAdapterConfigNavContainer } from '@apps/main/features/admin/components/ui/sms-adapter-config-nav'
import { useIsMobile } from '@shared/hooks/use-mobile'
import { cn, formatServerError, formatTime, getQueryResponseData, TIME_STYLE } from '@shared/lib/utils'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { ColumnDef } from '@tanstack/react-table'
import { Edit2, Plus, Trash2 } from 'lucide-react'
import React from 'react'
import { SmsAdapterConfigHwSmsDrawer } from './adapter-config-huawei-drawer'
import { smsAdapterConfigModuleConfig } from '../nav-info'

import { HwSmsConfigItemType } from '@shared/apis/admin/sender-sms'

export function SmsAdapterConfigHuaweiSmsPage() {
  const toast = useToast()
  const queryClient = useQueryClient()
  const isMobile = useIsMobile()
  const [configDrawerOpen, setConfigDrawerOpen] = React.useState(false)
  const [editingConfig, setEditingConfig] = React.useState<HwSmsConfigItemType | undefined>()

  // 获取配置列表
  const {
    data: configData,
    isLoading,
    isError,
    error,
  } = useQuery({
    queryKey: ['hw-sms-config-list'],
    queryFn: async ({ signal }) => {
      const result = await hwSmsConfigList({}, { signal })
      return result
    },
    placeholderData: (previousData) => previousData,
  })

  const configs = getQueryResponseData<HwSmsConfigItemType[]>(configData, [])
  
  // 检查 API 响应中的错误状态
  const apiError = configData && !configData.status ? configData : null

  // 删除配置
  const deleteMutation = useMutation({
    mutationFn: (id: number) => hwSmsConfigDel({ id }),
    onSuccess: () => {
      toast.success('华为云短信配置删除成功')
      queryClient.invalidateQueries({ queryKey: ['hw-sms-config-list'] })
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  const handleDelete = async (id: number) => {
    await deleteMutation.mutateAsync(id)
  }

  const refreshData = () => {
    queryClient.refetchQueries({ queryKey: ['hw-sms-config-list'] })
  }

  // 定义表格列
  const columns: ColumnDef<HwSmsConfigItemType>[] = [
    {
      accessorKey: 'id',
      header: () => <div className={cn(isMobile ? '' : 'text-right')}>ID</div>,
      size: 60,
      cell: ({ getValue }) => <div className={cn('font-mono text-sm', isMobile ? '' : 'text-right')}>{getValue<number>()}</div>,
    },
    {
      accessorKey: 'name',
      header: '配置名称',
      cell: ({ getValue }) => <div className="font-medium">{getValue<string>()}</div>,
    },
    {
      accessorKey: 'hide_app_key',
      header: 'App Key',
      cell: ({ getValue }) => <div className="font-mono text-sm text-muted-foreground">{getValue<string>()}</div>,
    },
    {
      accessorKey: 'change_time',
      header: '修改时间',
      size: 130,
      cell: ({ getValue }) => (
        <div className="text-sm text-muted-foreground whitespace-nowrap">
          {formatTime(getValue<number>(), TIME_STYLE.ABSOLUTE_TEXT)}
        </div>
      ),
    },
    {
      id: 'actions',
      header: () => <div className="text-center">操作</div>,
      size: 60,
      cell: ({ row }) => {
        const config = row.original

        return (
          <DataTableAction className={cn(isMobile ? 'justify-end' : 'justify-center')}>
            <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
              <Button
                variant="ghost"
                size="sm"
                className={cn('h-7 px-2')}
                onClick={() => {
                  setEditingConfig(config)
                  setConfigDrawerOpen(true)
                }}
              >
                <Edit2 className=" h-4 w-4" />
                <span className="ml-2">编辑</span>
              </Button>
            </DataTableActionItem>

            <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
              <ConfirmDialog
                title="确认删除"
                description={
                  <div>
                    确定要删除华为云短信配置 <strong>{config.name}</strong> 吗？
                    <br />
                    删除后无法恢复。
                  </div>
                }
                onConfirm={() => handleDelete(Number(config.id))}
              >
                <Button
                  size="sm"
                  variant="ghost"
                  className={cn('h-7 px-2 ')}
                  title="删除"
                >
                  <Trash2 className=" h-4 w-4" />
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
    <SmsAdapterConfigNavContainer
      {...smsAdapterConfigModuleConfig}
      actions={
        <Button
          variant="outline"
          size="sm"
          onClick={() => {
            setEditingConfig(undefined)
            setConfigDrawerOpen(true)
          }}
        >
          <Plus className={cn('mr-2 h-4 w-4')} />
          新增配置
        </Button>
      }
    >
      <div className="flex-1 overflow-hidden">
        <DataTable
          data={configs}
          columns={columns}
          loading={isLoading}
          error={isError || apiError ? <CenteredError error={error || apiError} variant="content" onReset={refreshData} /> : null}
          scrollSnapDelay={300}
          className="[&_tr]:h-11 [&_td]:py-1 [&_th]:py-1 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b h-full"
          tableContainerClassName="h-full"
        />
      </div>

      <SmsAdapterConfigHwSmsDrawer
        config={editingConfig}
        open={configDrawerOpen}
        onOpenChange={(open: boolean) => {
          setConfigDrawerOpen(open)
          if (!open) {
            setEditingConfig(undefined)
          }
        }}
      />
    </SmsAdapterConfigNavContainer>
  )
}
