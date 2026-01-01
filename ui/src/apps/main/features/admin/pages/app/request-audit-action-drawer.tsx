import {
    appConfirm,
    confirmExterFeature,
    confirmInnerFeatureExterLoginConfirm,
    confirmInnerFeatureSubAppConfirm,
    oauthClientConfirm,
    oauthClientScopeConfirm,
    oauthServerConfirm,
    type AppRequestItemType
} from '@shared/apis/admin/app'
import { UserDataTooltip } from '@apps/main/components/local/user-data-tooltip'
import CopyableText from '@shared/components/custom/text/copyable-text'
import { Badge } from '@shared/components/ui/badge'
import { Button } from '@shared/components/ui/button'
import { Label } from '@shared/components/ui/label'
import { RadioGroup, RadioGroupItem } from '@shared/components/ui/radio-group'
import { Separator } from '@shared/components/ui/separator'
import { Drawer, DrawerContent, DrawerDescription, DrawerHeader, DrawerTitle } from '@apps/main/components/local/drawer'
import { Textarea } from '@shared/components/ui/textarea'
import { useToast } from '@shared/contexts/toast-context'
import { cn, formatServerError, formatTime, TIME_STYLE } from '@shared/lib/utils'
import { DictList } from '@shared/types/apis-dict'
import { useMutation } from '@tanstack/react-query'
import { Loader2 } from 'lucide-react'
import { useState } from 'react'
import { AppRequestDataDisplay } from '../../components/ui/app-request-data-display'

interface RequestAuditActionDrawerProps {
    /** 申请数据 */
    request: AppRequestItemType | null
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

export function RequestAuditActionDrawer({
    request,
    open,
    onOpenChange,
    onAuditComplete,
    requestTypeDict,
    requestStatusDict
}: RequestAuditActionDrawerProps) {
    const { success: showSuccess, error: showError } = useToast()
    const [auditType, setAuditType] = useState<'approve' | 'reject'>('approve')
    const [note, setNote] = useState('')

    // 根据申请类型调用对应的审核接口
    const performAudit = async (request: AppRequestItemType, confirmStatus: number, confirmNote: string) => {
        let result

        // 申请类型映射:
        // 1: 新应用申请 -> /api/system/app/confirm
        // 2: 应用更改申请 -> /api/system/app/confirm
        // 3: 子应用可用申请 -> /api/system/app/confirm_inner_feature_sub_app_confirm
        // 4: 外部账号登录系统申请 -> /api/system/app/confirm_inner_feature_exter_login_confirm
        // 5: Oauth服务申请 -> /api/system/app/oauth_server_confirm
        // 6: Oauth登录申请 -> /api/system/app/oauth_client_confirm
        // 7: OAUTH登录新增权限申请 -> /api/system/app/oauth_client_scope_confirm
        // 8: 外部功能申请 -> /api/system/app/confirm_exter_feature

        switch (request.request_type) {
            case 1: // 新应用申请
            case 2: // 应用更改申请
                result = await appConfirm({
                    app_req_id: request.id,
                    confirm_status: confirmStatus,
                    confirm_note: confirmNote,
                })
                break
            case 3: // 子应用可用申请
                result = await confirmInnerFeatureSubAppConfirm({
                    app_id: request.app_id,
                    confirm_status: confirmStatus,
                    confirm_note: confirmNote,
                })
                break
            case 4: // 外部账号登录系统申请
                result = await confirmInnerFeatureExterLoginConfirm({
                    app_id: request.app_id,
                    confirm_status: confirmStatus,
                    confirm_note: confirmNote,
                })
                break
            case 5: // Oauth服务申请
                result = await oauthServerConfirm({
                    app_id: request.app_id,
                    confirm_status: confirmStatus,
                    confirm_note: confirmNote,
                })
                break
            case 6: // Oauth登录申请
                result = await oauthClientConfirm({
                    app_id: request.app_id,
                    confirm_status: confirmStatus,
                    confirm_note: confirmNote,
                })
                break
            case 7: // OAUTH登录新增权限申请
                result = await oauthClientScopeConfirm({
                    app_req_id: request.id,
                    confirm_status: confirmStatus,
                    confirm_note: confirmNote,
                })
                break
            case 8: // 外部功能申请
                result = await confirmExterFeature({
                    app_req_id: request.id,
                    confirm_status: confirmStatus,
                    confirm_note: confirmNote,
                })
                break
            default:
                throw new Error(`未知的申请类型: ${request.request_type}`)
        }

        return result
    }

    const auditMutation = useMutation({
        mutationFn: async ({ confirmStatus, confirmNote }: { confirmStatus: number; confirmNote: string }) => {
            if (!request) throw new Error('请求数据为空')
            return performAudit(request, confirmStatus, confirmNote)
        },
        onSuccess: () => {
            showSuccess(auditType === 'approve' ? '审核通过' : '已驳回申请')
            setNote('')
            setAuditType('approve')
            onOpenChange(false)
            onAuditComplete?.()
        },
        onError: (error) => {
            showError(formatServerError(error))
        }
    })

    const handleOpenChange = (newOpen: boolean) => {
        if (!auditMutation.isPending) {
            onOpenChange(newOpen)
            if (!newOpen) {
                setNote('') // 关闭时重置备注
                setAuditType('approve') // 重置为通过
            }
        }
    }

    const isApprove = auditType === 'approve'

    return (
        <Drawer open={open} onOpenChange={handleOpenChange}>
            <DrawerContent>
                <DrawerHeader className={cn("mb-6")}>
                    <DrawerTitle>审核申请</DrawerTitle>
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
                                        <span className="text-muted-foreground min-w-[80px]">应用ID:</span>
                                        <span className="font-mono text-xs">{request.app_id}</span>
                                    </div>

                                    {/* 应用名称和标识放在一行 */}
                                    <div className="flex items-start gap-4">
                                        {request.app_name && (
                                            <div className="flex items-start gap-2 flex-1">
                                                <span className="text-muted-foreground min-w-[80px]">应用名称:</span>
                                                <span className="font-medium">{request.app_name}</span>
                                            </div>
                                        )}
                                        {request.app_client && (
                                            <div className="flex items-start gap-2 flex-1">
                                                <span className="text-muted-foreground min-w-[80px]">应用标识:</span>
                                                <CopyableText value={request.app_client} className="text-xs" />
                                            </div>
                                        )}
                                    </div>

                                    {/* 请求用户 */}
                                    <Separator className="my-2" />
                                    <div className="flex items-start gap-2">
                                        <span className="text-muted-foreground min-w-[80px]">请求用户:</span>
                                        <UserDataTooltip userData={request.request_user_data} className="text-xs" />
                                    </div>

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
                                            <AppRequestDataDisplay data={request} mode="drawer" />
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
                                            <RadioGroupItem value="approve" id="approve" disabled={auditMutation.isPending} className="h-4 w-4" />
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
                                            <RadioGroupItem value="reject" id="reject" disabled={auditMutation.isPending} className="h-4 w-4" />
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
                                        onClick={() => {
                                            if (auditType === 'reject' && !note.trim()) {
                                                showError('驳回时必须填写驳回原因')
                                                return
                                            }
                                            const confirmStatus = auditType === 'approve' ? 2 : 3
                                            auditMutation.mutate({ confirmStatus, confirmNote: note.trim() || '' })
                                        }}
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
