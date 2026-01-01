import { ContentDialog } from "@shared/components/custom/dialog/content-dialog";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import { cn } from "@shared/lib/utils";
import type { ApiResult } from "@shared/types/apis-rest";
import { AlertTriangle, ChevronDown, Info, RefreshCw, ShieldAlert } from "lucide-react";
import { ReactNode, useState } from "react";
import { PageSkeletonBaseProps } from "./base";

/**
 * 服务器错误类型（支持 ApiResult 和 Axios 错误格式）
 */
interface ServerError {
    // ApiResult 格式（直接传入）
    code?: string;
    state?: string;
    status?: boolean;
    message?: string;
    response?: any;
    // Axios 错误格式（包含 ApiResult 在 data 中）
    data?: ApiResult;
}

export interface CenteredErrorProps extends PageSkeletonBaseProps {
    /** 
     * 错误展示变体
     * - 'content': 无边框模式（仅居中错误内容）
     * - 'card': 带卡片容器的错误提示
     * - 'page': 上下两栏布局（上面过滤块-小，下面内容块-大，每块中间提示）
     * - 'layout': 左右两栏布局（左侧边栏，右侧上下两栏，每块中间提示）
     * - 'inline': 行内模式（图标、消息、操作按钮排成一行）
     */
    variant?: 'content' | 'card' | 'page' | 'layout' | 'inline';
    /** 错误对象（支持多种类型：Error、ServerError、string、unknown） */
    error: unknown;
    /** 子元素（在 layout 模式下显示在右侧内容区域的下方） */
    children?: ReactNode;
    /** 重置回调函数，用于刷新错误状态 */
    onReset?: () => void;
}


/**
 * 判断是否为 ApiResult 对象
 */
function isApiResult(error: unknown): error is ApiResult {
    if (!error || typeof error !== 'object') return false;
    const err = error as any;
    // ApiResult 的特征：code, state, status, message
    return (
        typeof err.code === 'string' &&
        typeof err.state === 'string' &&
        typeof err.status === 'boolean' &&
        typeof err.message === 'string'
    );
}

/**
 * 判断是否为服务器错误对象（包含 ApiResult 在 data 中）
 */
function isServerError(error: unknown): error is ServerError {
    if (!error || typeof error !== 'object') return false;
    const err = error as any;

    // 1. 直接是 ApiResult
    if (isApiResult(err)) {
        return true;
    }

    // 2. Axios 错误格式，data 中包含 ApiResult
    if (err.data && isApiResult(err.data)) {
        return true;
    }

    return false;
}

/**
 * 判断是否为标准 Error 对象
 */
function isErrorInstance(error: unknown): error is Error {
    return error instanceof Error;
}

/**
 * 提取权限检查失败的详细信息
 */
function extractCheckDetails(error: ServerError): string[] {
    const checkDetails: string[] = [];

    // 尝试从 response.check_detail 中提取
    const response = (error as any).response || (error.data as any)?.response;
    if (response && response.check_detail && typeof response.check_detail === 'object') {
        // check_detail 是一个对象，每个键对应一个数组
        Object.values(response.check_detail).forEach((value) => {
            if (Array.isArray(value)) {
                checkDetails.push(...value.filter(v => typeof v === 'string'));
            }
        });
    }

    return checkDetails;
}

/**
 * 处理服务器错误 - 行模式版本
 */
function ServerErrorRowDisplay({ error, onReset }: { error: ServerError; onReset?: () => void }): ReactNode {
    // 提取错误信息
    const apiResult = isApiResult(error) ? error : error.data;
    const message = apiResult?.message || '服务器返回了一个错误';
    const state = apiResult?.state;

    // 检查是否为权限错误
    const isPermissionError = state === 'check_fail' || state === 'not_login';
    const checkDetails = isPermissionError ? extractCheckDetails(error) : [];

    // 构建详细信息
    let details: string | undefined;
    try {
        const errorStr = JSON.stringify(apiResult, null, 2);
        if (errorStr && errorStr !== '{}') {
            details = errorStr;
        }
    } catch {
        // JSON.stringify 可能失败，忽略
    }

    return (
        <div className={cn("flex items-center gap-2 w-full")}>
            {/* 图标 */}
            <div
                className={cn(
                    "shrink-0",
                    isPermissionError
                        ? "text-yellow-500/60 dark:text-yellow-500/70"
                        : "text-destructive/60 dark:text-destructive/70"
                )}
            >
                {isPermissionError ? <ShieldAlert size={14} /> : <AlertTriangle size={14} />}
            </div>

            {/* 消息 */}
            <span className={cn("text-xs text-muted-foreground truncate flex-1")}>
                {message}
            </span>

            {/* 操作图标区域 */}
            <div className={cn("flex items-center gap-1 shrink-0")}>
                {/* 权限详情 */}
                {isPermissionError && checkDetails.length > 0 && (
                    <ContentDialog
                        title="鉴权详细信息"
                        content={
                            <div className={cn("flex flex-wrap gap-2")}>
                                {checkDetails.map((detail, index) => (
                                    <Badge
                                        key={index}
                                        variant="outline"
                                        className={cn("text-xs text-yellow-700 dark:text-yellow-300")}
                                    >
                                        {detail}
                                    </Badge>
                                ))}
                            </div>
                        }
                    >
                        <ShieldAlert size={14} className={cn("text-yellow-500 hover:text-yellow-600 cursor-pointer")} />
                    </ContentDialog>
                )}

                {/* 错误详情 */}
                {details && (
                    <ContentDialog
                        title="错误详情"
                        content={
                            <pre className={cn("text-xs bg-muted p-3 rounded-md overflow-auto max-h-96")}>
                                {details.trim()}
                            </pre>
                        }
                    >
                        <Info size={14} className={cn("text-muted-foreground hover:text-foreground cursor-pointer")} />
                    </ContentDialog>
                )}

                {/* 刷新按钮 */}
                {onReset && (
                    <RefreshCw
                        size={14}
                        onClick={onReset}
                        className={cn("text-muted-foreground hover:text-foreground cursor-pointer")}
                    />
                )}
            </div>
        </div>
    );
}

/**
 * 处理标准 Error 对象 - 行模式版本
 */
function ErrorInstanceRowDisplay({ error, onReset }: { error: Error; onReset?: () => void }): ReactNode {
    const message = error.message || '发生了一个错误';
    const details = error.stack;

    return (
        <div className={cn("flex items-center gap-2 w-full")}>
            {/* 图标 */}
            <div className={cn("shrink-0 text-destructive/60 dark:text-destructive/70")}>
                <AlertTriangle size={14} />
            </div>

            {/* 消息 */}
            <span className={cn("text-xs text-muted-foreground truncate flex-1")}>
                {message}
            </span>

            {/* 操作图标区域 */}
            <div className={cn("flex items-center gap-1 shrink-0")}>
                {/* 错误详情 */}
                {details && (
                    <ContentDialog
                        title="错误详情"
                        content={
                            <div className={cn("text-xs bg-muted rounded-md overflow-auto max-h-96")}>
                                {details.split('\n').map((line, index) => (
                                    <div
                                        key={index}
                                        className={cn(index % 2 === 0 ? 'bg-muted/50 px-3 py-0.5' : 'px-3 py-0.5')}
                                    >
                                        {line || '\u00A0'}
                                    </div>
                                ))}
                            </div>
                        }
                    >
                        <Info size={14} className={cn("text-muted-foreground hover:text-foreground cursor-pointer")} />
                    </ContentDialog>
                )}

                {/* 刷新按钮 */}
                {onReset && (
                    <RefreshCw
                        size={14}
                        onClick={onReset}
                        className={cn("text-muted-foreground hover:text-foreground cursor-pointer")}
                    />
                )}
            </div>
        </div>
    );
}

/**
 * 处理简单字符串或其他类型错误 - 行模式版本
 */
function SimpleErrorRowDisplay({ error, onReset }: { error: unknown; onReset?: () => void }): ReactNode {
    let message = '发生了未知错误';
    let details: string | undefined;

    if (typeof error === 'string') {
        message = error || '发生了未知错误';
    } else if (typeof error === 'number') {
        message = `错误代码: ${error}`;
    } else if (typeof error === 'boolean') {
        message = '操作失败';
    } else if (error === null) {
        message = '未知错误';
    } else if (error === undefined) {
        message = '未知错误';
    } else {
        try {
            const errorStr = JSON.stringify(error, null, 2);
            if (errorStr && errorStr !== '{}') {
                message = '操作失败';
                details = errorStr;
            }
        } catch {
            message = '发生了一个无法解析的错误';
        }
    }

    return (
        <div className={cn("flex items-center gap-2 w-full")}>
            {/* 图标 */}
            <div className={cn("shrink-0 text-destructive/60 dark:text-destructive/70")}>
                <AlertTriangle size={14} />
            </div>

            {/* 消息 */}
            <span className={cn("text-xs text-muted-foreground truncate flex-1")}>
                {message}
            </span>

            {/* 操作图标区域 */}
            <div className={cn("flex items-center gap-1 shrink-0")}>
                {/* 错误详情 */}
                {details && (
                    <ContentDialog
                        title="错误详情"
                        content={
                            <pre className={cn("text-xs bg-muted p-3 rounded-md overflow-auto max-h-96")}>
                                {details.trim()}
                            </pre>
                        }
                    >
                        <Info size={14} className={cn("text-muted-foreground hover:text-foreground cursor-pointer")} />
                    </ContentDialog>
                )}

                {/* 刷新按钮 */}
                {onReset && (
                    <RefreshCw
                        size={14}
                        onClick={onReset}
                        className={cn("text-muted-foreground hover:text-foreground cursor-pointer")}
                    />
                )}
            </div>
        </div>
    );
}

/**
 * 处理服务器错误 - 直接返回渲染组件
 */
function ServerErrorDisplay({ error, onReset }: { error: ServerError; onReset?: () => void }): ReactNode {
    // 使用 state 控制展开的框类型：'none' | 'perm' | 'full'
    const [openDetail, setOpenDetail] = useState<'none' | 'perm' | 'full'>('none');

    // 提取错误信息
    const apiResult = isApiResult(error) ? error : error.data;
    const message = apiResult?.message || '服务器返回了一个错误';
    const state = apiResult?.state;

    // 检查是否为权限错误
    const isPermissionError = state === 'check_fail' || state === 'not_login';
    const checkDetails = isPermissionError ? extractCheckDetails(error) : [];

    // 构建详细信息
    let details: string | undefined;
    try {
        const errorStr = JSON.stringify(apiResult, null, 2);
        if (errorStr && errorStr !== '{}') {
            details = errorStr;
        }
    } catch {
        // JSON.stringify 可能失败，忽略
    }

    return (
        <div className={cn("flex flex-col items-center justify-center gap-4 text-center")}>
            {/* 图标容器 - 根据错误类型显示不同颜色和图标 */}
            <div
                className={cn(
                    "rounded-full",
                    isPermissionError
                        ? "text-yellow-500/60 dark:text-yellow-500/70 bg-yellow-500/10 dark:bg-yellow-500/20"
                        : "text-destructive/60 dark:text-destructive/70 bg-destructive/10 dark:bg-destructive/20"
                )}
                style={{ padding: '20px' }}
            >
                {isPermissionError ? <ShieldAlert size={40} /> : <AlertTriangle size={40} />}
            </div>

            {/* 文字内容 */}
            <div className={cn("space-y-2 max-w-md")}>
                {/* 服务器错误消息 */}
                <p className={cn("text-sm text-muted-foreground text-center")}>
                    <span > {message}</span>

                </p>

                {/* 权限检查失败的详细信息 - 显示为标签 */}
                {isPermissionError && checkDetails.length > 0 && (
                    <div className={cn("mt-2")}>
                        {/* 按钮行 - 始终显示两个按钮 */}
                        <div className={cn("flex items-center justify-center gap-4")}>
                            {/* 展开鉴权详细按钮 */}
                            <Button
                                type="button"
                                variant="ghost"
                                size="sm"
                                onClick={() => setOpenDetail(openDetail === 'perm' ? 'none' : 'perm')}
                                className={cn(
                                    "cursor-pointer text-xs transition-colors flex items-center gap-1 p-0 h-auto",
                                    openDetail === 'perm'
                                        ? "text-yellow-600 dark:text-yellow-400 font-medium"
                                        : "text-muted-foreground hover:text-foreground"
                                )}
                            >
                                <span>展开鉴权详细</span>
                                <ShieldAlert className={cn("inline-block")} size={14} />
                            </Button>

                            {/* 查看完整详情按钮 */}
                            {details && (
                                <Button
                                    type="button"
                                    variant="ghost"
                                    size="sm"
                                    onClick={() => setOpenDetail(openDetail === 'full' ? 'none' : 'full')}
                                    className={cn(
                                        "cursor-pointer text-xs transition-colors flex items-center gap-1 p-0 h-auto",
                                        openDetail === 'full'
                                            ? "text-foreground font-medium"
                                            : "text-muted-foreground hover:text-foreground"
                                    )}
                                >
                                    <span>查看错误详情</span>
                                    <ChevronDown
                                        className={cn(
                                            "inline-block transition-transform duration-200",
                                            openDetail === 'full' ? "" : "-rotate-90"
                                        )}
                                        size={14} />
                                </Button>
                            )}

                            {/* 刷新按钮 */}
                            {onReset && (
                                <Button
                                    type="button"
                                    variant="ghost"
                                    size="sm"
                                    onClick={onReset}
                                    className={cn("cursor-pointer text-xs transition-colors flex items-center gap-1 text-muted-foreground hover:text-foreground p-0 h-auto")}
                                >
                                    <span>刷新</span>
                                    <RefreshCw className={cn("inline-block")} size={14} />
                                </Button>
                            )}
                        </div>

                        {/* 权限详情框 - 展开后显示 */}
                        {openDetail === 'perm' && (
                            <div className={cn("mt-2 p-3 rounded-md border bg-muted/50")}>
                                {/* Badge 标签列表 */}
                                <div className={cn("flex flex-wrap gap-2")}>
                                    {checkDetails.map((detail, index) => (
                                        <Badge
                                            key={index}
                                            variant="outline"
                                            className={cn("text-xs text-yellow-700 dark:text-yellow-300")}
                                        >
                                            {detail}
                                        </Badge>
                                    ))}
                                </div>
                            </div>
                        )}

                        {/* 完整详情框 - 展开后显示 */}
                        {openDetail === 'full' && details && (
                            <div className={cn("mt-2")}>
                                <pre className={cn("text-xs text-left bg-muted p-3 rounded-md overflow-auto max-h-32")}>
                                    {details.trim()}
                                </pre>
                            </div>
                        )}
                    </div>
                )}

                {/* 非权限错误时的完整详情和刷新按钮 */}
                {!isPermissionError && (onReset || details) && (
                    <div className={cn("mt-2")}>

                        <div className={cn("flex items-center justify-center gap-4 flex-col")}>
                            <div className={cn("flex flex-row gap-2")}>

                                {details && (
                                    <Button
                                        type="button"
                                        variant="ghost"
                                        size="sm"
                                        onClick={() => setOpenDetail(openDetail === 'full' ? 'none' : 'full')}
                                        className={cn("cursor-pointer text-xs transition-colors flex items-center gap-1 text-muted-foreground hover:text-foreground p-0 h-auto")}
                                    >
                                        <span>查看完整详情</span>
                                        <ChevronDown
                                            className={cn(
                                                "inline-block transition-transform duration-200",
                                                openDetail === 'none' ? "" : "-rotate-90"
                                            )}
                                            size={14} />
                                    </Button>
                                )}
                                {onReset && (
                                    <Button
                                        type="button"
                                        variant="ghost"
                                        size="sm"
                                        onClick={onReset}
                                        className={cn("cursor-pointer text-xs transition-colors flex items-center gap-1 text-muted-foreground hover:text-foreground p-0 h-auto")}
                                    >
                                        <span>刷新</span>
                                        <RefreshCw className={cn("inline-block")} size={14} />
                                    </Button>
                                )}
                            </div>

                            {/* 查看完整详情按钮 */}
                            {details && (
                                <div className={cn("mt-1")}>
                                    <pre className={cn(
                                        "mt-2 text-xs text-left bg-muted p-3 rounded-md overflow-auto max-h-32",
                                        openDetail === 'full' ? 'block' : 'hidden'
                                    )}>
                                        {details}
                                    </pre>
                                </div>
                            )}

                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}

/**
 * 处理标准 Error 对象 - 直接返回渲染组件
 */
function ErrorInstanceDisplay({ error, onReset }: { error: Error; onReset?: () => void }): ReactNode {
    const [openDetail, setOpenDetail] = useState(false);
    const message = error.message || '发生了一个错误';
    const details = error.stack;

    return (
        <div className={cn("flex flex-col items-center justify-center gap-4 text-center")}>
            {/* 图标容器 - 客户端错误使用警告图标 */}
            <div
                className={cn("rounded-full text-destructive/60 dark:text-destructive/70 bg-destructive/10 dark:bg-destructive/20")}
                style={{ padding: '20px' }}
            >
                <AlertTriangle size={40} />
            </div>

            {/* 文字内容 */}
            <div className={cn("space-y-2 max-w-md")}>
                {/* 客户端错误消息 */}
                <p className={cn("text-sm text-muted-foreground text-center")}>
                    {message}
                </p>
                {(onReset || details) && (
                    <div className={cn("mt-2")}>
                        <div className={cn("flex items-center justify-center gap-4 flex-col")}>
                            <div className={cn("flex flex-row gap-2")}>

                                {details && (
                                    <Button
                                        type="button"
                                        variant="ghost"
                                        size="sm"
                                        onClick={() => setOpenDetail(!openDetail)}
                                        className={cn("cursor-pointer text-xs transition-colors flex items-center gap-1 text-muted-foreground hover:text-foreground p-0 h-auto")}
                                    >
                                        <span>查看完整详情</span>
                                        <ChevronDown
                                            className={cn(
                                                "inline-block transition-transform duration-200",
                                                openDetail ? "" : "-rotate-90"
                                            )}
                                            size={14} />
                                    </Button>
                                )}
                                {onReset && (
                                    <Button
                                        type="button"
                                        variant="ghost"
                                        size="sm"
                                        onClick={onReset}
                                        className={cn("cursor-pointer text-xs transition-colors flex items-center gap-1 text-muted-foreground hover:text-foreground p-0 h-auto")}
                                    >
                                        <span>刷新</span>
                                        <RefreshCw className={cn("inline-block")} size={14} />
                                    </Button>
                                )}
                            </div>

                            {/* 错误详情 */}
                            {details && (
                                <div className={cn("mt-1")}>
                                    <div className={cn(
                                        "mt-2 text-xs text-left bg-muted rounded-md overflow-auto max-h-32",
                                        openDetail ? 'block' : 'hidden'
                                    )}>
                                        {details.split('\n').map((line, index) => (
                                            <div
                                                key={index}
                                                className={cn(index % 2 === 0 ? 'bg-muted/50 px-3 py-0.5' : 'px-3 py-0.5')}
                                            >
                                                {line || '\u00A0'}
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            )}
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}

/**
 * 处理简单字符串或其他类型错误 - 直接返回渲染组件
 */
function SimpleErrorDisplay({ error, onReset }: { error: unknown; onReset?: () => void }): ReactNode {
    const [openDetail, setOpenDetail] = useState(false);
    let message = '发生了未知错误';
    let details: string | undefined;

    if (typeof error === 'string') {
        message = error || '发生了未知错误';
    } else if (typeof error === 'number') {
        message = `错误代码: ${error}`;
    } else if (typeof error === 'boolean') {
        message = '操作失败';
    } else if (error === null) {
        message = '未知错误 (null)';
    } else if (error === undefined) {
        message = '未知错误 (undefined)';
    } else {
        // 其他对象类型，尝试转换为字符串
        try {
            const errorStr = JSON.stringify(error, null, 2);
            if (errorStr && errorStr !== '{}') {
                message = '操作失败，请查看详情';
                details = errorStr;
            }
        } catch {
            message = '发生了一个无法解析的错误';
        }
    }

    return (
        <div className={cn("flex flex-col items-center justify-center gap-4 text-center")}>
            {/* 图标容器 - 通用错误使用警告图标 */}
            <div
                className={cn("rounded-full text-destructive/60 dark:text-destructive/70 bg-destructive/10 dark:bg-destructive/20")}
                style={{ padding: '20px' }}
            >
                <AlertTriangle size={40} />
            </div>

            {/* 文字内容 */}
            <div className={cn("space-y-2 max-w-md")}>
                {/* 简单错误消息 */}
                <p className={cn("text-sm text-muted-foreground text-center")}>
                    {message}
                </p>
                {(onReset || details) && (
                    <div className={cn("mt-2")}>
                        <div className={cn("flex items-center justify-center gap-4 flex-col")}>
                            <div className={cn("flex flex-row gap-2")}>

                                {details && (
                                    <Button
                                        type="button"
                                        variant="ghost"
                                        size="sm"
                                        onClick={() => setOpenDetail(!openDetail)}
                                        className={cn("cursor-pointer text-xs transition-colors flex items-center gap-1 text-muted-foreground hover:text-foreground p-0 h-auto")}
                                    >
                                        <span>查看完整详情</span>
                                        <ChevronDown
                                            className={cn(
                                                "inline-block transition-transform duration-200",
                                                openDetail ? "" : "-rotate-90"
                                            )}
                                            size={14} />
                                    </Button>
                                )}
                                {onReset && (
                                    <Button
                                        type="button"
                                        variant="ghost"
                                        size="sm"
                                        onClick={onReset}
                                        className={cn("cursor-pointer text-xs transition-colors flex items-center gap-1 text-muted-foreground hover:text-foreground p-0 h-auto")}
                                    >
                                        <span>刷新</span>
                                        <RefreshCw className={cn("inline-block")} size={14} />
                                    </Button>
                                )}
                            </div>

                            {/* 错误详情 */}
                            {details && (
                                <div className={cn("mt-1")}>
                                    <pre className={cn(
                                        "mt-2 text-xs text-left bg-muted p-3 rounded-md overflow-auto max-h-32",
                                        openDetail ? 'block' : 'hidden'
                                    )}>
                                        {details}
                                    </pre>
                                </div>
                            )}
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}

/**
 * 解析错误对象，返回错误渲染组件
 */
function parseError(error: unknown, isInlineMode: boolean = false, onReset?: () => void): ReactNode {
    // 处理数组错误（取第一个非空错误）
    if (Array.isArray(error)) {
        const firstError = error.find(e => e != null);
        if (firstError) {
            return parseError(firstError, isInlineMode, onReset);
        }
        return isInlineMode
            ? SimpleErrorRowDisplay({ error: '没有可用的错误信息', onReset })
            : SimpleErrorDisplay({ error: '没有可用的错误信息', onReset });
    }

    // 行内模式渲染
    if (isInlineMode) {
        // 1. 服务器错误（优先级最高，因为最具体）
        if (isServerError(error)) {
            return ServerErrorRowDisplay({ error, onReset });
        }

        // 2. 标准 Error 对象
        if (isErrorInstance(error)) {
            return ErrorInstanceRowDisplay({ error, onReset });
        }

        // 3. 其他类型（字符串、数字、对象等）
        return SimpleErrorRowDisplay({ error, onReset });
    }

    // 默认模式渲染
    // 1. 服务器错误（优先级最高，因为最具体）
    if (isServerError(error)) {
        return ServerErrorDisplay({ error, onReset });
    }

    // 2. 标准 Error 对象
    if (isErrorInstance(error)) {
        return ErrorInstanceDisplay({ error, onReset });
    }

    // 3. 其他类型（字符串、数字、对象等）
    return SimpleErrorDisplay({ error, onReset });
}

/**
 * CenteredError
 * 居中错误显示组件，支持5种展示模式
 * 自动根据错误类型智能展示
 */
export function CenteredError({
    className,
    variant = 'content',
    error,
    children,
    onReset,
}: CenteredErrorProps) {
    // 解析错误，得到渲染组件（已包含图标和文字）
    const isInlineMode = variant === 'inline';
    const errorContent = parseError(error, isInlineMode, onReset);

    const ariaProps = {
        role: "alert" as const,
        "aria-live": "assertive" as const,
        "aria-label": "错误信息",
    };

    // 模式 5: 行内模式（图标、消息、操作按钮排成一行）
    if (variant === 'inline') {
        return (
            <div
                className={cn(
                    "flex items-center min-h-[32px] px-2 py-1",
                    className
                )}
                {...ariaProps}
            >
                {errorContent}
                {/* 子元素区域 - 显示在行的最右侧 */}
                {children && (
                    <div className={cn("ml-2 shrink-0")}>
                        {children}
                    </div>
                )}
            </div>
        );
    }

    // 渲染灰色 info 图标（用于左侧边栏和上面过滤块）
    const renderGrayIcon = (size: 'small' | 'large') => {
        // 容器的 padding - 稍微增加留白
        const padding = size === 'large' ? '18px' : '14px';
        // 图标尺寸 - 稍微减小图标
        const iconSize = size === 'large' ? 32 : 26;

        return (
            <div className={cn("flex items-center justify-center w-full h-full")}>
                <div
                    className={cn("rounded-full text-muted-foreground/40 bg-muted/30")}
                    style={{ padding }}
                >
                    <Info size={iconSize} />
                </div>
            </div>
        );
    };

    // 模式 4: 左右两栏布局（左侧边栏，右侧上下两栏）
    if (variant === 'layout') {
        return (
            <div className={cn("flex", className)} {...ariaProps}>
                {/* 左侧边栏 - 显示灰色破裂图标（固定）- 移动端隐藏 */}
                <div className={cn("hidden md:flex w-64 shrink-0 border-r min-h-[calc(100vh-3rem)] flex-col items-center justify-center")}>
                    {renderGrayIcon('large')}
                </div>

                {/* 右侧内容区域（上下两栏或三栏） */}
                <div className={cn("flex-1 sm:p-4 sm:space-y-4 min-w-0")}>
                    {/* 上面过滤块（较小）- 显示灰色破裂图标（固定）- 移动端隐藏 */}
                    <div className={cn("hidden sm:flex relative flex-wrap items-center justify-center gap-2 lg:gap-3 px-4 py-4 sm:py-5 bg-card border rounded-lg shadow-sm h-16 sm:h-24")}>
                        {renderGrayIcon('small')}
                    </div>

                    {/* 下面内容块（较大，填充剩余空间）- 显示完整错误 */}
                    <div className={cn("rounded-lg border bg-card shadow-sm flex-1 flex flex-col items-center justify-center p-4 sm:p-6 min-h-[calc(100vh-6rem)] sm:min-h-0")}>
                        {errorContent}
                        {/* 子元素区域 - 显示在错误内容底部 */}
                        {children && (
                            <div className={cn("mt-4 sm:mt-6 w-full")}>
                                {children}
                            </div>
                        )}
                    </div>
                </div>
            </div>
        );
    }

    // 模式 3: 上下两栏布局（上面过滤块-小，下面内容块-大）
    if (variant === 'page') {
        return (
            <div className={cn("sm:space-y-4 min-h-[calc(100vh-6rem)] sm:min-h-[calc(100vh-8rem)]", className)} {...ariaProps}>
                {/* 上面过滤块（较小）- 显示灰色破裂图标（固定）- 移动端隐藏 */}
                <div className={cn("hidden sm:flex rounded-lg border bg-card shadow-sm h-16 sm:h-24 flex-col items-center justify-center p-4 sm:p-6")}>
                    {renderGrayIcon('small')}
                </div>

                {/* 下面内容块（较大）- 显示完整错误 */}
                <div className={cn("rounded-lg border bg-card shadow-sm min-h-[300px] sm:min-h-[500px] flex flex-col items-center justify-center p-4 sm:p-6")}>
                    {errorContent}
                    {/* 子元素区域 - 显示在错误内容底部 */}
                    {children && (
                        <div className={cn("mt-4 sm:mt-6 w-full")}>
                            {children}
                        </div>
                    )}
                </div>
            </div>
        );
    }

    // 模式 2: 带卡片容器的错误提示
    if (variant === 'card') {
        return (
            <div
                className={cn(
                    "flex flex-col items-center justify-center rounded-lg border border-destructive/20 bg-card text-card-foreground shadow-sm",
                    "min-h-[300px] p-8",
                    className
                )}
                {...ariaProps}
            >
                {errorContent}
                {/* 子元素区域 - 显示在错误内容底部 */}
                {children && (
                    <div className={cn("mt-6 w-full")}>
                        {children}
                    </div>
                )}
            </div>
        );
    }

    // 模式 1: 无边框模式（仅居中错误内容）
    return (
        <div
            className={cn(
                "flex flex-col items-center justify-center min-h-[200px] py-8",
                className
            )}
            {...ariaProps}
        >
            {errorContent}
            {/* 子元素区域 - 显示在错误内容底部 */}
            {children && (
                <div className={cn("mt-6 w-full")}>
                    {children}
                </div>
            )}
        </div>
    );
}
