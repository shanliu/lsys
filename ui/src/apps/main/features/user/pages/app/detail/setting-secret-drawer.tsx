import { appSecretAdd, appSecretChange, appSecretDel, appSecretView, type AppSecretAddParamType, type AppSecretChangeParamType, type AppSecretDelParamType } from "@shared/apis/user/app"
import { LoadingButton } from "@apps/main/components/local/sender-config/loading-button"
import { ConfirmDialog } from "@shared/components/custom/dialog/confirm-dialog"
import { TimeoutInput } from "@shared/components/custom/input/timeout-input"
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error"
import { CenteredLoading } from "@shared/components/custom/page-placeholder/centered-loading"
import { MaskedText } from "@shared/components/custom/text/masked-text"
import { Button } from "@shared/components/ui/button"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@shared/components/ui/card"
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from "@shared/components/ui/form"
import { Input } from "@shared/components/ui/input"
import { Drawer, DrawerClose, DrawerContent, DrawerDescription, DrawerFooter, DrawerHeader, DrawerTitle } from "@apps/main/components/local/drawer"
import { useToast } from "@shared/contexts/toast-context"
import { createCopyWithToast } from "@shared/lib/utils/copy-utils"
import { formatSeconds, formatServerError } from "@shared/lib/utils/format-utils"
import { calculateRemainingSeconds, cn } from "@shared/lib/utils/tools-utils"
import { zodResolver } from "@hookform/resolvers/zod"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { Copy, Edit, Plus, Trash2 } from "lucide-react"
import React from "react"
import { useForm } from "react-hook-form"
import { AppSecretItem, SecretFormData, SecretFormSchema } from "./setting-schema"

interface AppSettingSecretDrawerProps {
    appId: string
    open: boolean
    onOpenChange: (open: boolean) => void
}

export function AppSettingSecretDrawer({ appId, open, onOpenChange }: AppSettingSecretDrawerProps) {
    const toast = useToast()
    const queryClient = useQueryClient()
    const [editingId, setEditingId] = React.useState<string | null>(null)
    const copyFn = createCopyWithToast(toast.success, toast.error)

    const { data: secretQueryData, isLoading, isError, error } = useQuery({
        queryKey: ['app-secrets', appId],
        queryFn: async () => {
            const result = await appSecretView({
                app_id: Number(appId),
                app_secret: true,
                oauth_secret: false,
                notify_secret: false
            })
            return result
        },
        enabled: open && !!appId
    })

    const secretData = secretQueryData?.response?.app_secret ?? []

    const addMutation = useMutation({
        mutationFn: (data: AppSecretAddParamType) => appSecretAdd(data),
        onSuccess: (result) => {
            if (result.status) {
                toast.success(result.message)
                queryClient.invalidateQueries({ queryKey: ['app-secrets', appId] })
            } else {
                toast.error(formatServerError(result))
            }
        },
        onError: (error: any) => {
            toast.error(formatServerError(error))
        }
    })

    const handleQuickAdd = () => {
        addMutation.mutate({
            app_id: Number(appId),
            secret: undefined,
            secret_timeout: 0
        })
    }

    const changeMutation = useMutation({
        mutationFn: (data: AppSecretChangeParamType) => appSecretChange(data),
        onSuccess: async (result, variables) => {
            if (result.status) {
                toast.success(result.message)
                await queryClient.invalidateQueries({ queryKey: ['app-secrets', appId] })
                setEditingId(null)
            } else {
                toast.error(formatServerError(result))
            }
        },
        onError: (error: any, variables) => {
            toast.error(formatServerError(error))
            setEditingId(null)
        }
    })

    const deleteMutation = useMutation({
        mutationFn: (data: AppSecretDelParamType) => appSecretDel(data),
        onSuccess: (result, variables) => {
            if (result.status) {
                toast.success(result.message)
                queryClient.invalidateQueries({ queryKey: ['app-secrets', appId] })
            } else {
                toast.error(formatServerError(result))
            }
        },
        onError: (error: any, variables) => {
            toast.error(formatServerError(error))
        }
    })

    const handleEdit = (secret: AppSecretItem) => {
        setEditingId(secret.secret_data)
    }

    const handleSaveEdit = (secret: AppSecretItem, newData: SecretFormData) => {
        changeMutation.mutate({
            app_id: Number(appId),
            old_secret: secret.secret_data,
            secret: newData.secret || secret.secret_data,
            secret_timeout: newData.secret_timeout
        })
    }

    const handleCancelEdit = () => {
        setEditingId(null)
    }

    const handleDelete = (secret: AppSecretItem) => {
        deleteMutation.mutate({
            app_id: Number(appId),
            old_secret: secret.secret_data
        })
    }

    const isPendingForSecret = (secret: AppSecretItem) => {
        return deleteMutation.isPending && deleteMutation.variables?.old_secret === secret.secret_data
    }

    const isEditingSecret = (secret: AppSecretItem) => {
        return editingId === secret.secret_data
    }

    const isSavingSecret = (secret: AppSecretItem) => {
        return changeMutation.isPending && changeMutation.variables?.old_secret === secret.secret_data
    }

    const isAddingNewSecret = addMutation.isPending

    return (
        <Drawer open={open} onOpenChange={onOpenChange}>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>密钥管理</DrawerTitle>
                    <DrawerDescription>
                        管理应用密钥，用于API调用认证
                    </DrawerDescription>
                </DrawerHeader>

                <div className={cn("mt-6 space-y-4")}>
                    {/* Secret list */}
                    {isLoading && (
                        <CenteredLoading variant="content" />
                    )}

                    {isError && (
                        <CenteredError
                            variant="card"
                            error={error}
                        />
                    )}

                    {!isLoading && !isError && (
                        <>
                            {secretData && secretData.length > 0 && (
                                secretData.map((secret, index) => (
                                    <SecretCard
                                        key={secret.secret_data}
                                        secret={secret}
                                        index={index}
                                        isEditing={isEditingSecret(secret)}
                                        isSaving={isSavingSecret(secret)}
                                        isPending={isPendingForSecret(secret)}
                                        onEdit={() => handleEdit(secret)}
                                        onSave={(newData) => handleSaveEdit(secret, newData)}
                                        onCancel={handleCancelEdit}
                                        onDelete={() => handleDelete(secret)}
                                        copyFn={copyFn}
                                        appId={appId}
                                    />
                                ))
                            )}

                            {/* Add new secret card */}
                            {isAddingNewSecret ? (
                                <Card className={cn("relative")}>
                                    <div className={cn("absolute inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center z-10 rounded-lg")}>
                                        <CenteredLoading variant="content" iconSize="md" />
                                    </div>
                                    <CardHeader>
                                        <CardTitle className={cn("text-sm font-medium")}>添加新密钥</CardTitle>
                                        <CardDescription>正在创建永不过期的密钥...</CardDescription>
                                    </CardHeader>
                                    <CardContent className={cn("h-20")} />
                                </Card>
                            ) : (
                                <Card className={cn("border-dashed cursor-pointer hover:border-primary hover:bg-accent/50 transition-colors")} onClick={handleQuickAdd}>
                                    <CardContent className={cn("flex flex-col items-center justify-center py-12")} >
                                        <Plus className={cn("h-8 w-8 mb-2 text-muted-foreground")} />
                                        <p className={cn("text-sm font-medium")}>添加新密钥</p>
                                        <p className={cn("text-xs text-muted-foreground mt-1")}>点击创建永不过期的密钥</p>
                                    </CardContent>
                                </Card>
                            )}
                        </>
                    )}
                </div>

                <DrawerFooter className={cn("mt-6")}>
                    <DrawerClose asChild>
                        <Button type="button" variant="outline">
                            关闭
                        </Button>
                    </DrawerClose>
                </DrawerFooter>
            </DrawerContent>
        </Drawer >
    )
}

interface SecretCardProps {
    secret: AppSecretItem
    index: number
    isEditing: boolean
    isSaving: boolean
    isPending: boolean
    onEdit: () => void
    onSave: (data: SecretFormData) => void
    onCancel: () => void
    onDelete: () => void
    copyFn: (text: string, message?: string) => void
    appId: string
}
function SecretCard({
    secret,
    index,
    isEditing,
    isSaving,
    isPending,
    onEdit,
    onSave,
    onCancel,
    onDelete,
    copyFn,
    appId
}: SecretCardProps) {
    const queryClient = useQueryClient()
    const [remainingSeconds, setRemainingSeconds] = React.useState(() => calculateRemainingSeconds(secret.time_out))

    const editForm = useForm<SecretFormData>({
        resolver: zodResolver(SecretFormSchema),
        defaultValues: {
            secret: secret.secret_data,
            secret_timeout: calculateRemainingSeconds(secret.time_out)
        }
    })

    // 倒计时更新
    React.useEffect(() => {
        const updateRemaining = () => {
            const newRemaining = calculateRemainingSeconds(secret.time_out)
            const prevRemaining = remainingSeconds
            setRemainingSeconds(newRemaining)

            // 如果倒计时结束且之前不为0（表示刚刚过期），刷新数据
            if (newRemaining <= 0 && prevRemaining > 0 && secret.time_out !== null) {
                queryClient.invalidateQueries({ queryKey: ['app-secrets', appId] })
            }
        }

        // 立即更新一次
        updateRemaining()

        // 每秒更新一次
        const timer = setInterval(updateRemaining, 1000)

        return () => clearInterval(timer)
    }, [secret.time_out, remainingSeconds, queryClient, appId])

    React.useEffect(() => {
        if (isEditing) {
            editForm.reset({
                secret: secret.secret_data,
                secret_timeout: calculateRemainingSeconds(secret.time_out)
            })
        }
    }, [isEditing, secret, editForm])

    const handleSave = () => {
        const data = editForm.getValues()
        onSave(data)
    }

    if (isPending) {
        return (
            <Card className={cn("relative")}>
                <div className={cn("absolute inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center z-10 rounded-lg")}>
                    <CenteredLoading variant="content" iconSize="md" />
                </div>
                <CardHeader>
                    <CardTitle className={cn("text-sm font-medium")}>密钥 #{index + 1}</CardTitle>
                    <CardDescription>
                        有效时间:{formatSeconds(remainingSeconds)}
                    </CardDescription>
                </CardHeader>
                <CardContent>
                    <div className={cn("flex items-center gap-2")}>
                        <code className={cn("flex-1 rounded bg-muted px-2 py-1 text-xs break-all")}>
                            <MaskedText
                                text={secret.secret_data}
                                type="secret"
                                clickable={true}
                                onRevealedClick={() => copyFn(secret.secret_data, "已复制密钥")}
                            />
                        </code>
                    </div>
                </CardContent>
                <CardFooter className={cn("flex gap-2")}>
                    <Button variant="outline" size="sm" className={cn("flex-1")} disabled>
                        <Edit className={cn("mr-2 h-4 w-4")} />
                        编辑
                    </Button>
                    <Button variant="outline" size="sm" className={cn("flex-1 ")} disabled>
                        <Trash2 className={cn("mr-2 h-4 w-4")} />
                        删除
                    </Button>
                </CardFooter>
            </Card >
        )
    }

    if (isEditing) {
        return (
            <Card>
                <CardHeader>
                    <CardTitle className={cn("text-sm font-medium")}>编辑密钥 #{index + 1}</CardTitle>
                </CardHeader>
                <CardContent>
                    <Form {...editForm}>
                        <div className={cn("space-y-4")}>
                            <FormField
                                control={editForm.control}
                                name="secret"
                                render={({ field }) => (
                                    <FormItem>
                                        <FormLabel>密钥值</FormLabel>
                                        <FormControl>
                                            <Input
                                                placeholder="请输入新密钥"
                                                {...field}
                                            />
                                        </FormControl>
                                        <FormMessage />
                                    </FormItem>
                                )}
                            />
                            <FormField
                                control={editForm.control}
                                name="secret_timeout"
                                render={({ field }) => (
                                    <FormItem>
                                        <FormLabel>超时时间</FormLabel>
                                        <FormControl className="w-full">
                                            <TimeoutInput
                                                value={field.value}
                                                onChange={field.onChange}
                                                placeholder="请输入超时时间"
                                            />
                                        </FormControl>
                                        <FormDescription>
                                            0表示永不过期
                                        </FormDescription>
                                        <FormMessage />
                                    </FormItem>
                                )}
                            />
                        </div>
                    </Form>
                </CardContent>
                <CardFooter className={cn("flex gap-2")}>
                    <Button
                        variant="outline"
                        size="sm"
                        className={cn("flex-1")}
                        onClick={onCancel}
                        disabled={isSaving}
                    >
                        取消
                    </Button>
                    <LoadingButton
                        size="sm"
                        className={cn("flex-1")}
                        onClick={handleSave}
                        loading={isSaving}
                    >
                        保存
                    </LoadingButton>
                </CardFooter>
            </Card>
        )
    }

    return (
        <Card>
            <CardHeader>
                <CardTitle className={cn("text-sm font-medium")}>密钥 #{index + 1}</CardTitle>
                <CardDescription>
                    有效时间: {formatSeconds(remainingSeconds)}
                </CardDescription>
            </CardHeader>
            <CardContent>
                <div className={cn("flex items-center gap-2")}>
                    <code className={cn("flex-1 rounded bg-muted px-2 py-1 text-xs break-all")}>
                        <MaskedText
                            text={secret.secret_data}
                            type="secret"
                            clickable={true}
                            onRevealedClick={() => copyFn(secret.secret_data, "已复制密钥")}
                        />
                    </code>
                    <Button
                        variant="ghost"
                        size="icon"
                        className={cn("flex-shrink-0 h-8 w-8")}
                        onClick={() => copyFn(secret.secret_data, "已复制密钥")}
                    >
                        <Copy className={cn("h-4 w-4")} />
                    </Button>
                </div>
            </CardContent>
            <CardFooter className={cn("flex gap-2")}>
                <Button
                    variant="outline"
                    size="sm"
                    className={cn("flex-1")}
                    onClick={onEdit}
                >
                    <Edit className={cn("mr-2 h-4 w-4")} />
                    编辑
                </Button>
                <ConfirmDialog
                    title="确认删除密钥"
                    description={
                        <div>
                            <p>您确定要删除此密钥吗？</p>
                        </div>
                    }
                    onConfirm={onDelete}
                >
                    <Button
                        variant="outline"
                        size="sm"
                        className={cn("flex-1 ")}
                    >
                        <Trash2 className={cn("mr-2 h-4 w-4")} />
                        删除
                    </Button>
                </ConfirmDialog>
            </CardFooter>
        </Card>
    )
}
