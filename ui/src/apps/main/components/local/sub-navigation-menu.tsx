import { Button } from "@shared/components/ui/button"
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from "@shared/components/ui/dropdown-menu"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@shared/components/ui/tabs"
import { cn } from "@shared/lib/utils"
import { MenuItem, MenuItemState } from "@apps/main/types/ui-menu"
import { Link, useNavigate } from "@tanstack/react-router"
import { ChevronDown } from "lucide-react"
import { ReactNode, useEffect, useMemo, useState } from "react"

// Tailwind CSS 默认的 lg 断点值为 1024px
// 如果在 tailwind.config 中自定义了 lg 断点，需要同步修改此值
const LG_BREAKPOINT = 1024

/**
 * 次级导航菜单配置信息
 */
export interface SubNavigationMenuInfo {
    /**
     * 页面标题（可选）
     * 如果不提供，会自动从当前匹配的菜单项中解析
     */
    title?: string
    /**
     * 页面副标题（可选）
     */
    subtitle?: string
    /**
     * 菜单项列表（必填）
     */
    menuItems: MenuItem[]
}

/**
 * 次级导航容器组件的属性
 */
export interface SubNavigationContainerProps extends SubNavigationMenuInfo {
    /**
     * 操作区域（可选）
     * 用于放置按钮等操作组件
     */
    actions?: ReactNode
    /**
     * 子元素内容
     */
    children: ReactNode
    /**
     * 自定义样式类名（可选）
     */
    className?: string
    /**
     * 是否正在加载（可选）
     * 加载时显示骨架屏
     */
    isLoading?: boolean
    /**
     * 是否发生错误（可选）
     */
    isError?: boolean
    /**
     * 错误信息（可选）
     */
    error?: any
}

/**
 * 通用次级导航容器组件
 * 
 * 功能特性：
 * - 显示页面标题和副标题
 * - 支持桌面端标签页导航和移动端下拉菜单导航
 * - 自动通过 menuItems 的 isActive 属性判断激活项
 * - 支持自定义操作按钮区域
 * - 支持加载状态和错误状态
 * 
 * @example
 * ```tsx
 * <SubNavigationContainer
 *   title="用户管理"
 *   subtitle="管理系统用户信息"
 *   menuItems={[
 *     { name: '用户列表', path: '/admin/users', icon: Users, isActive: true },
 *     { name: '角色管理', path: '/admin/roles', icon: Shield }
 *   ]}
 *   actions={<Button variant="outline" size="sm"><Plus className="mr-2 h-4 w-4" />添加用户</Button>}
 * >
 *   <UserListContent />
 * </SubNavigationContainer>
 * ```
 */
export function SubNavigationContainer({
    title,
    subtitle,
    menuItems,
    actions,
    children,
    className,
}: SubNavigationContainerProps) {
    // 使用 Tailwind lg 断点判断是否为移动端
    const [isMobile, setIsMobile] = useState<boolean>(() => {
        if (typeof window === 'undefined') return false
        return window.innerWidth < LG_BREAKPOINT
    })

    useEffect(() => {
        const mql = window.matchMedia(`(max-width: ${LG_BREAKPOINT - 1}px)`)
        const onChange = () => {
            setIsMobile(window.innerWidth < LG_BREAKPOINT)
        }
        mql.addEventListener("change", onChange)
        return () => mql.removeEventListener("change", onChange)
    }, [])
    
    // 解析当前页面标题
    const resolvedTitle = useMemo(() => {
        if (title) return title

        // 查找激活的菜单项
        const activeItem = menuItems.find(item => item.isActive)
        if (activeItem) {
            return activeItem.name
        }

        // 如果找不到激活项，返回第一个菜单项的名称
        return menuItems[0]?.name || ''
    }, [title, menuItems])

    return (
        <div className={cn('flex h-full flex-1 flex-col mt-4 md:mt-0', className)}>
            {/* 标题和移动端菜单区域 */}
            <div className={cn('mb-4', menuItems.length === 0 ? "pb-2" : "")}>
                <div className="flex items-center justify-between gap-2">
                    {/* 标题区域 */}
                    <div className="min-w-0 flex-1">
                        <h1 className='text-lg lg:text-2xl font-bold tracking-tight'>
                            {resolvedTitle}
                        </h1>
                        {subtitle && (
                            <p className='hidden lg:block text-sm text-muted-foreground mt-1'>
                                {subtitle}
                            </p>
                        )}
                    </div>

                    {/* 当没有菜单项时，直接显示操作按钮 */}
                    {menuItems.length === 0 && actions && (
                        <div className="flex-shrink-0">
                            {actions}
                        </div>
                    )}

                    {/* 移动端下拉菜单 */}
                    {menuItems.length > 0 && (
                        <div className="flex-shrink-0 lg:hidden">
                            <SubNavigationDropdown menuItems={menuItems} />
                        </div>
                    )}
                </div>

                {/* 移动端操作按钮区域 */}
                {menuItems.length > 0 && actions && (
                    <div className="mt-3 flex flex-wrap gap-2 lg:hidden justify-end">
                        {actions}
                    </div>
                )}
            </div>

            {/* 内容区域 */}
            {menuItems.length > 0 && !isMobile ? (
                <SubNavigationTabs menuItems={menuItems} actions={actions}>
                    {children}
                </SubNavigationTabs>
            ) : (
                children
            )}
        </div>
    )
}

/**
 * 次级导航标签页组件（桌面端）
 */
interface SubNavigationTabsProps {
    menuItems: MenuItem[]
    actions?: ReactNode
    children: ReactNode
    className?: string
}

function SubNavigationTabs({
    menuItems,
    actions,
    children,
    className,
}: SubNavigationTabsProps) {
    // 查找激活的菜单项
    const tabValue = useMemo(() => {
        const activeItem = menuItems.find(item => item.isActive) || menuItems[0]
        return activeItem?.id || activeItem?.path || ''
    }, [menuItems])

    return (
        <>
            {/* 桌面端标签页导航 */}
            <Tabs
                orientation='vertical'
                value={tabValue}
                className={cn('space-y-4', className)}
            >
                <div className='w-full pb-2 flex flex-wrap items-center justify-between gap-4'>
                    <TabsList className="flex-shrink-0">
                        {menuItems.map((item) => (
                            <TabsTrigger
                                key={item.id || item.path}
                                value={item.id || item.path || ''}
                                disabled={item.state === MenuItemState.DISABLED}
                                asChild={!!item.path}
                            >
                                {item.path ? (
                                    <Link to={item.path} params={item.params}>
                                        {item.icon && <item.icon className=" h-4 w-4" />}
                                      <span className="ml-2">{item.name}</span>
                                    </Link>
                                ) : (
                                    <>
                                        {item.icon && <item.icon className=" h-4 w-4" />}
                                      <span className="ml-2">{item.name}</span>
                                    </>
                                )}
                            </TabsTrigger>
                        ))}
                    </TabsList>

                    {/* 桌面端操作按钮区域 */}
                    {actions && (
                        <div className="flex flex-wrap gap-2">
                            {actions}
                        </div>
                    )}
                </div>

                {/* 标签页内容 */}
                <TabsContent
                    value={tabValue}
                    className={cn('space-y-4')}
                >
                    {children}
                </TabsContent>
            </Tabs>
        </>
    )
}

/**
 * 次级导航下拉菜单组件（移动端）
 */
interface SubNavigationDropdownProps {
    menuItems: MenuItem[]
}

function SubNavigationDropdown({ menuItems }: SubNavigationDropdownProps) {
    const navigate = useNavigate()

    // 查找激活的菜单项
    const activeItem = menuItems.find(item => item.isActive) || menuItems[0]

    // 处理菜单项点击
    const handleMenuItemClick = (item: MenuItem) => {
        if (item.path && item.state !== MenuItemState.DISABLED) {
            navigate({ to: item.path, params: item.params })
        }
    }

    return (
        <DropdownMenu>
            <DropdownMenuTrigger asChild>
                <Button variant="outline" size="sm" className="gap-2">
                    {activeItem?.icon && <activeItem.icon className="h-4 w-4" />}
                    {activeItem?.name || '选择'}
                    <ChevronDown className="h-4 w-4" />
                </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-48 space-y-1 p-1">
                {menuItems.map((item) => (
                    <DropdownMenuItem
                        key={item.id || item.path}
                        disabled={item.state === MenuItemState.DISABLED}
                        onClick={() => handleMenuItemClick(item)}
                        className={cn(
                            "cursor-pointer",
                            item.isActive && "bg-accent"
                        )}
                    >
                        {item.icon && <item.icon className=" h-4 w-4" />}
                        <span className="ml-2">{item.name}</span>   
                    </DropdownMenuItem>
                ))}
            </DropdownMenuContent>
        </DropdownMenu>
    )
}
