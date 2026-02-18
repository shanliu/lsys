import {
    addExterFeature,
    editExterFeature,
    ExterFeatureAddParamSchema,
    type ExterFeatureItemType,
    type ExterFeatureAddParamType,
} from '@shared/apis/admin/config'
import {
    Drawer,
    DrawerContent,
    DrawerDescription,
    DrawerHeader,
    DrawerTitle,
} from '@apps/main/components/local/drawer'
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
import { formatServerError } from '@shared/lib/utils'
import { zodResolver } from '@hookform/resolvers/zod'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Loader2 } from 'lucide-react'
import { useEffect } from 'react'
import { useForm } from 'react-hook-form'

interface ExterFeatureDrawerProps {
    /** 扩展能力数据，为 undefined 时表示新增 */
    feature?: ExterFeatureItemType
    /** 是否打开 */
    open: boolean
    /** 打开状态变化回调 */
    onOpenChange: (open: boolean) => void
    /** 操作成功后的回调 */
    onSuccess?: () => void
}

export function ExterFeatureDrawer({
    feature,
    open,
    onOpenChange,
    onSuccess,
}: ExterFeatureDrawerProps) {
    const toast = useToast()
    const queryClient = useQueryClient()
    const isEdit = !!feature

    const form = useForm<ExterFeatureAddParamType>({
        resolver: zodResolver(ExterFeatureAddParamSchema),
        defaultValues: {
            feature_key: '',
            title: '',
        },
    })

    // 新增
    const addMutation = useMutation({
        mutationFn: (data: ExterFeatureAddParamType) => addExterFeature(data),
        onSuccess: () => {
            toast.success('添加成功')
            queryClient.invalidateQueries({ queryKey: ['exter-feature-list'] })
            onOpenChange(false)
            form.reset()
            onSuccess?.()
        },
        onError: (error: any) => {
            toast.error(formatServerError(error))
        },
    })

    // 编辑
    const editMutation = useMutation({
        mutationFn: (data: ExterFeatureAddParamType) =>
            editExterFeature({ id: feature!.id, ...data }),
        onSuccess: () => {
            toast.success('修改成功')
            queryClient.invalidateQueries({ queryKey: ['exter-feature-list'] })
            onOpenChange(false)
            onSuccess?.()
        },
        onError: (error: any) => {
            toast.error(formatServerError(error))
        },
    })

    const onSubmit = (data: ExterFeatureAddParamType) => {
        if (isEdit) {
            editMutation.mutate(data)
        } else {
            addMutation.mutate(data)
        }
    }

    // 当抽屉打开或 feature 变化时重置表单
    useEffect(() => {
        if (open) {
            if (feature) {
                form.reset({
                    feature_key: feature.key,
                    title: feature.title,
                })
            } else {
                form.reset({
                    feature_key: '',
                    title: '',
                })
            }
        }
    }, [open, feature, form])

    const isPending = isEdit ? editMutation.isPending : addMutation.isPending

    return (
        <Drawer open={open} onOpenChange={onOpenChange}>
            <DrawerContent>
                <DrawerHeader>
                    <DrawerTitle>{isEdit ? '编辑扩展能力' : '新增扩展能力'}</DrawerTitle>
                    <DrawerDescription>
                        {isEdit ? '修改扩展能力的标识和名称' : '添加一个新的外部扩展能力定义'}
                    </DrawerDescription>
                </DrawerHeader>

                <Form {...form}>
                    <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4 mt-6">
                        <FormField
                            control={form.control}
                            name="feature_key"
                            render={({ field }) => (
                                <FormItem>
                                    <FormLabel>标识</FormLabel>
                                    <FormControl>
                                        <Input
                                            placeholder="如: sms, mail, custom_x"
                                            {...field}
                                            disabled={isEdit}
                                        />
                                    </FormControl>
                                    <FormDescription>
                                        {isEdit
                                            ? '标识创建后不可修改'
                                            : '只能包含数字、字母、下划线和横杠'}
                                    </FormDescription>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />

                        <FormField
                            control={form.control}
                            name="title"
                            render={({ field }) => (
                                <FormItem>
                                    <FormLabel>名称</FormLabel>
                                    <FormControl>
                                        <Input placeholder="如: 短信服务, 邮件服务" {...field} />
                                    </FormControl>
                                    <FormDescription>扩展能力的显示名称</FormDescription>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />

                        <div className="flex justify-end gap-3 pt-4">
                            <Button
                                type="button"
                                variant="outline"
                                onClick={() => onOpenChange(false)}
                            >
                                取消
                            </Button>
                            <Button type="submit" disabled={isPending}>
                                {isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                                {isEdit ? '保存修改' : '创建'}
                            </Button>
                        </div>
                    </form>
                </Form>
            </DrawerContent>
        </Drawer>
    )
}
