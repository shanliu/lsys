"use client"

import { userSenderSmsNotifyGetConfig, userSenderSmsNotifySetConfig, UserSenderSmsNotifySetConfigParamSchema, type UserSenderSmsNotifySetConfigParamType } from "@shared/apis/user/sender-sms"
import { LoadingButton } from "@apps/main/components/local/sender-config/loading-button"
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error"
import { CenteredLoading } from "@shared/components/custom/page-placeholder/centered-loading"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@shared/components/ui/card"
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from "@shared/components/ui/form"
import { Drawer, DrawerContent, DrawerDescription, DrawerHeader, DrawerTitle } from "@apps/main/components/local/drawer"
import { Textarea } from "@shared/components/ui/textarea"
import { useToast } from "@shared/contexts/toast-context"
import { cn, formatServerError, getQueryResponseData } from "@shared/lib/utils"
import { zodResolver } from "@hookform/resolvers/zod"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import React from "react"
import { useForm } from "react-hook-form"


interface ListNotifyDrawerProps {
    appId: string
    open: boolean
    onOpenChange: (open: boolean) => void
}

export function ListNotifyDrawer({ appId, open, onOpenChange }: ListNotifyDrawerProps) {
    const toast = useToast()
    const queryClient = useQueryClient()

    // 查询回调地址配置
    const configQuery = useQuery({
        queryKey: ['sender-sms-notify-config', appId],
        queryFn: ({ signal }) => userSenderSmsNotifyGetConfig({ signal }),
        enabled: open && !!appId
    })

    // 从返回的数组中找到当前 app_id 对应的配置
    const configList = getQueryResponseData<Array<{ app_id: number; call_url?: string | null }>>(configQuery.data, [])
    const currentConfig = configList.find(config => config.app_id === Number(appId))
    const callbackUrl = currentConfig?.call_url || ""

    const form = useForm<UserSenderSmsNotifySetConfigParamType>({
        resolver: zodResolver(UserSenderSmsNotifySetConfigParamSchema),
        defaultValues: {
            app_id: Number(appId),
            url: callbackUrl
        }
    })

    // 当查询数据更新时同步表单
    React.useEffect(() => {
        form.reset({
            app_id: Number(appId),
            url: callbackUrl
        })
    }, [callbackUrl, appId, form])

    const updateMutation = useMutation({
        mutationFn: (data: UserSenderSmsNotifySetConfigParamType) => userSenderSmsNotifySetConfig(data),
        onSuccess: (result) => {
            if (result.status) {
                toast.success("回调地址配置成功")
                queryClient.invalidateQueries({ queryKey: ['sender-sms-notify-config', appId] })
            } else {
                toast.error(formatServerError(result))
            }
        },
        onError: (error: any) => {
            toast.error(formatServerError(error))
        }
    })

    const onSubmit = (data: UserSenderSmsNotifySetConfigParamType) => {
        updateMutation.mutate(data)
    }

    return (
        <Drawer open={open} onOpenChange={onOpenChange}>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>短信通知配置</DrawerTitle>
                    <DrawerDescription>
                        配置短信发送的回调通知地址
                    </DrawerDescription>
                </DrawerHeader>

                <div className={cn("mt-6")}>
                    {configQuery.isLoading ? (
                        <CenteredLoading variant="content" />
                    ) : configQuery.isError ? (
                        <CenteredError variant="content" error={configQuery.error} />
                    ) : (
                        <Card>
                            <CardHeader>
                                <CardTitle className={cn("text-base")}>回调通知地址</CardTitle>
                                <CardDescription>
                                    设置短信发送事件的回调通知URL
                                </CardDescription>
                            </CardHeader>
                            <CardContent>
                                <Form {...form}>
                                    <form onSubmit={form.handleSubmit(onSubmit)} className={cn("space-y-4")}>
                                        <FormField
                                            control={form.control}
                                            name="url"
                                            render={({ field }) => (
                                                <FormItem>
                                                    <FormLabel>回调地址</FormLabel>
                                                    <FormControl>
                                                        <Textarea
                                                            placeholder="https://example.com/callback"
                                                            className="resize-none min-h-24"
                                                            {...field}
                                                        />
                                                    </FormControl>
                                                    <FormDescription>
                                                        {callbackUrl ? "更新回调通知地址" : "输入回调通知地址"}
                                                    </FormDescription>
                                                    <FormMessage />
                                                </FormItem>
                                            )}
                                        />
                                        <LoadingButton
                                            type="submit"
                                            loading={updateMutation.isPending}
                                        >
                                            {callbackUrl ? "更新配置" : "保存配置"}
                                        </LoadingButton>
                                    </form>
                                </Form>
                            </CardContent>
                        </Card>
                    )}
                </div>
            </DrawerContent>
        </Drawer>
    )
}
