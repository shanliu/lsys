import { AppListItemType } from "@shared/apis/user/app"
import { SubNavigationContainer, SubNavigationContainerProps } from "@apps/main/components/local/sub-navigation-menu"
import { useLocation, useMatches } from "@tanstack/react-router"
import { useMemo } from "react"
import { ServiceModuleMenuInfo } from "../../pages/app/detail/nav-info"

/**
 * 应用详情页导航容器组件
 * 基于 SubNavigationContainer 实现，自动处理菜单加载和激活状态
 */
export function AppDetailNavContainer({
    title,
    subtitle,
    menuItems = [],
    actions,
    children,
    className,
}: SubNavigationContainerProps) {
    const location = useLocation();
    const matches = useMatches();
    // 使用传入的 menuItems 或从 hook 获取的菜单项
    const finalMenuItems = menuItems;

    // 计算带有 isActive 属性的菜单项
    const menuItemsWithActive = useMemo(() => {
        return finalMenuItems.map(item => {
            const itemPath = item.path;
            if (!itemPath) {
                return { ...item, isActive: false };
            }

            // 使用 matches 检查是否匹配当前路由
            const isMatched = matches.some((match) => {
                if (match.pathname === itemPath || match.fullPath === itemPath) {
                    return true;
                }

                // 对于参数化路径，检查模式匹配
                if (itemPath.includes('$')) {
                    const pathPattern = itemPath.replace(/\$\w+/g, '\\d+');
                    const regex = new RegExp(`^${pathPattern}$`);
                    return regex.test(match.pathname);
                }

                return false;
            });

            if (isMatched) {
                return { ...item, isActive: true };
            }

            // 回退：直接比较路径
            if (location.pathname === itemPath) {
                return { ...item, isActive: true };
            }

            return { ...item, isActive: false };
        });
    }, [location.pathname, matches, finalMenuItems]);

    // 如果没有任何菜单项被激活，激活第一个
    const finalMenuItemsWithActive = useMemo(() => {
        const hasActive = menuItemsWithActive.some(item => item.isActive);
        if (!hasActive && menuItemsWithActive.length > 0) {
            return menuItemsWithActive.map((item, index) => 
                index === 0 ? { ...item, isActive: true } : item
            );
        }
        return menuItemsWithActive;
    }, [menuItemsWithActive]);

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
    );
}

export function AppDetailServiceModuleNavContainer({
    title,
    subtitle,
    menuItems = [],
    actions,
    children,
    className,
    appData,
    validMenuIndexes = []
}: SubNavigationContainerProps & ServiceModuleMenuInfo & {
    appData?: AppListItemType | null
}) {
    // 如果 appData.parent_app 为空，传递所有 menuItems
    // 否则，根据 validMenuIndexes 过滤 menuItems
    const filteredMenuItems = useMemo(() => {
        if (!appData?.parent_app) {
            return menuItems;
        }

        if (validMenuIndexes.length === 0) {
            return [];
        }

        return menuItems.filter((_, index) => validMenuIndexes.includes(index));
    }, [appData?.parent_app, menuItems, validMenuIndexes]);

    return <AppDetailNavContainer
        title={title}
        subtitle={subtitle}
        menuItems={filteredMenuItems}
        actions={actions}
        className={className}
        children={children}
    />
}
