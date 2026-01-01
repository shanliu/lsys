/**
 * 状态颜色管理工具
 * 
 * 提供统一的状态样式管理，支持明暗主题自动切换
 * 基于 CSS 变量定义的语义化颜色（定义在 src/styles/custom.css）
 * 
 * 可用的状态颜色：
 * - neutral: 中性状态（灰色）- 用于初始化、禁用等中性状态
 * - info: 信息状态（蓝色）- 用于处理中、进行中等信息状态  
 * - warning: 警告状态（黄色）- 用于待审核、警告等需要注意的状态
 * - success: 成功状态（绿色）- 用于成功、启用、完成等正面状态
 * - danger: 危险状态（红色）- 用于失败、错误、拒绝等负面状态
 * 
 * @example
 * // 创建状态映射器
 * const appStatus = createStatusMapper({
 *   0: 'neutral',   // 初始化
 *   1: 'warning',   // 待审核
 *   2: 'success',   // 启用
 *   3: 'danger'     // 禁用
 * })
 * 
 * // 使用
 * <Badge className={appStatus.getClass(status)}>
 *   {getStatusText(status)}
 * </Badge>
 */

/**
 * 状态颜色类型
 * 对应 custom.css 中定义的状态颜色变量
 */
export type StatusColor = 'neutral' | 'info' | 'warning' | 'success' | 'danger'

/**
 * 状态配置项
 */
export interface StatusConfig {
    /** 颜色类型 */
    color: StatusColor
    /** 显示文本（可选，如果不提供则需要通过 textGetter 获取） */
    label?: string
}

/**
 * 文本获取器类型
 */
export type TextGetter<T extends number | string = number> = (status: T) => string

/**
 * 状态映射器接口
 */
export interface StatusMapper<T extends number | string = number> {
    /** 获取 Badge 类名 */
    getClass: (status: T) => string
    /** 获取背景色类名 */
    getBgClass: (status: T) => string
    /** 获取文本颜色类名 */
    getTextClass: (status: T) => string
    /** 获取边框颜色类名 */
    getBorderClass: (status: T) => string
    /** 获取状态文本 */
    getText: (status: T) => string
}

/**
 * 创建状态映射器
 * 
 * @param colorMap 状态值到颜色的映射表
 * @param textGetter 文本获取器（可选）- 可以是字典 getLabel 方法或自定义函数
 * @returns 状态映射器对象
 * 
 * @example
 * // 方式1：使用字典数据（推荐）
 * const appStatus = createStatusMapper(
 *   {
 *     0: 'neutral',
 *     1: 'warning',
 *     2: 'success',
 *     3: 'danger'
 *   },
 *   (status) => dictData.app_status.getLabel(String(status)) || String(status)
 * )
 * 
 * @example
 * // 方式2：直接写死文本（不推荐，但支持）
 * const appStatus = createStatusMapper(
 *   {
 *     0: 'neutral',
 *     1: 'warning',
 *     2: 'success',
 *     3: 'danger'
 *   },
 *   (status) => {
 *     const labels = { 0: '初始化', 1: '待审核', 2: '启用', 3: '禁用' }
 *     return labels[status] || String(status)
 *   }
 * )
 * 
 * @example
 * // 方式3：只传颜色映射，自己处理文本
 * const appStatus = createStatusMapper({
 *   0: 'neutral',
 *   1: 'warning',
 *   2: 'success',
 *   3: 'danger'
 * })
 * 
 * // 使用
 * <Badge className={appStatus.getClass(status)}>
 *   {dictData.app_status.getLabel(String(status))}
 * </Badge>
 */
export function createStatusMapper<T extends number | string = number>(
    colorMap: Record<T, StatusColor>,
    textGetter?: TextGetter<T extends number ? number : T>
): StatusMapper<T extends number ? number : T> {
    const getColor = (status: T extends number ? number : T): StatusColor => {
        return (colorMap as any)[status] || 'neutral'
    }

    return {
        getClass: (status: T extends number ? number : T) => {
            const color = getColor(status)
            return `badge-status-${color}`
        },
        getBgClass: (status: T extends number ? number : T) => {
            const color = getColor(status)
            return `bg-status-${color}`
        },
        getTextClass: (status: T extends number ? number : T) => {
            const color = getColor(status)
            return `text-status-${color}-foreground`
        },
        getBorderClass: (status: T extends number ? number : T) => {
            const color = getColor(status)
            return `border-status-${color}`
        },
        getText: (status: T extends number ? number : T) => {
            if (textGetter) {
                return textGetter(status)
            }
            return String(status)
        },
    }
}
