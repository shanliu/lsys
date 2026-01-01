import { SubNavigationContainer, SubNavigationContainerProps } from "@apps/main/components/local/sub-navigation-menu";
import { Route } from '@apps/main/routes/_main/admin/email/adapter-config';
import { useLocation, useMatches } from "@tanstack/react-router";
import { useMemo } from "react";

/**
 * 邮件适配器配置导航容器组件
 * 基于 SubNavigationContainer 实现，自动处理菜单加载和激活状态
 * 支持通过 type 查询参数自动确定当前激活的菜单项
 */
export function EmailAdapterConfigNavContainer({
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

    // 使用传入的 menuItems，设置激活状态
    const finalMenuItems = useMemo(() => {
        let activeValue = ''

        // 首先尝试从 type 查询参数中确定激活的菜单项
        const queryType = search.type
        if (queryType) {
            // 查找匹配的菜单项
            const matchedItem = menuItems.find((item) => {
                if (!item.path) return false
                // 检查路径中是否包含 type 参数
                return item.path.includes(`type=${queryType}`)
            })
            if (matchedItem) {
                activeValue = matchedItem.id || matchedItem.path || ''
            }
        }

        // 回退：使用路径匹配
        if (!activeValue) {
            for (const item of menuItems) {
                const itemPath = item.path
                if (!itemPath) continue

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
                    activeValue = item.id || itemPath || ''
                    break
                }

                // 直接比较路径
                if (location.pathname === itemPath) {
                    activeValue = item.id || itemPath || ''
                    break
                }
            }
        }

        // 如果没有匹配项，使用第一个菜单项作为默认值
        if (!activeValue) {
            activeValue = menuItems[0]?.id || menuItems[0]?.path || ''
        }

        // 设置激活状态
        return menuItems.map((item) => ({
            ...item,
            isActive: (item.id === activeValue || item.path === activeValue)
        }))
    }, [location.pathname, matches, menuItems, search])

    return (
        <SubNavigationContainer
            title={title}
            subtitle={subtitle}
            menuItems={finalMenuItems}
            actions={actions}
            className={className}
        >
            {children}
        </SubNavigationContainer>
    )
}
