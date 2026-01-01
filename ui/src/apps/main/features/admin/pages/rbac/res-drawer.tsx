import {
  resourceAdd,
  resourceEdit,
  dynamicResourceType,
  type ResourceItemType,
} from '@shared/apis/admin/rbac-res'
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
} from '@apps/main/components/local/drawer'
import { AutocompleteInput } from '@shared/components/custom/input/autocomplete-input'
import { Button } from '@shared/components/ui/button'
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@shared/components/ui/form'
import { Input } from '@shared/components/ui/input'
import { useToast } from '@shared/contexts/toast-context'
import { formatServerError, getQueryResponseData } from '@shared/lib/utils'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Loader2 } from 'lucide-react'
import React from 'react'
import { useForm } from 'react-hook-form'
import { ResFormSchema, type ResFormType } from './res-schema'

interface ResDrawerProps {
  /** 资源数据，为 undefined 时表示新增 */
  resource?: ResourceItemType
  /** 是否打开 */
  open: boolean
  /** 打开状态变化回调 */
  onOpenChange: (open: boolean) => void
}

export function ResDrawer({
  resource,
  open,
  onOpenChange,
}: ResDrawerProps) {
  const toast = useToast()
  const queryClient = useQueryClient()
  const isEdit = !!resource

  // 获取动态资源类型列表
  const { data: resTypeData, isLoading: resTypeLoading } = useQuery({
    queryKey: ['admin-rbac-dynamic-res-type'],
    queryFn: async ({ signal }) => {
      const result = await dynamicResourceType({
        page: { limit: 100 },
        count_num: false,
      }, { signal })
      return result
    },
    staleTime: 5 * 60 * 1000, // 5分钟缓存
  })

  // 将资源类型转换为 AutocompleteInput 需要的格式
  const resTypeOptions = React.useMemo(() => {
    const types = getQueryResponseData<{ id: number; res_type: string; description?: string }[]>(resTypeData, [])
    return types.map(item => ({
      value: item.res_type,
      label: item.description ? `${item.res_type} - ${item.description}` : item.res_type,
    }))
  }, [resTypeData])

  const form = useForm<ResFormType>({
    resolver: zodResolver(ResFormSchema),
    defaultValues: resource
      ? {
          res_name: resource.res_name,
          res_type: resource.res_type,
          res_data: resource.res_data,
        }
      : {
          res_name: '',
          res_type: '',
          res_data: '',
        },
  })

  const mutation = useMutation({
    mutationFn: (data: ResFormType) =>
      isEdit
        ? resourceEdit({
            res_id: resource.id,
            res_name: data.res_name,
            res_type: data.res_type,
            res_data: data.res_data,
          })
        : resourceAdd({
            res_name: data.res_name,
            res_type: data.res_type,
            res_data: data.res_data,
          }),
    onSuccess: () => {
      toast.success(isEdit ? '资源更新成功' : '资源添加成功')
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-res-list'] })
      onOpenChange(false)
      if (!isEdit) {
        form.reset()
      }
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  const onSubmit = (data: ResFormType) => {
    mutation.mutate(data)
  }

  // Reset form when resource changes or dialog opens
  React.useEffect(() => {
    if (open) {
      if (resource) {
        form.reset({
          res_name: resource.res_name,
          res_type: resource.res_type,
          res_data: resource.res_data,
        })
      } else {
        form.reset({
          res_name: '',
          res_type: '',
          res_data: '',
        })
      }
    }
  }, [open, resource, form])

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent>
        <DrawerHeader>
          <DrawerTitle>{isEdit ? '编辑资源' : '新增资源'}</DrawerTitle>
          <DrawerDescription>
            {isEdit ? '修改资源信息' : '填写资源基本信息'}
          </DrawerDescription>
        </DrawerHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4 mt-6">
            <FormField
              control={form.control}
              name="res_name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>资源名称</FormLabel>
                  <FormControl>
                    <Input placeholder="输入资源名称" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="res_type"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>资源类型</FormLabel>
                  <FormControl>
                    <AutocompleteInput
                      value={field.value}
                      onChange={field.onChange}
                      placeholder="输入或选择资源类型"
                      loading={resTypeLoading}
                      options={resTypeOptions}
                      filterOnInput={true}
                    />
                  </FormControl>
                  <FormDescription>
                    资源的分类标识，相同类型的资源共享操作定义
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="res_data"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>资源数据</FormLabel>
                  <FormControl>
                    <Input placeholder="输入资源标识数据" {...field} />
                  </FormControl>
                  <FormDescription>
                    资源的唯一标识，如路由路径、按钮ID等
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <div className="flex justify-end gap-3 pt-4">
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
              >
                取消
              </Button>
              <Button type="submit" disabled={mutation.isPending}>
                {mutation.isPending && (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                )}
                {isEdit ? '保存修改' : '创建资源'}
              </Button>
            </div>
          </form>
        </Form>
      </DrawerContent>
    </Drawer>
  )
}
