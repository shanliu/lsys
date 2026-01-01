"use client"

import { Drawer, DrawerContent, DrawerDescription, DrawerHeader, DrawerTitle } from "@apps/main/components/local/drawer"
import { LoadingButton } from "@apps/main/components/local/sender-config/loading-button"
import { appQueryKey } from "@apps/main/lib/auth-utils"
import { zodResolver } from "@hookform/resolvers/zod"
import { appOAuthClientSetDomain, AppOAuthClientSetDomainParamSchema, type AppOAuthClientSetDomainParamType } from "@shared/apis/user/app"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@shared/components/ui/card"
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from "@shared/components/ui/form"
import { Input } from "@shared/components/ui/input"
import { useToast } from "@shared/contexts/toast-context"
import { cn, formatServerError } from "@shared/lib/utils"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import React from "react"
import { useForm } from "react-hook-form"

interface OAuthClientSettingDrawerProps {
    appId: string
    currentDomain?: string
    open: boolean
    onOpenChange: (open: boolean) => void
}

export function OAuthClientSettingDrawer({ appId, currentDomain, open, onOpenChange }: OAuthClientSettingDrawerProps) {
    const toast = useToast()
    const queryClient = useQueryClient()

    const form = useForm<AppOAuthClientSetDomainParamType>({
        resolver: zodResolver(AppOAuthClientSetDomainParamSchema),
        defaultValues: {
            app_id: Number(appId),
            callback_domain: currentDomain || ""
        }
    })

    // 当传入的域名更新时同步表单
    React.useEffect(() => {
        form.reset({
            app_id: Number(appId),
            callback_domain: currentDomain || ""
        })
    }, [currentDomain, appId, form])

    const updateMutation = useMutation({
        mutationFn: (data: AppOAuthClientSetDomainParamType) => appOAuthClientSetDomain(data),
        onSuccess: (result) => {
            if (result.status) {
                toast.success("回调域名配置成功")
                queryClient.invalidateQueries({ queryKey: appQueryKey(appId), exact: false })
                onOpenChange(false)
            } else {
                toast.error(formatServerError(result))
            }
        },
        onError: (error: any) => {
            toast.error(formatServerError(error))
        }
    })

    const onSubmit = (data: AppOAuthClientSetDomainParamType) => {
        updateMutation.mutate(data)
    }

    return (
        <Drawer open={open} onOpenChange={onOpenChange}>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>OAuth客户端设置</DrawerTitle>
                    <DrawerDescription>
                        配置OAuth客户端的回调授权域名
                    </DrawerDescription>
                </DrawerHeader>

                <div className={cn("mt-6")}>
                    <Card>
                        <CardHeader>
                            <CardTitle className={cn("text-base")}>回调授权域名</CardTitle>
                            <CardDescription>
                                设置OAuth 2.0授权回调的允许域名
                            </CardDescription>
                        </CardHeader>
                        <CardContent>
                            <Form {...form}>
                                <form onSubmit={form.handleSubmit(onSubmit)} className={cn("space-y-4")}>
                                    <FormField
                                        control={form.control}
                                        name="callback_domain"
                                        render={({ field }) => (
                                            <FormItem>
                                                <FormLabel>授权域名</FormLabel>
                                                <FormControl>
                                                    <Input
                                                        placeholder="example.com"
                                                        {...field}
                                                    />
                                                </FormControl>
                                                <FormDescription>
                                                    {currentDomain ? "更新OAuth回调授权域名" : "输入OAuth回调授权域名（不包含协议和路径）"}
                                                </FormDescription>
                                                <FormMessage />
                                            </FormItem>
                                        )}
                                    />
                                    <LoadingButton
                                        type="submit"
                                        loading={updateMutation.isPending}
                                    >
                                        {currentDomain ? "更新配置" : "保存配置"}
                                    </LoadingButton>
                                </form>
                            </Form>
                        </CardContent>
                    </Card>
                </div>
            </DrawerContent>
        </Drawer>
    )
}
