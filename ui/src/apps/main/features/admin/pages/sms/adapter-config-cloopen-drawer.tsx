import {
  cloopenSmsConfigAdd,
  cloopenSmsConfigEdit
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
import { CloopenSmsConfigFormSchema, type CloopenSmsConfigFormType } from './adapter-config-schema'

import { CloopenSmsConfigItemType } from '@shared/apis/admin/sender-sms'

interface SmsAdapterConfigCloopenSmsDrawerProps {
  config?: CloopenSmsConfigItemType
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function SmsAdapterConfigCloopenSmsDrawer({ config, open, onOpenChange }: SmsAdapterConfigCloopenSmsDrawerProps) {
  const toast = useToast()
  const queryClient = useQueryClient()
  const isEdit = !!config

  const form = useForm<CloopenSmsConfigFormType>({
    resolver: zodResolver(CloopenSmsConfigFormSchema),
    defaultValues: {
      name: '',
      account_sid: '',
      account_token: '',
      sms_app_id: '',
      callback_key: '',
      limit: 0,
    },
  })

  const addMutation = useMutation({
    mutationFn: (data: CloopenSmsConfigFormType) => cloopenSmsConfigAdd(data),
    onSuccess: () => {
      toast.success('容联云短信配置添加成功')
      queryClient.invalidateQueries({ queryKey: ['cloopen-sms-config-list'] })
      onOpenChange(false)
      form.reset()
    },
    onError: (error: any) => toast.error(formatServerError(error, '添加失败')),
  })

  const editMutation = useMutation({
    mutationFn: (data: CloopenSmsConfigFormType & { id: number }) => cloopenSmsConfigEdit(data),
    onSuccess: () => {
      toast.success('容联云短信配置更新成功')
      queryClient.invalidateQueries({ queryKey: ['cloopen-sms-config-list'] })
      onOpenChange(false)
      form.reset()
    },
    onError: (error: any) => toast.error(formatServerError(error, '更新失败')),
  })

  const onSubmit = (data: CloopenSmsConfigFormType) => {
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
        account_sid: config.account_sid,
        account_token: config.account_token || '',
        sms_app_id: config.sms_app_id,
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
          <DrawerTitle>{isEdit ? '编辑容联云短信配置' : '新增容联云短信配置'}</DrawerTitle>
          <DrawerDescription>
            {isEdit ? '修改现有的容联云短信配置' : '添加新的容联云短信配置'}
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
                    <Input placeholder="例如：容联云短信主配置" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="account_sid"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Account SID</FormLabel>
                  <FormControl>
                    <Input placeholder="输入Account SID" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="account_token"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Account Token</FormLabel>
                  <FormControl>
                    <PasswordInput placeholder="输入Account Token" {...field} />
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
                    <Input
                      placeholder="输入SMS应用ID"
                      {...field}
                    />
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
                  <FormDescription>用于验证回调请求的密钥</FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="limit"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>发送限制</FormLabel>
                  <FormControl>
                    <Input
                      type="number"
                      placeholder="0"
                      {...field}
                      value={field.value ?? ''}
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
