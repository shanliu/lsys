"use client"

import { appOAuthServerSetting } from "@shared/apis/user/app"
import { LoadingButton } from "@apps/main/components/local/sender-config/loading-button"
import { Button } from "@shared/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@shared/components/ui/card"
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from "@shared/components/ui/form"
import { Input } from "@shared/components/ui/input"
import { Drawer, DrawerContent, DrawerDescription, DrawerHeader, DrawerTitle } from "@apps/main/components/local/drawer"
import { useToast } from "@shared/contexts/toast-context"
import { cn, formatServerError } from "@shared/lib/utils"
import { zodResolver } from "@hookform/resolvers/zod"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { Plus, Trash2 } from "lucide-react"
import React from "react"
import { useFieldArray, useForm } from "react-hook-form"
import z from "zod"

// OAuth Server Setting Form Schema
const OAuthServerSettingFormSchema = z.object({
    app_id: z.coerce.number(),
    scope_data: z.array(z.object({
        key: z.string().min(1, "权限标识不能为空"),
        name: z.string().min(1, "权限名称不能为空"),
        desc: z.string().min(1, "权限描述不能为空"),
    })),
})
type OAuthServerSettingFormData = z.infer<typeof OAuthServerSettingFormSchema>

export interface OAuthServerScopeDataItem {
    scope_key: string
    scope_name: string
}

interface SubAppOAuthServerSettingDrawerProps {
    appId: string
    scopeData: OAuthServerScopeDataItem[]
    open: boolean
    onOpenChange: (open: boolean) => void
}

export function SubAppOAuthServerSettingDrawer({ appId, scopeData, open, onOpenChange }: SubAppOAuthServerSettingDrawerProps) {
    const toast = useToast()
    const queryClient = useQueryClient()

    const form = useForm<OAuthServerSettingFormData>({
        resolver: zodResolver(OAuthServerSettingFormSchema),
        defaultValues: {
            app_id: Number(appId),
            scope_data: scopeData.map(item => ({
                key: item.scope_key,
                name: item.scope_name,
                desc: item.scope_name,
            })),
        }
    })

    const { fields, append, remove } = useFieldArray({
        control: form.control,
        name: "scope_data",
    })

    // Sync form with scopeData when it changes
    React.useEffect(() => {
        form.reset({
            app_id: Number(appId),
            scope_data: scopeData.map(item => ({
                key: item.scope_key,
                name: item.scope_name,
                desc: item.scope_name,
            })),
        })
    }, [scopeData, appId, form])

    // 更新OAuth服务设置
    const settingMutation = useMutation({
        mutationFn: (data: OAuthServerSettingFormData) => appOAuthServerSetting(data),
        onSuccess: (result) => {
            if (result.status) {
                toast.success("OAuth服务设置已更新")
                queryClient.invalidateQueries({ queryKey: ['app-oauth-server-detail', appId] })
                onOpenChange(false)
            } else {
                toast.error(formatServerError(result))
            }
        },
        onError: (error: any) => {
            toast.error(formatServerError(error))
        }
    })

    const onSubmit = (data: OAuthServerSettingFormData) => {
        settingMutation.mutate(data)
    }

    const addNewScope = () => {
        append({ key: "", name: "", desc: "" })
    }

    return (
        <Drawer open={open} onOpenChange={onOpenChange}>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>OAuth权限范围设置</DrawerTitle>
                    <DrawerDescription>
                        配置OAuth服务提供的权限范围(scope)
                    </DrawerDescription>
                </DrawerHeader>
                <div className={cn("mt-6")}>
                    <Form {...form}>
                        <form onSubmit={form.handleSubmit(onSubmit)} className={cn("space-y-6")}>
                            <div className={cn("space-y-4")}>
                                {fields.map((field, index) => (
                                    <Card key={field.id}>
                                        <CardHeader className={cn("pb-3")}>
                                            <div className={cn("flex items-center justify-between")}>
                                                <CardTitle className={cn("text-sm")}>权限 #{index + 1}</CardTitle>
                                                <Button
                                                    type="button"
                                                    variant="ghost"
                                                    size="sm"
                                                    onClick={() => remove(index)}
                                                >
                                                    <Trash2 className={cn("h-4 w-4")} />
                                                </Button>
                                            </div>
                                        </CardHeader>
                                        <CardContent className={cn("space-y-3")}>
                                            <FormField
                                                control={form.control}
                                                name={`scope_data.${index}.key`}
                                                render={({ field }) => (
                                                    <FormItem>
                                                        <FormLabel>权限标识</FormLabel>
                                                        <FormControl>
                                                            <Input placeholder="例如: user:read" {...field} />
                                                        </FormControl>
                                                        <FormMessage />
                                                    </FormItem>
                                                )}
                                            />
                                            <FormField
                                                control={form.control}
                                                name={`scope_data.${index}.name`}
                                                render={({ field }) => (
                                                    <FormItem>
                                                        <FormLabel>权限名称</FormLabel>
                                                        <FormControl>
                                                            <Input placeholder="例如: 读取用户信息" {...field} />
                                                        </FormControl>
                                                        <FormMessage />
                                                    </FormItem>
                                                )}
                                            />
                                            <FormField
                                                control={form.control}
                                                name={`scope_data.${index}.desc`}
                                                render={({ field }) => (
                                                    <FormItem>
                                                        <FormLabel>权限描述</FormLabel>
                                                        <FormControl>
                                                            <Input placeholder="详细描述此权限的作用" {...field} />
                                                        </FormControl>
                                                        <FormMessage />
                                                    </FormItem>
                                                )}
                                            />
                                        </CardContent>
                                    </Card>
                                ))}
                            </div>
                            <div className={cn("flex gap-2")}>
                                <Button
                                    type="button"
                                    variant="outline"
                                    onClick={addNewScope}
                                >
                                    <Plus className={cn("mr-2 h-4 w-4")} />
                                    添加权限
                                </Button>
                                <LoadingButton
                                    type="submit"
                                    loading={settingMutation.isPending}
                                >
                                    保存设置
                                </LoadingButton>
                            </div>
                        </form>
                    </Form>
                </div>
            </DrawerContent>
        </Drawer>
    )
}
