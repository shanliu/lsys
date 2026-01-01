import { UserParentAppSelector, type UserParentAppInfo } from '@apps/main/components/selector/user-parent-app-selector'
import { useAuthData } from '@apps/main/hooks/use-auth-data'
import { userQueryKey } from '@apps/main/lib/auth-utils'
import { zodResolver } from '@hookform/resolvers/zod'
import { appAdd, type AppAddParamType } from '@shared/apis/user/app'
import { Button } from '@shared/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@shared/components/ui/card'
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@shared/components/ui/form'
import { Input } from '@shared/components/ui/input'
import { useToast } from '@shared/contexts/toast-context'
import { cn, formatServerError } from '@shared/lib/utils'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { Loader2 } from 'lucide-react'
import { useState } from 'react'
import { useForm } from 'react-hook-form'
import { AppCreateSchema, type AppCreateForm } from './create-schema'

export default function AppCreatePage() {
  //docs\api\user\app\add.md
  //docs\api\user\app\parent_app.md
  const navigate = useNavigate()
  const toast = useToast()
  const queryClient = useQueryClient()
  const [selectedParentApp, setSelectedParentApp] = useState<UserParentAppInfo | null>(null)
  const user = useAuthData()
  const form = useForm<AppCreateForm>({
    resolver: zodResolver(AppCreateSchema),
    defaultValues: {
      parent_app_id: null,
      name: '',
      client_id: '',
    }
  })

  // 当用户处于某个应用上下文时：创建的应用默认作为当前应用的子应用

  // 创建应用的API调用
  const createMutation = useMutation({
    mutationFn: async (data: AppCreateForm) => {
      const params: AppAddParamType = {
        name: data.name,
        client_id: data.client_id,
        parent_app_id: data.parent_app_id,
      }
      return await appAdd(params)
    },
    onSuccess: (res) => {
      toast.success(res.message)
      form.reset()
      setSelectedParentApp(null)
      // 清理应用列表相关的缓存
      queryClient.invalidateQueries({
        queryKey: userQueryKey('appList'),
        exact: false
      })
      const appId = res.response?.id
      navigate({ to: '/user/app/$appId', params: { appId: Number(appId) } })
    },
    onError: (error: any) => {
      const errorMessage = formatServerError(error)
      toast.error(errorMessage)
    }
  })

  const onSubmit = (data: AppCreateForm) => {
    createMutation.mutate(data)
  }

  return (
    <div className="container mx-auto p-4">
      <Card>
        <CardHeader>
          <CardTitle>创建应用</CardTitle>
          <CardDescription>
            请填写以下信息来创建新的应用。应用创建后即可使用。
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Form {...form}>
            <form onSubmit={form.handleSubmit(onSubmit)} className={cn("space-y-6")}>
              {user.appData?.appId && user.appData?.appId > 0 ? (
                <div className={cn('rounded-md border bg-muted p-3')}>
                  <p className={cn('m-0 text-xs text-muted-foreground')}>
                    当前创建应用为{' '}
                    <span className={cn('font-medium text-foreground')}>{user.appData.appName}</span>
                    （<span className={cn('font-mono')}>{user.appData.clientId}</span>）的子应用
                  </p>
                </div>
              ) : <FormField
                control={form.control}
                name="parent_app_id"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>父应用</FormLabel>
                    <FormControl>
                      <UserParentAppSelector
                        value={field.value?.toString() || ""}
                        onValueChange={(appId, appInfo) => {
                          const parentAppId = appId ? Number(appId) : null
                          field.onChange(parentAppId)
                          setSelectedParentApp(appInfo || null)
                        }}
                        placeholder="选择父应用（可选）"
                        showUserInfo={true}
                        triggerClassName="w-full"
                      />
                    </FormControl>
                    <FormMessage />
                    <p className="text-xs text-muted-foreground">
                      可选择一个父应用，如不选择则创建独立应用
                    </p>
                    {selectedParentApp && (
                      <div className="text-xs text-muted-foreground p-2 bg-muted rounded">
                        选中的父应用：{selectedParentApp.name} ({selectedParentApp.client_id})
                      </div>
                    )}
                  </FormItem>
                )}
              />}


              <FormField
                control={form.control}
                name="name"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>应用名称 *</FormLabel>
                    <FormControl>
                      <Input
                        placeholder="请输入应用名称"
                        {...field}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <FormField
                control={form.control}
                name="client_id"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>应用标识 *</FormLabel>
                    <FormControl>
                      <Input
                        placeholder="请输入应用的唯一标识符（如：my-app）"
                        {...field}
                      />
                    </FormControl>
                    <FormMessage />
                    <p className="text-xs text-muted-foreground">
                      应用标识必须是唯一的，建议使用小写字母、数字和短横线组合
                    </p>
                  </FormItem>
                )}
              />

              <div className="flex gap-4">
                <Button
                  type="submit"
                  disabled={createMutation.isPending}
                  className={cn(createMutation.isPending && "opacity-50")}
                >
                  {createMutation.isPending && (
                    <Loader2 className={cn("mr-2 h-4 w-4 animate-spin")} />
                  )}
                  {createMutation.isPending ? '创建中...' : '创建应用'}
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => navigate({ to: '/user/app/list' })}
                >
                  取消
                </Button>
              </div>
            </form>
          </Form>
        </CardContent>
      </Card>
    </div>
  )
}
