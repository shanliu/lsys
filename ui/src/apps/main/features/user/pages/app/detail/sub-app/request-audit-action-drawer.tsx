import {
    appConfirm,
    appConfirmExterFeature,
    appOAuthServerClientConfirm,
    appOAuthServerClientScopeConfirm,
    type AppSubRequestItemType
} from '@shared/apis/user/app'
import { Badge } from '@shared/components/ui/badge'
import { Button } from '@shared/components/ui/button'
import { Label } from '@shared/components/ui/label'
import { RadioGroup, RadioGroupItem } from '@shared/components/ui/radio-group'
import { Separator } from '@shared/components/ui/separator'
import { Drawer, DrawerContent, DrawerDescription, DrawerHeader, DrawerTitle } from '@apps/main/components/local/drawer'
import { Textarea } from '@shared/components/ui/textarea'
import { useToast } from '@shared/contexts/toast-context'
import { SubAppRequestDataDisplay } from '@apps/main/features/user/components/ui/sub-app-request-data-display'
import { cn, formatServerError, formatTime, TIME_STYLE } from '@shared/lib/utils'
import { DictList } from '@shared/types/apis-dict'
import { useMutation } from '@tanstack/react-query'
import { Loader2 } from 'lucide-react'
import { useState } from 'react'

interface SubAppRequestAuditActionDrawerProps {
    /** 申请数据 */
    request: AppSubRequestItemType | null
    /** 是否打开 */
    open: boolean
    /** 打开状态变化回调 */
    onOpenChange: (open: boolean) => void
    /** 审核完成回调 - 通知父组件刷新数据 */
    onAuditComplete?: () => void
    /** 请求类型字典 */
    requestTypeDict?: DictList
    /** 请求状态字典 */
    requestStatusDict?: DictList
}

/**
 * 根据申请类型调用对应的审核接口
 */
const performAudit = async (request: AppSubRequestItemType, confirmStatus: number, confirmNote: string) => {
    // 子应用申请类型映射:
    // 1: 新子应用申请 -> /api/user/app/confirm
    // 2: 子应用更改申请 -> /api/user/app/confirm
    // 6: 子应用OAuth登录申请 -> /api/user/app/oauth_server_client_confirm
    // 7: 子应用OAuth登录新增权限申请 -> /api/user/app/oauth_server_client_scope_confirm
    // 8: 子应用外部功能申请 -> /api/user/app/confirm_exter_feature

    switch (request.request_type) {
        case 1: // 新子应用申请
        case 2: // 子应用更改申请
            return await appConfirm({
                app_req_id: request.id,
                confirm_status: confirmStatus,
                confirm_note: confirmNote,
            })
        case 6: // 子应用OAuth登录申请
            return await appOAuthServerClientConfirm({
                app_id: request.app_id,
                confirm_status: confirmStatus,
                confirm_note: confirmNote,
            })
        case 7: // 子应用OAuth登录新增权限申请
            return await appOAuthServerClientScopeConfirm({
                app_req_id: request.id,
                confirm_status: confirmStatus,
                confirm_note: confirmNote,
            })
        case 8: // 子应用外部功能申请
            return await appConfirmExterFeature({
                app_req_id: request.id,
                confirm_status: confirmStatus,
                confirm_note: confirmNote,
            })
        default:
            throw new Error(`未知的申请类型: ${request.request_type}`)
    }
}

export function SubAppRequestAuditActionDrawer({
    request,
    open,
    onOpenChange,
    onAuditComplete,
    requestTypeDict,
    requestStatusDict
}: SubAppRequestAuditActionDrawerProps) {
    const { success: showSuccess, error: showError } = useToast()
    const [auditType, setAuditType] = useState<'approve' | 'reject'>('approve')
    const [note, setNote] = useState('')

    // 封装审核 mutation
    const auditMutation = useMutation({
        mutationFn: async ({ request, confirmStatus, confirmNote }: {
            request: AppSubRequestItemType
            confirmStatus: number
            confirmNote: string
        }) => {
            return await performAudit(request, confirmStatus, confirmNote)
        },
        onSuccess: (result, variables) => {
            if (result.status) {
                const isApprove = variables.confirmStatus === 2
                showSuccess(isApprove ? '审核通过' : '已驳回申请')
                setNote('')
                setAuditType('approve')
                onOpenChange(false)
                onAuditComplete?.()
            } else {
                showError(formatServerError(result))
            }
        },
        onError: (error: any) => {
            showError(formatServerError(error))
        }
    })

    const handleConfirm = async () => {
        if (!request) return

        // 如果是拒绝且没有填写备注，提示用户
        if (auditType === 'reject' && !note.trim()) {
            showError('驳回时必须填写驳回原因')
            return
        }

        const confirmStatus = auditType === 'approve' ? 2 : 3 // 2=通过, 3=驳回
        const confirmNote = note.trim() || ''

        auditMutation.mutate({ request, confirmStatus, confirmNote })
    }

    const handleOpenChange = (newOpen: boolean) => {
        if (!auditMutation.isPending) {
            onOpenChange(newOpen)
            if (!newOpen) {
                setNote('')
                setAuditType('approve')
            }
        }
    }

    const isApprove = auditType === 'approve'

    return (
        <Drawer open={open} onOpenChange={handleOpenChange}>
            <DrawerContent>
                <DrawerHeader className="mb-6">
                    <DrawerTitle>审核子应用申请</DrawerTitle>
                    <DrawerDescription>
                        请仔细核对申请信息后进行审核
                    </DrawerDescription>
                </DrawerHeader>

                {/* 申请信息展示 - 可滚动区域 */}
                <div className="space-y-6 pb-6">
                    {request && (
                        <>
                            {/* 申请信息 */}
                            <div className="space-y-4">
                                <h3 className="font-semibold text-base">申请信息</h3>
                                <div className="grid gap-3 text-sm">
                                    {/* 申请类型 */}
                                    <div className="flex items-start gap-2">
                                        <span className="text-muted-foreground min-w-[80px]">申请类型:</span>
                                        <Badge variant="outline" className={cn("text-xs")}>
                                            {requestTypeDict?.getLabel(String(request.request_type)) || `类型 ${request.request_type}`}
                                        </Badge>
                                    </div>

                                    {/* 应用信息 */}
                                    <Separator className="my-2" />
                                    <div className="flex items-start gap-2">
                                        <span className="text-muted-foreground min-w-[80px]">申请ID:</span>
                                        <span className="font-mono text-xs">{request.id}</span>
                                    </div>

                                    <div className="flex items-start gap-2">
                                        <span className="text-muted-foreground min-w-[80px]">应用ID:</span>
                                        <span className="font-mono text-xs">{request.app_id}</span>
                                    </div>

                                    {/* 请求用户 */}
                                    <Separator className="my-2" />
                                    {request.user_data && (
                                        <>
                                            <div className="flex items-start gap-2">
                                                <span className="text-muted-foreground min-w-[80px]">用户ID:</span>
                                                <span className="font-mono text-xs">{request.user_data.id}</span>
                                            </div>
                                            <div className="flex items-start gap-2">
                                                <span className="text-muted-foreground min-w-[80px]">用户类型:</span>
                                                <span className="text-xs">{request.user_data.app_id === 0 ? "系统用户" : "应用用户"}</span>
                                            </div>
                                            <div className="flex items-start gap-2">
                                                <span className="text-muted-foreground min-w-[80px]">用户昵称:</span>
                                                <span className="text-xs">{request.user_data.user_nickname}</span>
                                            </div>
                                            <div className="flex items-start gap-2">
                                                <span className="text-muted-foreground min-w-[80px]">用户账号:</span>
                                                <span className="text-xs">{request.user_data.user_account}</span>
                                            </div>
                                        </>
                                    )}

                                    {/* 请求时间 */}
                                    <div className="flex items-start gap-2">
                                        <span className="text-muted-foreground min-w-[80px]">请求时间:</span>
                                        <span className="text-xs">
                                            {formatTime(request.request_time, TIME_STYLE.RELATIVE_ELEMENT)}
                                        </span>
                                    </div>

                                    {/* 请求数据 */}
                                    <Separator className="my-2" />
                                    <div className="space-y-2">
                                        <span className="text-muted-foreground">请求数据:</span>
                                        <div>
                                            <SubAppRequestDataDisplay data={request} mode="drawer" showLabel={true} />
                                        </div>
                                    </div>
                                </div>
                            </div>

                            {/* 审核操作 */}
                            <Separator />
                            <div className="space-y-5">
                                <h3 className="font-semibold text-base">审核操作</h3>

                                {/* 审核类型选择 */}
                                <div className="flex items-center gap-6">
                                    <Label className="min-w-[80px]">审核结果</Label>
                                    <RadioGroup
                                        value={auditType}
                                        onValueChange={(value) => setAuditType(value as 'approve' | 'reject')}
                                        disabled={auditMutation.isPending}
                                        className="flex flex-row items-center gap-4"
                                    >
                                        <div className={cn(
                                            "flex items-center gap-3 px-4 py-2 rounded-lg transition-all cursor-pointer",
                                            "bg-muted/30 hover:bg-muted/50",
                                            auditType === 'approve' && "bg-primary/10 hover:bg-primary/15",
                                            auditType === 'reject' && "bg-destructive/10 hover:bg-destructive/15",
                                            auditMutation.isPending && "opacity-50 cursor-not-allowed"
                                        )}>
                                            <RadioGroupItem value="approve" id="approve" disabled={auditMutation.isPending} className={cn("h-4 w-4")} />
                                            <Label htmlFor="approve" className={cn("font-normal cursor-pointer", auditMutation.isPending && "cursor-not-allowed")}>
                                                通过
                                            </Label>
                                        </div>
                                        <div className={cn(
                                            "flex items-center gap-3 px-4 py-2 rounded-lg transition-all cursor-pointer",
                                            "bg-muted/30 hover:bg-muted/50",
                                            auditType === 'reject' && "bg-destructive/10 hover:bg-destructive/15",
                                            auditType === 'approve' && "bg-primary/10 hover:bg-primary/15",
                                            auditMutation.isPending && "opacity-50 cursor-not-allowed"
                                        )}>
                                            <RadioGroupItem value="reject" id="reject" disabled={auditMutation.isPending} className={cn("h-4 w-4")} />
                                            <Label htmlFor="reject" className={cn("font-normal cursor-pointer", auditMutation.isPending && "cursor-not-allowed")}>
                                                驳回
                                            </Label>
                                        </div>
                                    </RadioGroup>
                                </div>

                                {/* 备注输入 */}
                                <div className="space-y-3">
                                    <Label htmlFor="audit-note">
                                        {isApprove ? '审核备注（可选）' : '驳回原因'}
                                        {!isApprove && <span className="text-red-500 ml-1">*</span>}
                                    </Label>
                                    <Textarea
                                        id="audit-note"
                                        placeholder={isApprove ? '请输入审核备注...' : '请输入驳回原因...'}
                                        value={note}
                                        onChange={(e) => setNote(e.target.value)}
                                        disabled={auditMutation.isPending}
                                        className="mt-4 resize-none min-h-24"
                                    />
                                </div>

                                {/* 操作按钮 */}
                                <div className="flex justify-end gap-2 pt-2">
                                    <Button
                                        variant="outline"
                                        onClick={() => handleOpenChange(false)}
                                        disabled={auditMutation.isPending}
                                    >
                                        取消
                                    </Button>
                                    <Button
                                        onClick={handleConfirm}
                                        disabled={auditMutation.isPending || (!isApprove && !note.trim())}
                                    >
                                        {auditMutation.isPending && <Loader2 className={cn("mr-2 h-4 w-4 animate-spin")} />}
                                        {isApprove ? '通过' : '驳回'}
                                    </Button>
                                </div>
                            </div>
                        </>
                    )}
                </div>
            </DrawerContent>
        </Drawer>
    )
}
