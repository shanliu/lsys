import {
  neteaseSmsConfigAdd,
  neteaseSmsConfigEdit
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
import { NeteaseSmsConfigFormSchema, type NeteaseSmsConfigFormType } from './adapter-config-schema'

import { NeteaseSmsConfigItemType } from '@shared/apis/admin/sender-sms'

interface SmsAdapterConfigNeteaseSmsDrawerProps {
  config?: NeteaseSmsConfigItemType
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function SmsAdapterConfigNeteaseSmsDrawer({ config, open, onOpenChange }: SmsAdapterConfigNeteaseSmsDrawerProps) {
  const toast = useToast()
  const queryClient = useQueryClient()
  const isEdit = !!config

  const form = useForm<NeteaseSmsConfigFormType>({
    resolver: zodResolver(NeteaseSmsConfigFormSchema),
    defaultValues: {
      name: '',
      access_key: '',
      access_secret: '',
      limit: 0,
    },
  })

  const addMutation = useMutation({
    mutationFn: (data: NeteaseSmsConfigFormType) => neteaseSmsConfigAdd(data),
    onSuccess: () => {
      toast.success('网易云短信配置添加成功')
      queryClient.invalidateQueries({ queryKey: ['netease-sms-config-list'] })
      onOpenChange(false)
      form.reset()
    },
    onError: (error: any) => toast.error(formatServerError(error, '添加失败')),
  })

  const editMutation = useMutation({
    mutationFn: (data: NeteaseSmsConfigFormType & { id: number }) => neteaseSmsConfigEdit(data),
    onSuccess: () => {
      toast.success('网易云短信配置更新成功')
      queryClient.invalidateQueries({ queryKey: ['netease-sms-config-list'] })
      onOpenChange(false)
      form.reset()
    },
    onError: (error: any) => toast.error(formatServerError(error, '更新失败')),
  })

  const onSubmit = (data: NeteaseSmsConfigFormType) => {
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
        access_key: config.access_key,
        access_secret: config.access_secret || '',
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
          <DrawerTitle>{isEdit ? '编辑网易云短信配置' : '新增网易云短信配置'}</DrawerTitle>
          <DrawerDescription>
            {isEdit ? '修改现有的网易云短信配置' : '添加新的网易云短信配置'}
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
                    <Input placeholder="例如：网易云短信主配置" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="access_key"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Access Key</FormLabel>
                  <FormControl>
                    <Input placeholder="输入Access Key" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="access_secret"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Access Secret</FormLabel>
                  <FormControl>
                    <PasswordInput placeholder="输入Access Secret" {...field} />
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
