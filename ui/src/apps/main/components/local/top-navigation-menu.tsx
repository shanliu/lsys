import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { Button } from "@shared/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@shared/components/ui/dropdown-menu";
import { Skeleton } from "@shared/components/ui/skeleton";
import { useToast } from "@shared/contexts/toast-context";
import { cn } from "@shared/lib/utils";
import { generateMenuKey } from "@apps/main/lib/menu-utils";
import { MenuItem, MenuItemState } from "@apps/main/types/ui-menu";
import {
  Link,
} from "@tanstack/react-router";
import { ChevronDown, Menu } from "lucide-react";
import { MenuTipIcon } from "@apps/main/components/text/menu-tip-icon";

export interface TopNavigationMenuProps {
  className?: string;
  isLoading?: boolean;
  isError?: boolean;
  error?: string | null;
  loadingSkeletonCount?: number;
  menuItems: MenuItem[];
}

export function TopNavigationMenu({
  className,
  isLoading = false,
  isError = false,
  error = null,
  loadingSkeletonCount = 5,
  menuItems = [],
}: TopNavigationMenuProps) {

  if (isLoading) {
    return (
      <>
        {/* 移动端加载动画 - 只显示1个 */}
        <div className="lg:hidden">
          <Skeleton className={cn("h-9 w-20 rounded-md")} />
        </div>

        {/* 桌面端加载动画 - 显示指定数量 */}
        <div className={cn("hidden items-center space-x-2 lg:flex", className)}>
          {Array.from({ length: loadingSkeletonCount }).map((_, i) => (
            <Skeleton key={i} className={cn("h-9 w-28 rounded-md")} />
          ))}
        </div>
      </>
    );
  }

  if (isError) {
    return (
      <div className={cn(className)}>
        <CenteredError error={error} variant="inline" />
      </div>
    );
  }

  // 递归查找最深层激活的菜单项（用于移动端按钮显示）
  // 优先返回子菜单中激活的项，这样能找到最具体的激活菜单
  const findActiveMenuItem = (items: MenuItem[]): MenuItem | undefined => {
    for (const item of items) {
      // 先检查子菜单，找到最深层的激活项
      if (item.children && item.children.length > 0) {
        const activeChild = findActiveMenuItem(item.children);
        if (activeChild) {
          return activeChild;
        }
      }
      // 如果没有激活的子菜单，检查当前项
      if (item.isActive) {
        return item;
      }
    }
    return undefined;
  };

  // 递归查找是否有任何菜单项激活（用于父级高亮判断）
  const hasActiveDescendant = (menuItem: MenuItem): boolean => {
    if (menuItem.isActive) {
      return true;
    }
    if (menuItem.children && menuItem.children.length > 0) {
      return menuItem.children.some(child => hasActiveDescendant(child));
    }
    return false;
  };

  return (
    <div className={cn("flex items-center w-full relative min-w-0 lg:mt-1")}>
      {/* 移动端菜单 - 在小于 lg 屏幕显示 */}
      <div className={cn("block lg:hidden -ml-2")}>
        <DropdownMenu modal={false}>
          <DropdownMenuTrigger asChild>
            <Button
              size="sm"
              variant="outline"
              className={cn("h-9 px-2 gap-1.5 mt-0.5")}
            >
              <Menu className={cn("h-3.5 w-3.5 flex-shrink-0")} />
              <span className={cn("text-[11px] sm:text-xs truncate")}>
                {findActiveMenuItem(menuItems)?.name || "菜单"}
              </span>
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent
            side="bottom"
            align="start"
            className={cn("space-y-0.5 py-2")}
          >
            {menuItems.map((item, index) => (
              <MobileMenuItem
                key={generateMenuKey(item, "mobile", index)}
                item={item}
                hasActiveDescendant={hasActiveDescendant}
              />
            ))}
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      {/* 桌面端菜单 - 只在 lg 及以上屏幕显示 */}
      <nav
        className={cn(
          "hidden lg:flex items-center space-x-2",
          className,
        )}
      >
        {menuItems.map((item, index) => (
          <TopNavigationMenuItem
            key={generateMenuKey(item, "desktop", index)}
            item={item}
          />
        ))}
      </nav>
    </div>
  );
}

interface TopNavigationMenuItemProps {
  item: MenuItem;
}

interface MobileMenuItemProps {
  item: MenuItem;
  hasActiveDescendant: (menuItem: MenuItem) => boolean;
}

// 移动端菜单项组件 - 独立的选中逻辑处理
function MobileMenuItem({ item, hasActiveDescendant }: MobileMenuItemProps) {
  const hasChildren = item.children && item.children.length > 0;
  const isDisabled = item.state === MenuItemState.DISABLED;
  const { error } = useToast();

  const handleErrorClick = (errorMessage?: string) => {
    if (errorMessage) {
      error(errorMessage);
    }
  };

  const isActive = item.isActive || false;
  const isChildActive = hasChildren && item.children?.some(child => hasActiveDescendant(child));

  // 移动端：计算是否应该显示为激活状态
  // - 如果父级菜单有路径，只有在没有子项激活时，才允许父项显示为激活状态
  // - 如果父级菜单没有路径（如分组菜单），当有子项激活时，父级应该显示为激活状态
  const shouldShowAsActive = hasChildren
    ? item.path
      ? isActive && !isChildActive
      : isChildActive
    : isActive;

  if (hasChildren && item.children && item.children.length > 0) {
    // 有子菜单的情况，显示父级和子级
    return (
      <>
        {/* 父级菜单项 */}
        {item.path && !isDisabled ? (
          <DropdownMenuItem
            key={generateMenuKey(item, "parent")}
            asChild
            className={cn(
              shouldShowAsActive || isChildActive
                ? "bg-accent text-accent-foreground"
                : "",
            )}
          >
            <Link
              to={item.path}
              params={item.params as any}
              className={cn("flex items-center space-x-2")}
            >
              {item.icon && <item.icon className="h-4 w-4" />}
              <span>{item.name}</span>
            </Link>
          </DropdownMenuItem>
        ) : (
          <DropdownMenuItem
            key={generateMenuKey(item, "parent")}
            disabled
            className={cn(
              "flex items-center space-x-2",
              isChildActive ? "bg-accent/50 text-accent-foreground" : "",
            )}
            onSelect={(e) => {
              e.preventDefault();
              handleErrorClick(item.tipsMessage);
            }}
          >
            {item.icon && <item.icon className="h-4 w-4" />}
            <span>{item.name}</span>
            <MenuTipIcon tipsMessage={item.tipsMessage} />
          </DropdownMenuItem>
        )}

        {/* 子级菜单项 */}
        {item.children.map((child: MenuItem, index: number) => {
          const childDisabled = child.state === MenuItemState.DISABLED;
          const childIsActive = child.isActive || false;
          const childHasActiveDescendant = hasActiveDescendant(child);
          // 子菜单项高亮：自身激活 或 有激活的后代
          const shouldChildHighlight = childIsActive || childHasActiveDescendant;

          if (childDisabled) {
            return (
              <DropdownMenuItem
                key={generateMenuKey(child, "mobile-child", index)}
                disabled
                className={cn(
                  "pl-6 flex items-center space-x-2",
                  childHasActiveDescendant ? "bg-accent/50 text-accent-foreground" : "",
                )}
                onSelect={(e) => {
                  e.preventDefault();
                  handleErrorClick(child.tipsMessage);
                }}
              >
                {child.icon && <child.icon className={cn("h-4 w-4")} />}
                <span>{child.name}</span>
                <MenuTipIcon tipsMessage={child.tipsMessage} />
              </DropdownMenuItem>
            );
          }

          return (
            <DropdownMenuItem
              key={generateMenuKey(child, "mobile-child", index)}
              asChild
              className={cn(
                shouldChildHighlight ? "bg-accent text-accent-foreground" : "",
              )}
            >
              <Link
                to={child.path}
                params={child.params as any}
                className={cn("pl-6 flex items-center space-x-2")}
              >
                {child.icon && <child.icon className={cn("h-4 w-4")} />}
                <span>{child.name}</span>
                <MenuTipIcon tipsMessage={child.tipsMessage} />
              </Link>
            </DropdownMenuItem>
          );
        })}
      </>
    );
  }

  // 没有子菜单的情况
  if (isDisabled) {
    return (
      <DropdownMenuItem
        disabled
        className={cn("flex items-center space-x-2")}
        onSelect={(e) => {
          e.preventDefault();
          handleErrorClick(item.tipsMessage);
        }}
      >
        {item.icon && <item.icon className={cn("h-4 w-4")} />}
        <span>{item.name}</span>
        <MenuTipIcon tipsMessage={item.tipsMessage} />
      </DropdownMenuItem>
    );
  }

  return (
    <DropdownMenuItem
      asChild
      className={cn(
        shouldShowAsActive ? "bg-accent text-accent-foreground" : "",
      )}
    >
      <Link
        to={item.path}
        params={item.params as any}
        className={cn("flex items-center space-x-2")}
      >
        {item.icon && <item.icon className="h-4 w-4" />}
        <span>{item.name}</span>
        <MenuTipIcon tipsMessage={item.tipsMessage} />
      </Link>
    </DropdownMenuItem>
  );
}

// PC端菜单项组件 - 独立的选中逻辑处理

function TopNavigationMenuItem({ item }: TopNavigationMenuItemProps) {
  const hasChildren = item.children && item.children.length > 0;
  const { error } = useToast();

  const handleErrorClick = (errorMessage?: string) => {
    if (errorMessage) {
      error(errorMessage);
    }
  };

  // 递归检查是否有任何子菜单或后代菜单处于活跃状态
  const hasActiveDescendant = (menuItem: MenuItem): boolean => {
    if (menuItem.isActive) {
      return true;
    }

    if (menuItem.children && menuItem.children.length > 0) {
      return menuItem.children.some(child => hasActiveDescendant(child));
    }

    return false;
  };

  const isActive = item.isActive || false;
  const isChildActive = hasChildren && item.children?.some(child => hasActiveDescendant(child));

  // 对于有子菜单的项目：
  // - 如果父级菜单有路径，只有在没有子项激活时，才允许父项显示为激活状态
  // - 如果父级菜单没有路径（如分组菜单），当有子项激活时，父级应该显示为激活状态
  const shouldShowAsActive = hasChildren
    ? item.path
      ? isActive && !isChildActive
      : isChildActive
    : isActive;

  // 处理禁用状态
  const isDisabled = item.state === MenuItemState.DISABLED;

  // 渲染菜单项内容
  const renderMenuContent = () => {
    const content = (
      <div className={cn("flex items-center space-x-2")}>
        {item.icon && (
          <span className={cn("flex-shrink-0")}>
            <item.icon className={cn("h-4 w-4")} />
          </span>
        )}
        <span className={cn("truncate")}>{item.name}</span>
        <MenuTipIcon tipsMessage={item.tipsMessage} />
      </div>
    );

    return content;
  };

  // 如果有子菜单
  if (hasChildren && item.children && item.children.length > 0) {
    // 如果父菜单被禁用，显示为禁用状态
    if (isDisabled) {
      return (
        <Button
          variant="ghost"
          className={cn(
            "opacity-50 cursor-not-allowed",
            "text-muted-foreground hover:text-muted-foreground hover:bg-transparent",
          )}
          disabled={true}
        >
          {renderMenuContent()}
          <ChevronDown className={cn("ml-1 h-3 w-3")} />
        </Button>
      );
    }

    return (
      <DropdownMenu modal={false}>
        <DropdownMenuTrigger asChild>
          <Button
            variant="ghost"
            className={cn(
              "hover:text-primary transition-colors",
              shouldShowAsActive || isChildActive
                ? "bg-accent text-accent-foreground"
                : "text-muted-foreground",
            )}
          >
            {renderMenuContent()}
            <ChevronDown className={cn("ml-1 h-3 w-3")} />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" className={cn("space-y-0.5 py-2 min-w-[var(--radix-dropdown-menu-trigger-width)]")}>
          {item.children.map((child: MenuItem, index) => {
            const childDisabled = child.state === MenuItemState.DISABLED;

            if (childDisabled) {
              return (
                <DropdownMenuItem
                  key={generateMenuKey(child, "desktop-child", index)}
                  disabled
                  className={cn(
                    "flex items-center space-x-2",
                  )}
                  onSelect={(e) => {
                    e.preventDefault();
                    handleErrorClick(child.tipsMessage);
                  }}
                >
                  {child.icon && <child.icon className={cn("h-4 w-4")} />}
                  <span>{child.name}</span>
                  <MenuTipIcon tipsMessage={child.tipsMessage} />
                </DropdownMenuItem>
              );
            }

            return (
              <DropdownMenuItem
                key={generateMenuKey(child, "desktop-child", index)}
                asChild
                className={cn(
                  child.isActive ? "bg-accent text-accent-foreground" : "",
                )}
              >
                <Link
                  to={child.path}
                  params={child.params as any}
                  className={cn("flex items-center space-x-2")}
                >
                  {child.icon && <child.icon className={cn("h-4 w-4")} />}
                  <span>{child.name}</span>
                  <MenuTipIcon tipsMessage={child.tipsMessage} />
                </Link>
              </DropdownMenuItem>
            );
          })}
        </DropdownMenuContent>
      </DropdownMenu>
    );
  }

  // 如果没有子菜单，显示为普通链接
  if (isDisabled) {
    return (
      <Button
        variant="ghost"
        size="sm"
        className={cn(
          "opacity-50 cursor-not-allowed",
          "text-muted-foreground hover:text-muted-foreground hover:bg-transparent",
        )}
        disabled={true}
      >
        {renderMenuContent()}
      </Button>
    );
  }

  // 如果没有路径，显示为普通按钮（不可点击）
  if (!item.path) {
    return (
      <Button
        variant="ghost"
        size="sm"
        className={cn(
          "text-sm font-medium transition-colors cursor-default",
          shouldShowAsActive
            ? "bg-accent text-accent-foreground"
            : "text-muted-foreground",
        )}
        disabled={true}
      >
        {renderMenuContent()}
      </Button>
    );
  }

  return (
    <Button
      variant="ghost"
      size="sm"
      asChild
      className={cn(
        "hover:text-primary text-sm font-medium transition-colors",
        shouldShowAsActive
          ? "bg-accent text-accent-foreground"
          : "text-muted-foreground",
      )}
    >
      <Link to={item.path} params={item.params as any}>
        {renderMenuContent()}
      </Link>
    </Button>
  );
}
