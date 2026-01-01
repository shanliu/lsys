import {
  roleAdd,
  roleEdit,
  type RoleItemType,
} from '@shared/apis/admin/rbac-role'
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
} from '@apps/main/components/local/drawer'
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@shared/components/ui/select'
import { useToast } from '@shared/contexts/toast-context'
import { useDictData } from '@apps/main/hooks/use-dict-data'
import { formatServerError } from '@shared/lib/utils'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Loader2 } from 'lucide-react'
import React from 'react'
import { useForm } from 'react-hook-form'
import { RoleFormSchema, type RoleFormType } from './role-schema'

interface RoleDrawerProps {
  /** 角色数据，为 undefined 时表示新增 */
  role?: RoleItemType
  /** 是否打开 */
  open: boolean
  /** 打开状态变化回调 */
  onOpenChange: (open: boolean) => void
}

export function RoleDrawer({
  role,
  open,
  onOpenChange,
}: RoleDrawerProps) {
  const toast = useToast()
  const queryClient = useQueryClient()
  const isEdit = !!role

  // 内部获取字典数据
  const { dictData, isLoading: _dictLoading } = useDictData(['admin_rbac'] as const)

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
        },
  })

  const mutation = useMutation({
    mutationFn: (data: RoleFormType) =>
      isEdit
        ? roleEdit({
            role_id: role.id,
            role_name: data.role_name,
            role_key: data.role_key,
          })
        : roleAdd({
            role_name: data.role_name,
            role_key: data.role_key,
            user_range: data.user_range,
            res_range: data.res_range,
          }),
    onSuccess: () => {
      toast.success(isEdit ? '角色更新成功' : '角色添加成功')
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-role-list'], refetchType: 'all' })
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
        })
      }
    }
  }, [open, role, form])

  // 用户范围选项
  const userRangeOptions = dictData.role_user_range?.map((item) => ({
    value: String(item.key),
    label: item.val,
  })) || []

  // 资源范围选项
  const resRangeOptions = dictData.role_res_range?.map((item) => ({
    value: String(item.key),
    label: item.val,
  })) || []

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
            <FormField
              control={form.control}
              name="role_name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>角色名称</FormLabel>
                  <FormControl>
                    <Input placeholder="输入角色名称" {...field} />
                  </FormControl>
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
                    <Input
                      placeholder="输入角色标识"
                      {...field}
                      disabled={isEdit}
                    />
                  </FormControl>
                  <FormDescription>
                    角色的唯一标识，创建后不可修改
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            {/* 新增时才显示用户范围和资源范围 */}
            {!isEdit && (
              <>
                <FormField
                  control={form.control}
                  name="user_range"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>用户范围</FormLabel>
                      <Select
                        onValueChange={(value) => field.onChange(Number(value))}
                        value={String(field.value)}
                      >
                        <FormControl>
                          <SelectTrigger>
                            <SelectValue placeholder="选择用户范围" />
                          </SelectTrigger>
                        </FormControl>
                        <SelectContent className="max-h-[300px]">
                          {userRangeOptions.map((option) => (
                            <SelectItem key={option.value} value={option.value}>
                              {option.label}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                      <FormDescription>
                        指定用户：需要手动添加用户到角色；任意用户：所有用户自动拥有此角色
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
                      >
                        <FormControl>
                          <SelectTrigger>
                            <SelectValue placeholder="选择资源范围" />
                          </SelectTrigger>
                        </FormControl>
                        <SelectContent className="max-h-[300px]">
                          {resRangeOptions.map((option) => (
                            <SelectItem key={option.value} value={option.value}>
                              {option.label}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                      <FormDescription>
                        包含指定授权：需要手动配置权限；访问任意资源：拥有所有资源权限；禁止指定授权：禁止访问指定资源
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />
              </>
            )}

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
                {isEdit ? '保存修改' : '创建角色'}
              </Button>
            </div>
          </form>
        </Form>
      </DrawerContent>
    </Drawer>
  )
}
