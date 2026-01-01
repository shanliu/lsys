import { Drawer, DrawerContent, DrawerDescription, DrawerHeader, DrawerTitle } from '@apps/main/components/local/drawer';
import { externalBindCheck, externalBindUrl } from '@shared/apis/user/profile';
import { Button } from '@shared/components/ui/button';
import { useToast } from '@shared/contexts/toast-context';
import { cn } from '@shared/lib/utils';
import { formatServerError } from '@shared/lib/utils/format-utils';
import { generateRandomString } from '@shared/lib/utils';
import { DictItemType } from '@shared/types/apis-dict';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useRouter } from '@tanstack/react-router';
import { Loader2 } from 'lucide-react';
import React, { useCallback, useEffect, useRef, useState } from 'react';

interface ExternalBindDrawerProps {
    isOpen: boolean;
    onClose: () => void;
    availableTypes: DictItemType[];
}

export const ExternalBindDrawer: React.FC<ExternalBindDrawerProps> = ({
    isOpen,
    onClose,
    availableTypes,
}) => {
    const { error, success } = useToast();
    const queryClient = useQueryClient();
    const router = useRouter();

    // 状态
    const [pollingState, setPollingState] = useState<{
        active: boolean;
        type: string;
        typeName: string;
        loginState: string;
        countdown: number;
    } | null>(null);

    const pollingRef = useRef<NodeJS.Timeout>(null);

    // 停止轮询
    const stopPolling = useCallback(() => {
        if (pollingRef.current) {
            clearInterval(pollingRef.current);
            pollingRef.current = null;
        }
        setPollingState(prev => {
            if (prev?.active) {
                return null;
            }
            return prev;
        });
    }, []);

    // 保存预打开的窗口引用
    const popupWindowRef = useRef<Window | null>(null);

    // 获取绑定URL并跳转
    const bindMutation = useMutation({
        mutationFn: async (item: DictItemType) => {
            const loginState = generateRandomString(32);
            // 使用 router 生成带 basepath 的路径，包含 state 和 login_type 参数
            const oauthNotifyPath = router.buildLocation({
                to: '/oauth-qrcode',
                search: { state: loginState, login_type: item.key },
            }).href;
            const callbackUrl = `${window.location.origin}${oauthNotifyPath}`;

            const res = await externalBindUrl({
                login_type: item.key,
                login_state: loginState,
                callback_url: callbackUrl,
            });
            return { res, loginState, item };
        },
        onSuccess: ({ res, loginState, item }) => {
            console.log('绑定URL响应:', res);
            if (res.status && res.response?.url) {
                const bindUrl = res.response.url;
                console.log('准备打开授权页面:', bindUrl);

                // 如果有预打开的窗口，更新其 URL
                if (popupWindowRef.current && !popupWindowRef.current.closed) {
                    popupWindowRef.current.location.href = bindUrl;
                } else {
                    // 如果预打开窗口被关闭或失败，尝试直接打开
                    // 这在某些情况下可能被阻止，但作为后备方案
                    const popup = window.open(bindUrl, '_blank', 'noopener,noreferrer');
                    if (!popup) {
                        // 如果打开新窗口失败（被拦截），提示用户
                        error('无法打开授权窗口，请允许弹出窗口后重试');
                        return;
                    }
                }

                // 清理预打开窗口引用
                popupWindowRef.current = null;

                // 开始轮询
                setPollingState({
                    active: true,
                    type: item.key,
                    typeName: item.val,
                    loginState,
                    countdown: 60, // 60秒倒计时
                });
            } else {
                // 关闭预打开的窗口
                if (popupWindowRef.current && !popupWindowRef.current.closed) {
                    popupWindowRef.current.close();
                }
                popupWindowRef.current = null;
                error(res.message || '获取授权URL失败');
            }
        },
        onError: (err: any) => {
            console.error('绑定URL请求失败:', err);
            // 关闭预打开的窗口
            if (popupWindowRef.current && !popupWindowRef.current.closed) {
                popupWindowRef.current.close();
            }
            popupWindowRef.current = null;
            const errorMessage = formatServerError(err);
            error(errorMessage);
        },
    });

    // 轮询检查
    const checkMutation = useMutation({
        mutationFn: async () => {
            if (!pollingState) return;
            return externalBindCheck({
                login_type: pollingState.type,
                login_state: pollingState.loginState,
            });
        },
        onSuccess: (res) => {
            if (res?.response?.id) {
                // 绑定成功
                success("绑定成功");
                stopPolling();
                onClose();
                queryClient.invalidateQueries({ queryKey: ['externalList'] });
            } else if (res?.response?.reload === "1") {
                // 继续轮询，不做操作
            } else {
                // 其他情况视为失败或结束
                stopPolling();
                error("绑定失败，请重试");
            }
        },
        onError: () => {
            // 忽略网络错误，继续尝试或者由倒计时结束
        }
    });

    // 将 checkMutation.mutate 提取为 ref，避免依赖变化导致重新创建定时器
    const checkMutateRef = useRef(checkMutation.mutate);
    checkMutateRef.current = checkMutation.mutate;

    // 启动/停止定时器
    useEffect(() => {
        if (pollingState?.active) {
            pollingRef.current = setInterval(() => {
                setPollingState(prev => {
                    if (!prev || prev.countdown <= 0) {
                        stopPolling();
                        return null;
                    }
                    // 每秒检查一次
                    if (prev.countdown % 2 === 0) { // 每2秒调一次接口
                        checkMutateRef.current();
                    }
                    return { ...prev, countdown: prev.countdown - 1 };
                });
            }, 1000);
        } else {
            stopPolling();
        }

        return () => stopPolling();
    }, [pollingState?.active, stopPolling]);

    // 关闭时重置
    useEffect(() => {
        if (!isOpen) {
            stopPolling();
            // 关闭预打开的窗口
            if (popupWindowRef.current && !popupWindowRef.current.closed) {
                popupWindowRef.current.close();
            }
            popupWindowRef.current = null;
        }
    }, [isOpen, stopPolling]);


    const handleBindClick = (item: DictItemType) => {
        // 在所有平台上，都需要在用户点击事件的同步代码中预打开窗口
        // 否则浏览器（特别是 iOS Safari）会阻止弹出窗口
        const popup = window.open('about:blank', '_blank');
        if (popup) {
            popupWindowRef.current = popup;
            // 显示加载提示
            popup.document.write(`
                <!DOCTYPE html>
                <html>
                <head>
                    <meta charset="utf-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1">
                    <title>正在跳转...</title>
                    <style>
                        body { 
                            display: flex; 
                            justify-content: center; 
                            align-items: center; 
                            height: 100vh; 
                            margin: 0; 
                            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                            background: #f5f5f5;
                            color: #666;
                        }
                        @media (prefers-color-scheme: dark) {
                            body { background: #1a1a1a; color: #aaa; }
                            .spinner { border-color: #333; border-top-color: #3b82f6; }
                        }
                        .loading { text-align: center; }
                        .spinner {
                            width: 40px;
                            height: 40px;
                            border: 3px solid #e0e0e0;
                            border-top: 3px solid #3b82f6;
                            border-radius: 50%;
                            animation: spin 1s linear infinite;
                            margin: 0 auto 16px;
                        }
                        @keyframes spin { to { transform: rotate(360deg); } }
                    </style>
                </head>
                <body>
                    <div class="loading">
                        <div class="spinner"></div>
                        <div>正在跳转到授权页面...</div>
                    </div>
                </body>
                </html>
            `);
        }
        bindMutation.mutate(item);
    };

    return (
        <Drawer open={isOpen} onOpenChange={(open) => !open && onClose()}>
            <DrawerContent className={cn("bg-background text-foreground border-border ")}>
                <DrawerHeader>
                    <DrawerTitle className={cn("text-foreground")}>
                        绑定外部账号
                    </DrawerTitle>
                    <DrawerDescription className={cn("text-muted-foreground")}>
                        选择需要绑定的外部账号类型进行绑定
                    </DrawerDescription>
                </DrawerHeader>

                <div className="space-y-6 mt-6 overflow-y-auto flex-1">
                    {availableTypes.length > 0 ? (
                        <div className="grid gap-4">
                            {availableTypes.map((item) => {
                                const isCurrentPolling = pollingState?.type === item.key;
                                const isBusy = bindMutation.isPending || (pollingState?.active && !isCurrentPolling);

                                return (
                                    <Button
                                        key={item.key}
                                        variant="outline"
                                        className="w-full justify-start h-12 text-lg"
                                        onClick={() => handleBindClick(item)}
                                        disabled={isBusy || isCurrentPolling}
                                    >
                                        {isCurrentPolling ? (
                                            <>
                                                <Loader2 className=" h-4 w-4 animate-spin" />
                                                <span className="ml-2">检查 {item.val} 授权中 ({pollingState?.countdown}s)</span>
                                            </>
                                        ) : bindMutation.isPending && bindMutation.variables?.key === item.key ? (
                                            <Loader2 className=" h-4 w-4 animate-spin" />
                                        ) : null}
                                        {!isCurrentPolling && <span className='ml-2'>{`绑定 ${item.val}`}</span>}
                                    </Button>
                                );
                            })}
                        </div>
                    ) : (
                        <div className="text-center py-8 text-muted-foreground">
                            暂无支持的外部账号类型
                        </div>
                    )}
                </div>
            </DrawerContent>
        </Drawer>
    );
};
