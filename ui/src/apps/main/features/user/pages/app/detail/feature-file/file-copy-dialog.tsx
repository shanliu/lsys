import { zodResolver } from '@hookform/resolvers/zod'
import {
    userFileCopy,
    type UserFileItemType,
    type StorageTypeDictItemType,
} from '@shared/apis/user/file'
import { Button } from '@shared/components/ui/button'
import { FormDialog } from '@shared/components/custom/dialog/form-dialog'
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from '@shared/components/ui/form'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@shared/components/ui/select'
import { useToast } from '@shared/contexts/toast-context'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useForm } from 'react-hook-form'
import { z } from 'zod'

interface FileCopyDialogProps {
    open: boolean
    onOpenChange: (open: boolean) => void
    appId: number
    file: UserFileItemType
    storageTypes: StorageTypeDictItemType[]
}

const copySchema = z.object({
    storage_type: z.string().min(1, '请选择目标存储类型'),
})

type CopyForm = z.infer<typeof copySchema>

export function FileCopyDialog({ open, onOpenChange, appId, file, storageTypes }: FileCopyDialogProps) {
    const queryClient = useQueryClient()
    const { success: showSuccess, error: showError } = useToast()

    const isOssFile = !file.storage_type.startsWith('local_')
    const availableStorageTypes = isOssFile 
        ? storageTypes.filter((t) => t.type === 'local')
        : storageTypes

    const form = useForm<CopyForm>({
        resolver: zodResolver(copySchema),
        defaultValues: { 
            storage_type: isOssFile ? 'local_public' : file.storage_type
        },
    })

    const mutation = useMutation({
        mutationFn: (data: CopyForm) =>
            userFileCopy({
                app_id: appId,
                file_ref_id: file.id,
                storage_type: data.storage_type,
            }),
        onSuccess: () => {
            showSuccess('文件已拷贝')
            queryClient.invalidateQueries({ queryKey: ['userFileList'] })
            onOpenChange(false)
        },
        onError: (error: any) => {
            showError(error?.data?.message || error?.message || '拷贝失败')
        },
    })

    return (
        <FormDialog
            open={open}
            onOpenChange={onOpenChange}
            title="拷贝文件"
            description={`拷贝文件 "${file.file_name}" 到指定存储类型`}
        >
            <Form {...form}>
                <form onSubmit={form.handleSubmit((data) => mutation.mutate(data))} className="space-y-4">
                    <FormField
                        control={form.control}
                        name="storage_type"
                        render={({ field }) => (
                            <FormItem>
                                <FormLabel>目标存储类型</FormLabel>
                                <Select onValueChange={field.onChange} defaultValue={field.value}>
                                    <FormControl>
                                        <SelectTrigger>
                                            <SelectValue placeholder="选择目标存储类型" />
                                        </SelectTrigger>
                                    </FormControl>
                                    <SelectContent>
                                        {availableStorageTypes.map((type) => (
                                            <SelectItem key={type.key} value={type.key}>
                                                {type.val}
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                                <FormDescription>
                                    {isOssFile 
                                        ? 'OSS 文件只能拷贝到本地存储类型' 
                                        : '本地文件可以拷贝到任意存储类型'}
                                </FormDescription>
                                <FormMessage />
                            </FormItem>
                        )}
                    />
                    <div className="flex justify-end gap-2 pt-4">
                        <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
                            取消
                        </Button>
                        <Button type="submit" disabled={mutation.isPending}>
                            {mutation.isPending ? '拷贝中...' : '确认拷贝'}
                        </Button>
                    </div>
                </form>
            </Form>
        </FormDialog>
    )
}
