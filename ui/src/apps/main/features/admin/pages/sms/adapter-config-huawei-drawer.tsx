import {
  hwSmsConfigAdd,
  hwSmsConfigEdit
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
import { cn, formatServerError } from '@shared/lib/utils'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Loader2 } from 'lucide-react'
import React from 'react'
import { useForm } from 'react-hook-form'
import { HwSmsConfigFormSchema, type HwSmsConfigFormType } from './adapter-config-schema'

import { HwSmsConfigItemType } from '@shared/apis/admin/sender-sms'

interface SmsAdapterConfigHwSmsDrawerProps {
  config?: HwSmsConfigItemType
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function SmsAdapterConfigHwSmsDrawer({ config, open, onOpenChange }: SmsAdapterConfigHwSmsDrawerProps) {
  const toast = useToast()
  const queryClient = useQueryClient()
  const isEdit = !!config

  const form = useForm<HwSmsConfigFormType>({
    resolver: zodResolver(HwSmsConfigFormSchema),
    defaultValues: {
      name: '',
      url: '',
      app_key: '',
      app_secret: '',
      callback_key: '',
      limit: 0,
    },
  })

  // 添加配置
  const addMutation = useMutation({
    mutationFn: (data: HwSmsConfigFormType) => hwSmsConfigAdd(data),
    onSuccess: () => {
      toast.success('华为云短信配置添加成功')
      queryClient.invalidateQueries({ queryKey: ['hw-sms-config-list'] })
      onOpenChange(false)
      form.reset()
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  // 编辑配置
  const editMutation = useMutation({
    mutationFn: (data: HwSmsConfigFormType & { id: number }) => hwSmsConfigEdit(data),
    onSuccess: () => {
      toast.success('华为云短信配置更新成功')
      queryClient.invalidateQueries({ queryKey: ['hw-sms-config-list'] })
      onOpenChange(false)
      form.reset()
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  const onSubmit = (data: HwSmsConfigFormType) => {
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
        url: config.url,
        app_key: config.app_key,
        app_secret: config.app_secret || '',
        callback_key: config.callback_key || '',
        limit: Number(config.limit) || 0,
      })
    } else if (open && !config) {
      form.reset({
        name: '',
        url: '',
        app_key: '',
        app_secret: '',
        callback_key: '',
        limit: 0,
      })
    }
  }, [open, config, form])

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent>
        <DrawerHeader>
          <DrawerTitle>{isEdit ? '编辑华为云短信配置' : '新增华为云短信配置'}</DrawerTitle>
          <DrawerDescription>
            {isEdit ? '修改现有的华为云短信服务器配置' : '添加新的华为云短信服务器配置'}
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
                    <Input placeholder="例如：华为云短信主配置" {...field} />
                  </FormControl>
                  <FormDescription>用于标识此配置的名称</FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="url"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>API地址</FormLabel>
                  <FormControl>
                    <Input placeholder="https://api.example.com" {...field} />
                  </FormControl>
                  <FormDescription>华为云短信API服务地址</FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="app_key"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>App Key</FormLabel>
                  <FormControl>
                    <Input placeholder="输入App Key" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="app_secret"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>App Secret</FormLabel>
                  <FormControl>
                    <PasswordInput placeholder="输入App Secret" {...field} />
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
                    <Input placeholder="选填，用于回调验证" {...field} />
                  </FormControl>
                  <FormDescription>用于验证回调请求的密钥</FormDescription>
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
                  <FormDescription>每日发送限制数量，0表示不限制</FormDescription>
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
