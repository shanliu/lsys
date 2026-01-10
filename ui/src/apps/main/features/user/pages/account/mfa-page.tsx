import { mfaStatus, mfaBindQrcode, mfaBindDevice, mfaUnbind } from '@shared/apis/user/account';
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error';
import { ContentDialog } from '@shared/components/custom/dialog/content-dialog';
import { ConfirmDialog } from '@shared/components/custom/dialog/confirm-dialog';
import { Badge } from '@shared/components/ui/badge';
import { Button } from '@shared/components/ui/button';
import { Input } from '@shared/components/ui/input';
import { useToast } from '@shared/contexts/toast-context';
import { cn } from '@shared/lib/utils';
import { formatServerError } from '@shared/lib/utils/format-utils';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Shield, ShieldOff, Plus, Trash2 } from 'lucide-react';
import React, { useState } from 'react';
import QRCode from 'qrcode.react';
import { mfaBindFormSchema, type MfaBindFormType } from './mfa-schema';
import { CenteredLoading } from '@/shared/components/custom/page-placeholder/centered-loading';

const MfaPageContent: React.FC = () => {
    const { success, error } = useToast();
    const queryClient = useQueryClient();

    const [formData, setFormData] = React.useState<MfaBindFormType>({ code: '' });
    const [bindStep, setBindStep] = React.useState<'qrcode' | 'verify'>('qrcode');
    const [isBindDialogOpen, setIsBindDialogOpen] = useState(false);
    const [codeDigits, setCodeDigits] = React.useState<string[]>([]);

    const resetVerifyInput = React.useCallback(() => {
        setFormData({ code: '' });
        setCodeDigits([]);
    }, []);

    React.useEffect(() => {
        if (!isBindDialogOpen) {
            setFormData({ code: '' });
            setBindStep('qrcode');
            setCodeDigits([]);
            queryClient.removeQueries({ queryKey: ['mfaBindQrcode'] });
        }
    }, [isBindDialogOpen, queryClient]);

    // 获取MFA状态
    const { data: statusData, isLoading, error: queryError } = useQuery({
        queryKey: ['mfaStatus'],
        queryFn: () => mfaStatus()
    });

    const mfaEnabled = statusData?.response?.enabled ?? false;

    // 获取二维码数据
    const { data: qrcodeData, isLoading: qrcodeLoading, error: qrcodeError, refetch: refetchQrcode } = useQuery({
        queryKey: ['mfaBindQrcode'],
        queryFn: () => mfaBindQrcode(),
        enabled: isBindDialogOpen,
        staleTime: 0,
        gcTime: 0,
        refetchOnMount: 'always',
    });

    const qrcodeResponse = qrcodeData?.response;

    React.useEffect(() => {
        resetVerifyInput();
    }, [qrcodeResponse?.secret, resetVerifyInput]);

    React.useEffect(() => {
        if (bindStep === 'qrcode') {
            resetVerifyInput();
        }
    }, [bindStep, resetVerifyInput]);

    // 绑定MFA设备
    const bindMutation = useMutation({
        mutationFn: (data: { secret: string; code: string }) => mfaBindDevice(data),
        onSuccess: () => {
            success('MFA设备绑定成功');
            setIsBindDialogOpen(false);
            queryClient.invalidateQueries({ queryKey: ['mfaStatus'] });
        },
        onError: (err: any) => {
            const errorMessage = formatServerError(err);
            error(errorMessage);
        },
    });

    // 解绑MFA设备
    const unbindMutation = useMutation({
        mutationFn: () => mfaUnbind(),
        onSuccess: () => {
            success('MFA设备已解绑');
            queryClient.invalidateQueries({ queryKey: ['mfaStatus'] });
        },
        onError: (err: any) => {
            const errorMessage = formatServerError(err);
            error(errorMessage);
        },
    });

    // 处理验证码数字输入
    const handleCodeDigitChange = (index: number, value: string) => {
        if (value && !/^\d$/.test(value)) return; // 只允许单个数字
        const newDigits = [...codeDigits];
        newDigits[index] = value;
        setCodeDigits(newDigits);

        // 自动移动到下一个输入框
        if (value && index < (qrcodeResponse?.len || 6) - 1) {
            const nextInput = document.getElementById(`code-${index + 1}`);
            nextInput?.focus();
        }

        // 更新formData中的code
        setFormData({ code: newDigits.join('') });
    };

    // 处理退格键
    const handleCodeKeyDown = (index: number, e: React.KeyboardEvent) => {
        if (e.key === 'Backspace' && !codeDigits[index] && index > 0) {
            const prevInput = document.getElementById(`code-${index - 1}`);
            prevInput?.focus();
        }
    };

    // 处理绑定提交
    const handleBindSubmit = () => {
        const result = mfaBindFormSchema.safeParse(formData);
        if (!result.success) {
            const firstError = result.error.errors[0];
            error(firstError.message);
            return;
        }

        if (!qrcodeResponse?.secret) {
            error('获取Secret失败，请重试');
            return;
        }

        // 验证验证码长度
        if (qrcodeResponse.len && result.data.code.length !== qrcodeResponse.len) {
            error(`验证码长度应为${qrcodeResponse.len}位`);
            return;
        }

        bindMutation.mutate({
            secret: qrcodeResponse.secret,
            code: result.data.code,
        });
    };

    // 绑定流程内容
    const bindContent = (
        <div className="space-y-4">
            {bindStep === 'qrcode' && (
                <>
                    {/* 二维码区域 */}
                    <div className="space-y-4">
                        <span className="text-sm font-medium block">扫描二维码</span>
                        <div
                            className={cn(
                                "flex justify-center items-center border rounded-lg p-2 aspect-square cursor-pointer transition-colors",
                                qrcodeLoading ? "bg-background hover:bg-muted/50" : "bg-white hover:bg-gray-50"
                            )}
                            onClick={() => {
                                resetVerifyInput();
                                refetchQrcode();
                            }}
                        >
                            {qrcodeLoading && (
                                <CenteredLoading iconSize="sm" />
                            )}
                            {!qrcodeLoading && qrcodeError && (
                                <div className="text-center text-red-500 text-xs">
                                    <p>获取二维码失败</p>
                                    <p className="text-xs mt-1">{formatServerError(qrcodeError)}</p>
                                </div>
                            )}
                            {!qrcodeLoading && !qrcodeError && qrcodeResponse?.otpauth_url && (
                                <QRCode
                                    value={qrcodeResponse.otpauth_url}
                                    size={200}
                                    level="H"
                                    includeMargin={false}
                                />
                            )}
                            {!qrcodeLoading && !qrcodeError && !qrcodeResponse?.otpauth_url && (
                                <div className="text-center text-red-500 text-xs">
                                    <p>无法获取二维码数据</p>
                                </div>
                            )}
                        </div>

                        {/* 提示信息 */}
                        <div className="p-2 border rounded-lg text-[0.75rem] text-muted-foreground text-center">
                            <p>使用 Microsoft Authenticator 等验证器应用扫描二维码</p>
                        </div>
                    </div>
                </>
            )}

            {bindStep === 'verify' && (
                <>
                    {/* 验证码输入 */}
                    <div className="space-y-4">
                        <div className="text-center space-y-1">
                            <p className="text-sm font-medium">输入验证码</p>
                            <p className="text-xs text-muted-foreground">
                                请输入验证器应用中显示的 {qrcodeResponse?.len || 6} 位数字验证码
                            </p>
                        </div>
                        <div className="flex justify-center gap-2 w-full">
                            {Array.from({ length: qrcodeResponse?.len || 6 }).map((_, index) => (
                                <Input
                                    key={index}
                                    id={`code-${index}`}
                                    type="text"
                                    inputMode="numeric"
                                    placeholder=""
                                    value={codeDigits[index] || ''}
                                    onChange={(e) => handleCodeDigitChange(index, e.target.value)}
                                    onKeyDown={(e) => handleCodeKeyDown(index, e)}
                                    className="w-10 h-12 text-center text-lg font-semibold p-0 flex-shrink-0"
                                    maxLength={1}
                                    autoFocus={index === 0}
                                />
                            ))}
                        </div>
                    </div>
                </>
            )}
        </div>
    );

    if (queryError) {
        return <CenteredError error={queryError} className={cn("m-4 md:m-6")} />;
    }

    if (isLoading) {
        return <CenteredLoading variant="card" className={cn('m-4 md:m-6')} />;
    }
    return (
        <div className="px-0 py-6 space-y-6">
            {/* 页面标题 */}
            <div className="px-4 md:px-6">
                <h1 className="text-lg md:text-2xl font-semibold text-foreground">双因素认证</h1>
            </div>

            {/* 双因素认证卡片 */}
            <div className="px-4 md:px-6">
                <div className="rounded-lg border p-6">
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-4">
                            {mfaEnabled ? (
                                <Shield className="w-8 h-8" />
                            ) : (
                                <ShieldOff className="w-8 h-8" />
                            )}

                            <div className="space-y-1">
                                <h3 className="font-semibold text-lg">
                                    双因素认证（MFA）
                                </h3>
                                <p className="text-sm">
                                    {mfaEnabled
                                        ? "您已启用双因素认证"
                                        : "未启用双因素认证"
                                    }
                                </p>
                            </div>
                        </div>

                        <Badge variant={mfaEnabled ? "default" : "secondary"}>
                            {mfaEnabled ? "已启用" : "未启用"}
                        </Badge>
                    </div>

                    {/* 操作按钮 */}
                    <div className="mt-6 flex gap-2">
                        {!mfaEnabled ? (
                            <ContentDialog
                                title="绑定MFA设备"
                                content={bindContent}
                                open={isBindDialogOpen}
                                onOpenChange={setIsBindDialogOpen}
                                className={bindStep === 'verify' ? "!w-[360px]" : "!w-[280px]"}
                                footer={(closeDialog) => (
                                    <div className="flex gap-2 justify-end">
                                        <Button
                                            variant="outline"
                                            onClick={() => {
                                                if (bindStep === 'verify') {
                                                    setBindStep('qrcode');
                                                    resetVerifyInput();
                                                } else {
                                                    closeDialog();
                                                }
                                            }}
                                        >
                                            {bindStep === 'verify' ? '返回' : '取消'}
                                        </Button>
                                        {bindStep === 'qrcode' ? (
                                            <Button
                                                onClick={() => {
                                                    resetVerifyInput();
                                                    setBindStep('verify');
                                                }}
                                                disabled={qrcodeLoading}
                                            >
                                                下一步
                                            </Button>
                                        ) : (
                                            <Button
                                                onClick={handleBindSubmit}
                                                disabled={bindMutation.isPending}
                                            >
                                                {bindMutation.isPending ? '绑定中...' : '完成绑定'}
                                            </Button>
                                        )}
                                    </div>
                                )}
                            >
                                <Button variant="outline">
                                    <Plus className="w-4 h-4 mr-2" />
                                    绑定MFA设备
                                </Button>
                            </ContentDialog>
                        ) : (
                            <ConfirmDialog
                                title="确认解绑MFA设备"
                                description="解绑后您需要重新绑定新设备才能继续使用双因素认证。"
                                onConfirm={async () => {
                                    await unbindMutation.mutateAsync();
                                }}
                            >
                                <Button
                                    variant="outline"
                                    className="gap-2 border-destructive text-destructive hover:bg-destructive/10 hover:text-destructive"
                                >
                                    <Trash2 size={16} />
                                    解绑MFA设备
                                </Button>
                            </ConfirmDialog>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
};

const MfaPage: React.FC = () => {
    return <MfaPageContent />;
};

export { MfaPage };
