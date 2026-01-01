import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { isRedirect, Navigate, type ErrorComponentProps } from "@tanstack/react-router";

/**
 * PageErrorBoundary 创建器配置
 */
export interface PageErrorBoundaryCreaterOptions {
    /** 自定义类名 */
    className?: string;
    /** 
     * 错误展示变体
     * - 'content': 无边框模式（仅居中错误内容）
     * - 'card': 带卡片容器的错误提示
     * - 'page': 上下两栏布局（上面过滤块-小，下面内容块-大，每块中间提示）
     * - 'layout': 左右两栏布局（左侧边栏，右侧上下两栏，每块中间提示）
     * - 'inline': 行内模式（图标、消息、操作按钮排成一行）
     */
    variant?: 'content' | 'card' | 'page' | 'layout' | 'inline';
}

/**
 * PageErrorBoundaryCreater
 * 创建一个可用于 errorComponent 的错误边界组件
 * 自动在错误消息中显示刷新按钮（如果 reset 函数可用）
 * 
 * @param options - 配置选项（className 和 variant）
 * @returns 返回一个可用于 errorComponent 的函数组件
 * 
 * @example
 * ```tsx
 * // 在路由配置中使用
 * export const Route = createRoute({
 *   path: '/dashboard',
 *   component: Dashboard,
 *   errorComponent: PageErrorBoundaryCreater({ variant: 'page' }),
 * })
 * 
 * // 使用 layout 模式
 * errorComponent: PageErrorBoundaryCreater({ variant: 'layout' }),
 * ```
 */
export function PageErrorBoundaryCreater(
    options: PageErrorBoundaryCreaterOptions = {}
) {
    const { className, variant = 'content' } = options;

    // 返回一个符合 errorComponent 接口的函数组件
    return function PageErrorBoundary(props: ErrorComponentProps) {
        const { error, reset, info } = props;

        // 处理错误对象
        let processedError: unknown = error;

        // 如果错误是 Error 实例且有 componentStack，将其添加到 stack 中
        if (error instanceof Error && info?.componentStack) {
            // 创建一个增强的错误对象，包含组件堆栈信息
            processedError = {
                ...error,
                message: error.message,
                stack: error.stack,
                componentStack: info.componentStack,
            };
        }

        // 将处理后的错误传递给 CenteredError，并传递 reset 函数
        return (
            <CenteredError
                className={className}
                variant={variant}
                error={processedError}
                onReset={reset}
            >
                {isRedirect(error) && error.options && <Navigate {...error.options} />}
            </CenteredError>
        );
    };
}
