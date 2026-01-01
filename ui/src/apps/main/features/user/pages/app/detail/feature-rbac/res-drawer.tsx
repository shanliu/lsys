import {
  appRbacResAdd,
  appRbacResEdit,
  type AppRbacResDataItemType,
} from '@shared/apis/user/rbac'
import { Button } from '@shared/components/ui/button'
import { Checkbox } from '@shared/components/ui/checkbox'
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
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
} from '@apps/main/components/local/drawer'
import { useToast } from '@shared/contexts/toast-context'
import { cn, formatServerError } from '@shared/lib/utils'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Loader2 } from 'lucide-react'
import React from 'react'
import { useForm } from 'react-hook-form'
import { ResFormSchema, type ResFormType } from './res-schema'

interface ResDrawerProps {
  /** 资源数据，为 undefined 时表示新增 */
  resource?: AppRbacResDataItemType
  /** 是否打开 */
  open: boolean
  /** 打开状态变化回调 */
  onOpenChange: (open: boolean) => void
  /** 应用ID */
  appId: number
}

export function ResDrawer({
  resource,
  open,
  onOpenChange,
  appId,
}: ResDrawerProps) {
  const toast = useToast()
  const queryClient = useQueryClient()
  const isEdit = !!resource

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
          use_app_user: false,
          user_param: '',
        },
  })

  const mutation = useMutation({
    mutationFn: (data: ResFormType) =>
      isEdit
        ? appRbacResEdit({
            res_id: resource.id,
            res_name: data.res_name,
            res_type: data.res_type,
            res_data: data.res_data,
          })
        : appRbacResAdd({
            app_id: appId,
            use_app_user: data.use_app_user ?? false,
            user_param: data.user_param ?? '',
            res_name: data.res_name,
            res_type: data.res_type,
            res_data: data.res_data,
          }),
    onSuccess: () => {
      toast.success(isEdit ? '资源更新成功' : '资源添加成功')
      queryClient.invalidateQueries({ queryKey: ['rbac-res-list'] })
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
          use_app_user: false,
          user_param: '',
        })
      }
    }
  }, [open, resource, form])

  // 监听 use_app_user 的变化
  const useAppUser = form.watch('use_app_user')

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
            {/* 新增时显示用户模式选择 */}
            {!isEdit && (
              <>
                <FormField
                  control={form.control}
                  name="use_app_user"
                  render={({ field }) => (
                    <FormItem className="flex flex-row items-start space-x-3 space-y-0 rounded-md border p-4">
                      <FormControl>
                        <Checkbox
                          checked={field.value}
                          onCheckedChange={field.onChange}
                        />
                      </FormControl>
                      <div className="space-y-1 leading-none">
                        <FormLabel>使用应用用户</FormLabel>
                        <FormDescription>
                          启用后将使用应用自己的用户体系
                        </FormDescription>
                      </div>
                    </FormItem>
                  )}
                />

                {!useAppUser && (
                  <FormField
                    control={form.control}
                    name="user_param"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>用户参数</FormLabel>
                        <FormControl>
                          <Input
                            placeholder="用户参数"
                            {...field}
                          />
                        </FormControl>
                        <FormDescription>
                          自定义用户的标识参数
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                )}
              </>
            )}

            <FormField
              control={form.control}
              name="res_name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>资源名称</FormLabel>
                  <FormControl>
                    <Input placeholder="如: 用户管理" {...field} />
                  </FormControl>
                  <FormDescription>用于标识资源的显示名称</FormDescription>
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
                    <Input
                      placeholder="如: user"
                      {...field}
                      disabled={isEdit}
                    />
                  </FormControl>
                  <FormDescription>
                    {isEdit
                      ? '资源类型不可修改'
                      : '资源分类标识，用于关联操作'}
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
                    <Input
                      placeholder="如: 123 或 *"
                      {...field}
                      disabled={isEdit}
                    />
                  </FormControl>
                  <FormDescription>
                    {isEdit
                      ? '资源数据不可修改'
                      : '具体资源标识，如ID或通配符'}
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <div className="flex justify-end gap-2 pt-4">
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
                disabled={mutation.isPending}
              >
                取消
              </Button>
              <Button type="submit" disabled={mutation.isPending}>
                {mutation.isPending && (
                  <Loader2 className={cn('mr-2 h-4 w-4 animate-spin')} />
                )}
                {isEdit ? '保存' : '确定'}
              </Button>
            </div>
          </form>
        </Form>
      </DrawerContent>
    </Drawer>
  )
}
