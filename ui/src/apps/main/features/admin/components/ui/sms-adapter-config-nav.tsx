import { SubNavigationContainer, SubNavigationContainerProps } from "@apps/main/components/local/sub-navigation-menu";
import { Route } from '@apps/main/routes/_main/admin/sms/adapter-config';
import { useLocation, useMatches } from "@tanstack/react-router";
import { useMemo } from "react";
/**
 * 短信适配器配置导航容器组件
 * 基于 SubNavigationContainer 实现，自动处理菜单加载和激活状态
 * 支持通过 type 查询参数自动确定当前激活的菜单项
 */
export function SmsAdapterConfigNavContainer({
    title,
    subtitle,
    menuItems = [],
    actions,
    children,
    className,
}: SubNavigationContainerProps) {
    const location = useLocation()
    const matches = useMatches()
    const search = Route.useSearch()

    // 使用传入的 menuItems
    const finalMenuItems = menuItems

    // 计算带有 isActive 属性的菜单项
    const menuItemsWithActive = useMemo(() => {
        return finalMenuItems.map(item => {
            // 首先尝试从 type 查询参数中确定激活的菜单项
            const queryType = search.type
            if (queryType && item.path) {
                // 检查路径中是否包含 type 参数
                if (item.path.includes(`type=${queryType}`)) {
                    return { ...item, isActive: true }
                }
            }

            // 回退：使用路径匹配
            const itemPath = item.path
            if (itemPath) {
                // 使用 matches 检查是否匹配当前路由
                const isMatched = matches.some((match) => {
                    if (match.pathname === itemPath || match.fullPath === itemPath) {
                        return true
                    }

                    // 对于参数化路径，检查模式匹配
                    if (itemPath.includes('$')) {
                        const pathPattern = itemPath.replace(/\$\w+/g, '\\d+')
                        const regex = new RegExp(`^${pathPattern}$`)
                        return regex.test(match.pathname)
                    }

                    return false
                })

                if (isMatched) {
                    return { ...item, isActive: true }
                }

                // 直接比较路径
                if (location.pathname === itemPath) {
                    return { ...item, isActive: true }
                }
            }

            return { ...item, isActive: false }
        })
    }, [location.pathname, matches, finalMenuItems, search])

    // 如果没有任何菜单项被激活，激活第一个
    const finalMenuItemsWithActive = useMemo(() => {
        const hasActive = menuItemsWithActive.some(item => item.isActive)
        if (!hasActive && menuItemsWithActive.length > 0) {
            return menuItemsWithActive.map((item, index) => 
                index === 0 ? { ...item, isActive: true } : item
            )
        }
        return menuItemsWithActive
    }, [menuItemsWithActive])

    return (
        <SubNavigationContainer
            title={title}
            subtitle={subtitle}
            menuItems={finalMenuItemsWithActive}
            actions={actions}
            className={className}
        >
            {children}
        </SubNavigationContainer>
    )
}
