import { SubNavigationContainer, SubNavigationContainerProps } from "@apps/main/components/local/sub-navigation-menu";
import { Route } from '@apps/main/routes/_main/admin/config';
import { useMatches } from "@tanstack/react-router";
import { useMemo } from "react";

/**
 * 系统配置导航容器组件
 * 基于 SubNavigationContainer 实现，自动处理菜单加载和激活状态
 * 支持通过 type 查询参数自动确定当前激活的菜单项
 */
export function ConfigNavContainer({
    title,
    subtitle,
    menuItems = [],
    actions,
    children,
    className,
}: SubNavigationContainerProps) {
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
                    return false
                })

                if (isMatched) {
                    return { ...item, isActive: true }
                }
            }

            return { ...item, isActive: false }
        })
    }, [finalMenuItems, matches, search.type])

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
