import {
  appNotifySecretChange,
  AppNotifySecretChangeParamSchema,
  appSecretView,
  type AppNotifySecretChangeParamType
} from "@shared/apis/user/app"
import { LoadingButton } from "@apps/main/components/local/sender-config/loading-button"
import { TimeoutInput } from "@shared/components/custom/input/timeout-input"
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error"
import { CenteredLoading } from "@shared/components/custom/page-placeholder/centered-loading"
import { MaskedText } from "@shared/components/custom/text/masked-text"
import { Button } from "@shared/components/ui/button"
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from "@shared/components/ui/form"
import { Input } from "@shared/components/ui/input"
import { Drawer, DrawerContent, DrawerDescription, DrawerHeader, DrawerTitle } from "@apps/main/components/local/drawer"
import { useToast } from "@shared/contexts/toast-context"
import { createCopyWithToast } from "@shared/lib/utils/copy-utils"
import { formatSeconds, formatServerError } from "@shared/lib/utils/format-utils"
import { cn } from "@shared/lib/utils/tools-utils"
import { zodResolver } from "@hookform/resolvers/zod"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { Copy, Edit, RotateCcw } from "lucide-react"
import React from "react"
import { useForm } from "react-hook-form"

interface NotifySecretDrawerProps {
  appId: number | string
  open: boolean
  onOpenChange: (open: boolean) => void
}

// 计算剩余秒数
function calculateRemainingSeconds(timeOut: Date | null): number {
  if (timeOut === null) {
    return 0  // 永不过期
  }
  const now = Date.now()  // 当前时间戳（毫秒）
  const remaining = Math.floor((timeOut.getTime() - now) / 1000)  // 转换为秒
  return remaining > 0 ? remaining : 0  // 如果已过期，返回0
}

export function NotifySecretDrawer({ appId, open, onOpenChange }: NotifySecretDrawerProps) {
  const toast = useToast()
  const queryClient = useQueryClient()
  const [isEditing, setIsEditing] = React.useState(false)
  const copyFn = createCopyWithToast(toast.success, toast.error)

  // 获取回调密钥
  const { data: notifySecret, isLoading, isError, error, refetch } = useQuery({
    queryKey: ['app-notify-secret', appId],
    queryFn: async () => {
      const result = await appSecretView({
        app_id: Number(appId),
        notify_secret: true,
        app_secret: false,
        oauth_secret: false
      })
      return result.response?.notify_secret || null
    },
    enabled: open && !!appId
  })

  const [remainingSeconds, setRemainingSeconds] = React.useState(0)

  const secretForm = useForm<AppNotifySecretChangeParamType>({
    resolver: zodResolver(AppNotifySecretChangeParamSchema),
    defaultValues: {
      app_id: Number(appId),
      secret: "",
      secret_timeout: 0
    }
  })

  // 倒计时更新
  React.useEffect(() => {
    if (!notifySecret?.timeout) return

    const timeOut = notifySecret.timeout

    const updateRemaining = () => {
      const remaining = calculateRemainingSeconds(timeOut)
      setRemainingSeconds(remaining)

      // 如果过期且有超时时间，重新加载数据
      if (remaining === 0 && timeOut !== null) {
        refetch()
      }
    }

    // 立即更新一次
    updateRemaining()

    // 如果是永不过期（null）或已过期，不启动定时器
    if (timeOut === null || calculateRemainingSeconds(timeOut) === 0) {
      return
    }

    // 每秒更新一次
    const timer = setInterval(updateRemaining, 1000)

    return () => clearInterval(timer)
  }, [notifySecret?.timeout, refetch])

  // 进入编辑模式时初始化表单
  React.useEffect(() => {
    if (isEditing && notifySecret) {
      const timeOut = notifySecret.timeout
      secretForm.reset({
        app_id: Number(appId),
        secret: notifySecret.secret,
        secret_timeout: calculateRemainingSeconds(timeOut)
      })
    }
  }, [isEditing, notifySecret, appId, secretForm])

  const secretChangeMutation = useMutation({
    mutationFn: (data: AppNotifySecretChangeParamType) => appNotifySecretChange(data),
    onSuccess: (result) => {
      toast.success("回调密钥更新成功")
      if (result.response?.data) {
        toast.success(`新密钥: ${result.response.data}`)
      }
      setIsEditing(false)
      queryClient.invalidateQueries({ queryKey: ['app-notify-secret', appId] })
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    }
  })

  const handleReset = () => {
    secretChangeMutation.mutate({
      app_id: Number(appId),
      secret_timeout: 0,
    })
  }

  const onSecretSubmit = (data: AppNotifySecretChangeParamType) => {
    secretChangeMutation.mutate(data)
  }

  const handleCancelEdit = () => {
    setIsEditing(false)
    secretForm.reset()
  }

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent>
        <DrawerHeader>
          <DrawerTitle>回调密钥管理</DrawerTitle>
          <DrawerDescription>
            管理应用的回调通知密钥
          </DrawerDescription>
        </DrawerHeader>
        <div className="mt-6">
          {isLoading && (
            <CenteredLoading variant="content" />
          )}

          {isError && (
            <CenteredError
              variant="content"
              error={error}
              onReset={() => refetch()}
            />
          )}

          {!isLoading && !isError && notifySecret && (
            <>
              {!isEditing ? (
                // 显示模式
                <div className="space-y-4">
                  <div className="p-4 border rounded-md bg-muted">
                    <p className="text-sm font-medium mb-2">当前密钥</p>
                    <div className="flex items-center gap-2 mb-3">
                      <div className="font-mono text-sm flex-1">
                        <MaskedText
                          text={notifySecret.secret}
                          type="secret"
                          clickable={true}
                        />
                      </div>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8 flex-shrink-0"
                        onClick={() => copyFn(notifySecret.secret, "已复制密钥")}
                      >
                        <Copy className="h-4 w-4" />
                      </Button>
                    </div>
                    <p className="text-xs text-muted-foreground">
                      过期时间: {notifySecret.timeout === null ? "永不过期" : formatSeconds(remainingSeconds)}
                    </p>
                  </div>

                  <div className="flex gap-2">
                    <Button
                      variant="outline"
                      className="flex-1"
                      onClick={handleReset}
                      disabled={secretChangeMutation.isPending}
                    >
                      <RotateCcw className={cn("mr-2 h-4 w-4", secretChangeMutation.isPending && "animate-spin")} />
                      重置
                    </Button>
                    <Button
                      variant="outline"
                      className="flex-1"
                      onClick={() => setIsEditing(true)}
                    >
                      <Edit className=" h-4 w-4" />
                    <span className="ml-2">编辑</span>
                    </Button>
                  </div>
                </div>
              ) : (
                // 编辑模式
                <Form {...secretForm}>
                  <form onSubmit={secretForm.handleSubmit(onSecretSubmit)} className="space-y-4">
                    <FormField
                      control={secretForm.control}
                      name="secret"
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel>新密钥</FormLabel>
                          <FormControl>
                            <Input placeholder="输入新的回调密钥（留空则自动生成）" {...field} />
                          </FormControl>
                          <FormMessage />
                        </FormItem>
                      )}
                    />
                    <FormField
                      control={secretForm.control}
                      name="secret_timeout"
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel>超时时间</FormLabel>
                          <FormControl>
                            <TimeoutInput
                              value={field.value}
                              onChange={field.onChange}
                              placeholder="请输入超时时间（秒）"
                            />
                          </FormControl>
                          <FormDescription>
                            0表示永不过期
                          </FormDescription>
                          <FormMessage />
                        </FormItem>
                      )}
                    />
                    <div className="flex gap-2">
                      <Button
                        type="button"
                        variant="outline"
                        className="flex-1"
                        onClick={handleCancelEdit}
                        disabled={secretChangeMutation.isPending}
                      >
                        取消
                      </Button>
                      <LoadingButton
                        type="submit"
                        className="flex-1"
                        loading={secretChangeMutation.isPending}
                      >
                        保存
                      </LoadingButton>
                    </div>
                  </form>
                </Form>
              )}
            </>
          )}
        </div>
      </DrawerContent>
    </Drawer>
  )
}
