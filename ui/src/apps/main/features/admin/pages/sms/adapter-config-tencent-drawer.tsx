import {
  tencentSmsConfigAdd,
  tencentSmsConfigEdit
} from '@shared/apis/admin/sender-sms'
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
} from '@apps/main/components/local/drawer'
import { PasswordInput } from '@shared/components/custom/input/password-input'
import { Button } from '@shared/components/ui/button'
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from '@shared/components/ui/form'
import { Input } from '@shared/components/ui/input'
import { useToast } from '@shared/contexts/toast-context'
import { cn, formatServerError } from '@shared/lib/utils'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Loader2 } from 'lucide-react'
import React from 'react'
import { useForm } from 'react-hook-form'
import { TencentSmsConfigFormSchema, type TencentSmsConfigFormType } from './adapter-config-schema'

import { TencentSmsConfigItemType } from '@shared/apis/admin/sender-sms'

interface SmsAdapterConfigTencentSmsDrawerProps {
  config?: TencentSmsConfigItemType
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function SmsAdapterConfigTencentSmsDrawer({ config, open, onOpenChange }: SmsAdapterConfigTencentSmsDrawerProps) {
  const toast = useToast()
  const queryClient = useQueryClient()
  const isEdit = !!config

  const form = useForm<TencentSmsConfigFormType>({
    resolver: zodResolver(TencentSmsConfigFormSchema),
    defaultValues: {
      name: '',
      region: 'ap-guangzhou',
      secret_id: '',
      secret_key: '',
      sms_app_id: '',
      callback_key: '',
      limit: 0,
    },
  })

  const addMutation = useMutation({
    mutationFn: (data: TencentSmsConfigFormType) => tencentSmsConfigAdd(data),
    onSuccess: () => {
      toast.success('腾讯云短信配置添加成功')
      queryClient.invalidateQueries({ queryKey: ['tencent-sms-config-list'] })
      onOpenChange(false)
      form.reset()
    },
    onError: (error: any) => toast.error(formatServerError(error, '添加失败')),
  })

  const editMutation = useMutation({
    mutationFn: (data: TencentSmsConfigFormType & { id: number }) => tencentSmsConfigEdit(data),
    onSuccess: () => {
      toast.success('腾讯云短信配置更新成功')
      queryClient.invalidateQueries({ queryKey: ['tencent-sms-config-list'] })
      onOpenChange(false)
      form.reset()
    },
    onError: (error: any) => toast.error(formatServerError(error, '更新失败')),
  })

  const onSubmit = (data: TencentSmsConfigFormType) => {
    if (config) {
      editMutation.mutate({ ...data, id: config.id })
    } else {
      addMutation.mutate(data)
    }
  }

  const isSubmitting = addMutation.isPending || editMutation.isPending

  React.useEffect(() => {
    if (open && config) {
      form.reset({
        name: config.name,
        region: config.region,
        secret_id: config.secret_id,
        secret_key: config.secret_key || '',
        sms_app_id: String(config.sms_app_id),
        callback_key: config.callback_key || '',
        limit: Number(config.limit) || 0,
      })
    } else if (open && !config) {
      form.reset()
    }
  }, [open, config, form])

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent>
        <DrawerHeader>
          <DrawerTitle>{isEdit ? '编辑腾讯云短信配置' : '新增腾讯云短信配置'}</DrawerTitle>
          <DrawerDescription>
            {isEdit ? '修改现有的腾讯云短信配置' : '添加新的腾讯云短信配置'}
          </DrawerDescription>
        </DrawerHeader>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4 mt-6">
            <FormField
              control={form.control}
              name="name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>配置名称</FormLabel>
                  <FormControl>
                    <Input placeholder="例如：腾讯云短信主配置" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="region"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>地域</FormLabel>
                  <FormControl>
                    <Input placeholder="例如：ap-guangzhou" {...field} />
                  </FormControl>
                  <FormDescription>腾讯云服务地域</FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="secret_id"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Secret ID</FormLabel>
                  <FormControl>
                    <Input placeholder="输入Secret ID" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="secret_key"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Secret Key</FormLabel>
                  <FormControl>
                    <PasswordInput placeholder="输入Secret Key" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="sms_app_id"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>SMS App ID</FormLabel>
                  <FormControl>
                    <Input placeholder="输入SMS应用ID" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="callback_key"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>回调密钥</FormLabel>
                  <FormControl>
                    <Input placeholder="选填" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="limit"
              render={({ field: { value, ...field } }) => (
                <FormItem>
                  <FormLabel>发送限制</FormLabel>
                  <FormControl>
                    <Input
                      type="number"
                      placeholder="0"
                      {...field}
                      value={value ?? ''}
                      onChange={(e) => field.onChange(e.target.valueAsNumber)}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <div className="flex gap-3 pt-4">
              <Button type="submit" className={cn("flex-1")} disabled={isSubmitting}>
                {isSubmitting && <Loader2 className={cn('mr-2 h-4 w-4 animate-spin')} />}
                {isEdit ? '更新配置' : '添加配置'}
              </Button>
            </div>
          </form>
        </Form>
      </DrawerContent>
    </Drawer>
  )
}
