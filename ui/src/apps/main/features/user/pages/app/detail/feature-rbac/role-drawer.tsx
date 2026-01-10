import {
  appRbacRoleAdd,
  appRbacRoleEdit,
  type AppRbacRoleDataItemType,
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
import { useToast } from '@shared/contexts/toast-context'
import { useDictData } from '@apps/main/hooks/use-dict-data'
import { cn, formatServerError } from '@shared/lib/utils'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Loader2 } from 'lucide-react'
import React from 'react'
import { useForm } from 'react-hook-form'
import { RoleFormSchema, type RoleFormType } from './role-schema'

interface RoleDrawerProps {
  /** 角色数据，为 undefined 时表示新增 */
  role?: AppRbacRoleDataItemType
  /** 是否打开 */
  open: boolean
  /** 打开状态变化回调 */
  onOpenChange: (open: boolean) => void
  /** 应用ID */
  appId: number
}

export function RoleDrawer({
  role,
  open,
  onOpenChange,
  appId,
}: RoleDrawerProps) {
  const toast = useToast()
  const queryClient = useQueryClient()
  const isEdit = !!role

  // 内部获取字典数据
  const { dictData, isLoading: _dictLoading } = useDictData(['app_rbac'] as const)
  const userRangeDict = dictData.role_user_range
  const resRangeDict = dictData.role_res_range

  const form = useForm<RoleFormType>({
    resolver: zodResolver(RoleFormSchema),
    defaultValues: role
      ? {
          role_name: role.role_name,
          role_key: role.role_key,
          user_range: role.user_range,
          res_range: role.res_range,
        }
      : {
          role_name: '',
          role_key: '',
          user_range: 1,
          res_range: 1,
          use_app_user: true,
          user_param: '',
        },
  })

  const mutation = useMutation({
    mutationFn: (data: RoleFormType) =>
      isEdit
        ? appRbacRoleEdit({
            role_id: role.id,
            role_name: data.role_name,
            role_key: data.role_key || '',
          })
        : appRbacRoleAdd({
            app_id: appId,
            use_app_user: data.use_app_user ?? false,
            user_param: data.user_param ?? '',
            role_name: data.role_name,
            role_key: data.role_key || '',
            user_range: data.user_range,
            res_range: data.res_range,
          }),
    onSuccess: () => {
      toast.success(isEdit ? '角色更新成功' : '角色添加成功')
      queryClient.invalidateQueries({ queryKey: ['rbac-role-list'] })
      onOpenChange(false)
      if (!isEdit) {
        form.reset()
      }
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  const onSubmit = (data: RoleFormType) => {
    mutation.mutate(data)
  }

  // Reset form when role changes or dialog opens
  React.useEffect(() => {
    if (open) {
      if (role) {
        form.reset({
          role_name: role.role_name,
          role_key: role.role_key,
          user_range: role.user_range,
          res_range: role.res_range,
        })
      } else {
        form.reset({
          role_name: '',
          role_key: '',
          user_range: 1,
          res_range: 1,
          use_app_user: false,
          user_param: '',
        })
      }
    }
  }, [open, role, form])

  // 监听 use_app_user 的变化
  const useAppUser = form.watch('use_app_user')

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent>
        <DrawerHeader>
          <DrawerTitle>{isEdit ? '编辑角色' : '新增角色'}</DrawerTitle>
          <DrawerDescription>
            {isEdit ? '修改角色信息' : '填写角色基本信息'}
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
                        <FormLabel>应用本身</FormLabel>
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
              name="role_name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>角色名称</FormLabel>
                  <FormControl>
                    <Input placeholder="如: 管理员" {...field} />
                  </FormControl>
                  <FormDescription>用于标识角色的显示名称</FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="role_key"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>角色标识</FormLabel>
                  <FormControl>
                    <Input placeholder="如: admin (可选)" {...field} />
                  </FormControl>
                  <FormDescription>
                    用于会话角色鉴权时的标识符，可为空
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="user_range"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>用户范围</FormLabel>
                  <Select
                    onValueChange={(value) => field.onChange(Number(value))}
                    value={String(field.value)}
                    disabled={isEdit}
                  >
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder="请选择用户范围" />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent className="max-h-[300px]">
                      {userRangeDict?.map((item) => (
                        <SelectItem key={item.key} value={item.key}>
                          {item.val}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <FormDescription>
                    {isEdit
                      ? '用户范围不可修改'
                      : '指定角色的用户关联方式'}
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="res_range"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>资源范围</FormLabel>
                  <Select
                    onValueChange={(value) => field.onChange(Number(value))}
                    value={String(field.value)}
                    disabled={isEdit}
                  >
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder="请选择资源范围" />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent className="max-h-[300px]">
                      {resRangeDict?.map((item) => (
                        <SelectItem key={item.key} value={item.key}>
                          {item.val}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <FormDescription>
                    {isEdit
                      ? '资源范围不可修改'
                      : '指定角色的资源访问范围'}
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
