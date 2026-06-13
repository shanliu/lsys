import {
    userFileUpdateExpireTime,
    type UserFileItemType,
} from '@shared/apis/user/file'
import { Button } from '@shared/components/ui/button'
import { FormDialog } from '@shared/components/custom/dialog/form-dialog'
import { DateTimePicker } from '@shared/components/custom/input/datetime-picker'
import { useToast } from '@shared/contexts/toast-context'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useState } from 'react'

interface FileExpireTimeDialogProps {
    open: boolean
    onOpenChange: (open: boolean) => void
    appId: number
    file: UserFileItemType
}

export function FileExpireTimeDialog({ open, onOpenChange, appId, file }: FileExpireTimeDialogProps) {
    const queryClient = useQueryClient()
    const { success: showSuccess, error: showError } = useToast()

    const [expireDate, setExpireDate] = useState<Date | undefined>(
        file.expire_time ? new Date(file.expire_time) : undefined,
    )

    const mutation = useMutation({
        mutationFn: () => {
            const expire_time = expireDate ? Math.floor(expireDate.getTime() / 1000) : 0
            return userFileUpdateExpireTime({ app_id: appId, file_ref_id: file.id, expire_time })
        },
        onSuccess: () => {
            showSuccess('过期时间已更新')
            queryClient.invalidateQueries({ queryKey: ['userFileList'] })
            onOpenChange(false)
        },
        onError: (error: any) => {
            showError(error?.data?.message || error?.message || '更新失败')
        },
    })

    return (
        <FormDialog
            open={open}
            onOpenChange={onOpenChange}
            title="更新过期时间"
            description={`更新文件 "${file.file_name}" 的过期时间`}
        >
            <div className="space-y-4">
                <div className="space-y-2">
                    <label className="text-sm font-medium">过期时间</label>
                    <DateTimePicker
                        value={expireDate}
                        onChange={setExpireDate}
                        placeholder="不设置则永不过期"
                        minDateTime={new Date()}
                    />
                    <p className="text-xs text-muted-foreground">
                        不选择日期表示永不过期
                    </p>
                </div>
                <div className="flex justify-end gap-2 pt-4">
                    <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
                        取消
                    </Button>
                    <Button onClick={() => mutation.mutate()} disabled={mutation.isPending}>
                        {mutation.isPending ? '更新中...' : '确认更新'}
                    </Button>
                </div>
            </div>
        </FormDialog>
    )
}
