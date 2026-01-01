import { SubNavigationContainer, SubNavigationContainerProps } from "@apps/main/components/local/sub-navigation-menu";
import { useMatches } from "@tanstack/react-router";
import { useMemo } from "react";

/**
 * 邮件发送配置导航容器组件
 * 专门用于邮件发送配置页面，自动处理菜单加载和激活状态
 */
export function EmailSendConfigNavContainer({
    title,
    subtitle,
    menuItems = [],
    actions,
    children,
    className,
}: SubNavigationContainerProps) {
    const matches = useMatches()

    // 使用传入的 menuItems
    const finalMenuItems = menuItems

    // 计算带有 isActive 属性的菜单项
    const menuItemsWithActive = useMemo(() => {
        return finalMenuItems.map(item => {
            const itemPath = item.path
            if (itemPath) {
                // 使用 matches 检查是否匹配当前路由
                const isMatched = matches.some((match) => {
                    if (match.pathname === itemPath || match.fullPath === itemPath) {
                        return true
                    }
                    
                    // 检查是否是子路由
                    if (match.pathname.startsWith(itemPath)) {
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
    }, [matches, finalMenuItems])

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
