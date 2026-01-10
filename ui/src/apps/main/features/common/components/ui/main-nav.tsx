
import { AppLogo } from "@apps/main/components/local/app-logo";
import { useMobileMenu } from "@apps/main/contexts/mobile-menu-context";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { CenteredLoading } from "@shared/components/custom/page-placeholder/centered-loading";
import { Button } from "@shared/components/ui/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@shared/components/ui/collapsible";
import {
  NavigationMenu,
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  NavigationMenuTrigger,
} from "@shared/components/ui/navigation-menu";
import { Sheet, SheetContent, SheetTrigger } from "@shared/components/ui/sheet";
import { useTheme, type Theme } from "@shared/contexts/theme-context";
import { useMenu } from "@apps/main/hooks/use-menu-data";
import { useIsMobile } from "@shared/hooks/use-mobile";
import { cn, getHomeUrl } from "@shared/lib/utils";
import { Link, useLocation, useMatches } from "@tanstack/react-router";
import { ChevronDown, Menu, Monitor, Moon, Sun } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { MainNavThemeToggle } from "./main-nav-theme-toggle";
import { MainNavUserInfo, MobileUserInfo } from "./main-nav-user-info";

export function MainNavMenu() {
  const isMobile = useIsMobile();

  if (isMobile) {
    return (
      <div className="flex items-center w-full">
        <MobileNav />
      </div>
    );
  }

  return <DesktopNav />;
}

// 桌面端导航组件
function DesktopNav() {
  const location = useLocation();
  const matches = useMatches();

  // 获取用户菜单和系统菜单数据
  const { result: userMenus, isLoading: userMenusLoading, isError: userMenusError, error: userMenusErrorMsg } = useMenu("user");
  const { result: systemMenus, isLoading: systemMenusLoading, isError: systemMenusError, error: systemMenusErrorMsg } = useMenu("system");

  // 判断菜单路径是否激活
  const isMenuActive = useMemo(() => {
    return (menuPath: string | undefined, menuParams?: Record<string, any>): boolean => {
      if (!menuPath) return false;

      // 移除查询参数进行路径比较
      const menuPathWithoutQuery = menuPath.split('?')[0];
      const currentPath = location.pathname;

      // 1. 精确路径匹配（忽略查询参数）
      if (menuPathWithoutQuery === currentPath) {
        return true;
      }

      // 2. 使用 useMatches 检查是否有匹配的路由
      for (const match of matches) {
        // 对于参数化路径 (如 $appId)
        if (menuPathWithoutQuery.includes('$')) {
          // 将 $appId 等参数转换为正则表达式模式
          const pathPattern = menuPathWithoutQuery.replace(/\$\w+/g, '[^/]+');
          const regex = new RegExp(`^${pathPattern}$`);

          // 检查是否匹配当前路径
          if (regex.test(match.pathname)) {
            return true;
          }

          // 检查是否是嵌套路由 - 当前路径以菜单路径开头
          const nestedRegex = new RegExp(`^${pathPattern}/`);
          if (nestedRegex.test(match.pathname)) {
            return true;
          }
        } else {
          // 非参数化路径的直接匹配
          if (match.pathname === menuPathWithoutQuery) {
            return true;
          }

          // 前缀匹配 - 用于嵌套路由
          if (match.pathname.startsWith(menuPathWithoutQuery + '/')) {
            return true;
          }
        }
      }

      return false;
    };
  }, [location.pathname, matches]);

  // 判断是否在用户模块下（以 /user/ 开头的路径）
  const isInUserModule = useMemo(() => {
    return location.pathname.startsWith('/user/') || location.pathname === '/user';
  }, [location.pathname]);

  // 判断是否在管理模块下（以 /admin/ 开头的路径）
  const isInAdminModule = useMemo(() => {
    return location.pathname.startsWith('/admin/') || location.pathname === '/admin';
  }, [location.pathname]);

  // 判断菜单组是否有激活的子项
  const isGroupActive = useMemo(() => {
    return (children?: Array<{ path?: string; params?: Record<string, any> }>): boolean => {
      if (!children) return false;

      // 首先检查子项是否有精确匹配
      if (children.some(child => isMenuActive(child.path, child.params))) {
        return true;
      }

      // 检查当前路径是否属于某个子项的模块
      // 例如: /user/app/1/features-mail/list 应该匹配 /user/app/list 所在的模块
      for (const child of children) {
        if (!child.path) continue;
        const childPathWithoutQuery = child.path.split('?')[0];

        // 获取路径的前两段作为模块前缀 (如 /user/app)
        const pathParts = childPathWithoutQuery.split('/').filter(Boolean);
        if (pathParts.length >= 2) {
          const modulePrefix = '/' + pathParts.slice(0, 2).join('/');
          if (location.pathname.startsWith(modulePrefix + '/')) {
            return true;
          }
        }
      }

      return false;
    };
  }, [isMenuActive, location.pathname]);

  return (
    <div className="flex items-center gap-4 px-4 py-2">
      {/* Logo - 只保留图标 */}
      <a href={getHomeUrl()}
        className="flex items-center hover:opacity-80 transition-opacity duration-200"
        title="lsys 管理系统"
      >
        <div className="app-logo-wrapper flex items-center justify-center h-8 w-8 rounded-lg bg-primary/10 hover:bg-primary/15 transition-colors duration-200">
          <AppLogo alt="lsys Logo" className="app-logo h-5 w-5 object-contain" />
        </div>
      </a>

      {/* Navigation Menu */}
      <NavigationMenu
        className={cn("justify-start min-w-0")}
        delayDuration={0}
        skipDelayDuration={0}
      >
        <NavigationMenuList className={cn("justify-start")}>          {/* 用户菜单加载中或错误状态 */}
          {(userMenusLoading || userMenusError) && (
            <NavigationMenuItem>
              {userMenusLoading ? (
                <CenteredLoading variant="content" iconSize="xs" className="px-3 py-2" />
              ) : (
                <CenteredError variant="inline" error={userMenusErrorMsg || "加载用户菜单失败"} />
              )}
            </NavigationMenuItem>
          )}

          {/* 系统菜单加载中或错误状态 */}
          {(systemMenusLoading || systemMenusError) && (
            <NavigationMenuItem>
              {systemMenusLoading ? (
                <CenteredLoading variant="content" iconSize="xs" className="px-3 py-2" />
              ) : (
                <CenteredError variant="inline" error={systemMenusErrorMsg || "加载系统菜单失败"} />
              )}
            </NavigationMenuItem>
          )}
          {/* 合并首页和用户中心 */}
          {!userMenusLoading && !userMenusError && (
            <NavigationMenuItem>
              <NavigationMenuTrigger

                className={cn(
                  "px-3",
                  (location.pathname === '/' || isInUserModule) && "bg-accent"
                )}
              >
                用户中心
              </NavigationMenuTrigger>
              <NavigationMenuContent>
                <div className="w-full sm:w-[480px] lg:w-[520px] p-4">
                  <div className="grid gap-4 grid-cols-[180px_1fr]">
                    {/* 左侧 - 首页 */}
                    <NavigationMenuLink asChild>

                      <a href={getHomeUrl()}
                        className={cn(
                          "group relative flex h-full select-none flex-col justify-center rounded-lg p-5 no-underline outline-none",
                          "border border-border/50 bg-card",
                          "hover:border-primary/30 hover:bg-accent/50 hover:shadow-sm transition-all duration-200",
                          (userMenus.length==1?"py-4 px-3":"")
                        )}
                      >
                        <div className={cn("flex items-center gap-3 mb-2" ,(userMenus.length==1?"gap-1 mb-0":""))}>
                          <AppLogo alt="Logo" className={
                            cn(
                              " drop-shadow-sm",
                            (userMenus.length==1?"h-3":"h-8 w-8 object-contain")
                          )} />
                          <div className={cn((userMenus.length==1?"text-sm truncate":"text-lg  font-bold"))}>首页</div>
                        </div>
                        <p className={cn("text-sm leading-tight text-muted-foreground"
                          ,(userMenus.length==1?"text-xs truncate":"")
                        )}>
                          返回系统首页
                        </p>
                      </a>
                    </NavigationMenuLink>

                    {/* 右侧 */}
                    <div className="space-y-3 min-w-0">
                      {userMenus.map((group) => {
                        const firstChild = group.children?.[0];
                        if (!firstChild?.path) return null;

                        // 将所有子菜单名称用逗号连接
                        const childrenNames = group.children?.map(child => child.name).join('、') || '';
                        const isActive = isGroupActive(group.children);

                        return (
                          <NavigationMenuLink key={group.name} asChild>
                            <Link
                              to={firstChild.path}
                              params={firstChild.params}
                              className={cn(
                                "block select-none rounded-lg p-3 no-underline outline-none min-w-0",
                                "border border-border/50 bg-card",
                                "hover:border-primary/30 hover:bg-accent/50 hover:shadow-sm transition-all duration-200",
                                isActive && "border-primary bg-accent"
                              )}
                            >
                              <div className="text-sm font-semibold mb-1 truncate">{group.name}</div>
                              <p className="text-xs text-muted-foreground truncate" title={childrenNames}>{childrenNames}</p>
                            </Link>
                          </NavigationMenuLink>
                        );
                      })}
                    </div>
                  </div>
                </div>
              </NavigationMenuContent>
            </NavigationMenuItem>
          )}

          {/* 系统管理 */}
          {!systemMenusLoading && !systemMenusError && systemMenus.length > 0 && (
            <NavigationMenuItem>
              <NavigationMenuTrigger
                className={cn(
                  "px-3",
                  isInAdminModule && "bg-accent"
                )}
              >
                系统管理
              </NavigationMenuTrigger>
              <NavigationMenuContent>
                <div className="w-full sm:w-[500px] lg:w-[550px] p-4">
                  <div className="grid gap-4 sm:grid-cols-2">
                    {/* 左侧 */}
                    <div className="space-y-3 min-w-0">
                      {systemMenus.slice(0, Math.ceil(systemMenus.length / 2)).map((menu) => {
                        // 处理有子菜单的组
                        if (menu.children && menu.children.length > 0) {
                          const firstChild = menu.children[0];
                          if (!firstChild?.path) return null;

                          // 将所有子菜单名称用逗号连接
                          const childrenNames = menu.children.map(child => child.name).join('、');
                          const isActive = isGroupActive(menu.children);

                          return (
                            <NavigationMenuLink key={menu.name} asChild>
                              <Link
                                to={firstChild.path}
                                params={firstChild.params}
                                className={cn(
                                  "block select-none rounded-lg p-2.5 no-underline outline-none min-w-0",
                                  "border border-border/50 bg-card",
                                  "hover:border-primary/30 hover:bg-accent/50 hover:shadow-sm transition-all duration-200",
                                  isActive && "border-primary bg-accent"
                                )}
                              >
                                <div className="text-sm font-semibold mb-1 truncate">{menu.name}</div>
                                <p className="text-xs text-muted-foreground truncate" title={childrenNames}>{childrenNames}</p>
                              </Link>
                            </NavigationMenuLink>
                          );
                        }

                        // 处理没有子菜单的独立项
                        if (menu.path) {
                          const isActive = isMenuActive(menu.path, menu.params);

                          return (
                            <NavigationMenuLink key={menu.name} asChild>
                              <Link
                                to={menu.path}
                                params={menu.params}
                                className={cn(
                                  "block select-none rounded-lg p-2.5 no-underline outline-none min-w-0",
                                  "border border-border/50 bg-card",
                                  "hover:border-primary/30 hover:bg-accent/50 hover:shadow-sm transition-all duration-200",
                                  isActive && "border-primary bg-accent"
                                )}
                              >
                                <div className="text-sm font-semibold mb-1 truncate">{menu.name}</div>
                                <p className="text-xs text-muted-foreground truncate">{menu.name}</p>
                              </Link>
                            </NavigationMenuLink>
                          );
                        }

                        return null;
                      })}
                    </div>

                    {/* 右侧 */}
                    <div className="space-y-3 min-w-0">
                      {systemMenus.slice(Math.ceil(systemMenus.length / 2)).map((menu) => {
                        // 处理有子菜单的组
                        if (menu.children && menu.children.length > 0) {
                          const firstChild = menu.children[0];
                          if (!firstChild?.path) return null;

                          // 将所有子菜单名称用逗号连接
                          const childrenNames = menu.children.map(child => child.name).join('、');
                          const isActive = isGroupActive(menu.children);

                          return (
                            <NavigationMenuLink key={menu.name} asChild>
                              <Link
                                to={firstChild.path}
                                params={firstChild.params}
                                className={cn(
                                  "block select-none rounded-lg p-2.5 no-underline outline-none min-w-0",
                                  "border border-border/50 bg-card",
                                  "hover:border-primary/30 hover:bg-accent/50 hover:shadow-sm transition-all duration-200",
                                  isActive && "border-primary bg-accent"
                                )}
                              >
                                <div className="text-sm font-semibold mb-1 truncate">{menu.name}</div>
                                <p className="text-xs text-muted-foreground truncate" title={childrenNames}>{childrenNames}</p>
                              </Link>
                            </NavigationMenuLink>
                          );
                        }

                        // 处理没有子菜单的独立项
                        if (menu.path) {
                          const isActive = isMenuActive(menu.path, menu.params);

                          return (
                            <NavigationMenuLink key={menu.name} asChild>
                              <Link
                                to={menu.path}
                                params={menu.params}
                                className={cn(
                                  "block select-none rounded-lg p-2.5 no-underline outline-none min-w-0",
                                  "border border-border/50 bg-card",
                                  "hover:border-primary/30 hover:bg-accent/50 hover:shadow-sm transition-all duration-200",
                                  isActive && "border-primary bg-accent"
                                )}
                              >
                                <div className="text-sm font-semibold mb-1 truncate">{menu.name}</div>
                                <p className="text-xs text-muted-foreground truncate">{menu.name}</p>
                              </Link>
                            </NavigationMenuLink>
                          );
                        }

                        return null;
                      })}
                    </div>
                  </div>
                </div>
              </NavigationMenuContent>
            </NavigationMenuItem>
          )}
        </NavigationMenuList>
      </NavigationMenu>
    </div>
  )
}

// 移动端导航组件
function MobileNav() {
  const { theme, setTheme } = useTheme();
  const location = useLocation();
  const matches = useMatches();
  const { isOpen: open, setOpen } = useMobileMenu();

  // 判断当前是否在管理模块
  const isAdminModule = useMemo(() => {
    return matches.some(match => match.pathname.startsWith('/admin'));
  }, [matches]);

  // 用户中心和系统管理的独立展开状态
  const [userMenuOpen, setUserMenuOpen] = useState(!isAdminModule);
  const [systemMenuOpen, setSystemMenuOpen] = useState(isAdminModule);

  // 当模块切换时，更新菜单展开状态
  useEffect(() => {
    setUserMenuOpen(!isAdminModule);
    setSystemMenuOpen(isAdminModule);
  }, [isAdminModule]);

  // 获取用户菜单和系统菜单数据
  const { result: userMenus, isLoading: userMenusLoading, isError: userMenusError, error: userMenusErrorMsg } = useMenu("user");
  const { result: systemMenus, isLoading: systemMenusLoading, isError: systemMenusError, error: systemMenusErrorMsg } = useMenu("system");

  // 判断菜单路径是否激活（移动端专用，包含同级路径匹配逻辑）
  const isMenuItemActive = useMemo(() => {
    return (menuPath: string | undefined, menuParams?: Record<string, any>): boolean => {
      if (!menuPath) return false;

      // 移除查询参数进行路径比较
      const menuPathWithoutQuery = menuPath.split('?')[0];
      const currentPath = location.pathname;

      // 1. 精确路径匹配（忽略查询参数）
      if (menuPathWithoutQuery === currentPath) {
        return true;
      }

      // 2. 使用 useMatches 检查是否有匹配的路由
      for (const match of matches) {
        // 对于参数化路径 (如 $appId)
        if (menuPathWithoutQuery.includes('$')) {
          // 将 $appId 等参数转换为正则表达式模式
          const pathPattern = menuPathWithoutQuery.replace(/\$\w+/g, '[^/]+');
          const regex = new RegExp(`^${pathPattern}$`);

          // 检查是否匹配当前路径
          if (regex.test(match.pathname)) {
            return true;
          }

          // 检查是否是嵌套路由 - 当前路径以菜单路径开头
          const nestedRegex = new RegExp(`^${pathPattern}/`);
          if (nestedRegex.test(match.pathname)) {
            return true;
          }
        } else {
          // 非参数化路径的直接匹配
          if (match.pathname === menuPathWithoutQuery) {
            return true;
          }

          // 前缀匹配 - 用于嵌套路由
          if (match.pathname.startsWith(menuPathWithoutQuery + '/')) {
            return true;
          }

          // 同级路径匹配 - 用于同一父路径下的不同子页面（移动端专用）
          // 例如: /admin/email/send-config/channel 和 /admin/email/send-config/template
          // 只在当前访问的完整路径（currentPath）与菜单路径有相同的父路径时才匹配
          // 不使用 match.pathname（可能是中间路由）来避免误匹配
          const menuParentPath = menuPathWithoutQuery.substring(0, menuPathWithoutQuery.lastIndexOf('/'));
          const currentParentPath = currentPath.substring(0, currentPath.lastIndexOf('/'));

          // 硬编码排除：某些模块下的菜单项不应该互相匹配
          // /user/app 模块：create、list 和详情页面不应该互相匹配
          // /user/account 模块：各个账号管理子页面不应该互相匹配
          // /admin/app 模块：request、list 不应该互相匹配
          // /admin/email、/admin/sms、/admin/rbac、/admin/user 模块：各个子页面不应该互相匹配
          const excludeParentPaths = [
            '/user/app', '/user/account',
            '/admin/app', '/admin/email', '/admin/sms', '/admin/rbac', '/admin/user'
          ];
          const shouldExclude = excludeParentPaths.includes(menuParentPath);

          if (menuParentPath && menuParentPath === currentParentPath && menuParentPath !== menuPathWithoutQuery && !shouldExclude) {
            return true;
          }

          // 模块前缀匹配 - 仅用于列表页到详情页的匹配（移动端专用）
          // 例如: /user/app/list 应该匹配 /user/app/1/service/oauth-client
          // 但 /user/app/create 不应该匹配
          const pathParts = menuPathWithoutQuery.split('/').filter(Boolean);
          if (pathParts.length >= 3) {
            const lastSegment = pathParts[pathParts.length - 1];
            // 只有当菜单路径最后一段是 'list' 时才应用模块前缀匹配
            if (lastSegment === 'list') {
              const modulePrefix = '/' + pathParts.slice(0, 2).join('/');
              if (match.pathname.startsWith(modulePrefix + '/')) {
                // 检查当前路径是否是带参数的嵌套路径（如 /user/app/1/...）
                const remainingPath = match.pathname.substring(modulePrefix.length + 1);
                const nextSegment = remainingPath.split('/')[0];
                // 如果下一段是数字（appId），则认为是同一模块
                if (/^\d+$/.test(nextSegment)) {
                  return true;
                }
              }
            }
          }
        }
      }

      return false;
    };
  }, [location.pathname, matches]);

  // 判断菜单组是否有激活的子项（用于高亮整个模块）
  const isGroupActive = useMemo(() => {
    return (children?: Array<{ path?: string; params?: Record<string, any> }>): boolean => {
      if (!children) return false;

      // 首先检查子项是否有精确匹配
      if (children.some(child => isMenuItemActive(child.path, child.params))) {
        return true;
      }

      // 检查当前路径是否属于某个子项的模块
      // 例如: /user/app/1/features-mail/list 应该匹配 /user/app/list 所在的模块
      for (const child of children) {
        if (!child.path) continue;
        const childPathWithoutQuery = child.path.split('?')[0];

        // 获取路径段
        const pathParts = childPathWithoutQuery.split('/').filter(Boolean);
        
        // 对于三段及以上路径（如 /admin/rbac/role），检查前两段是否相同
        // 这样可以匹配同一模块下的其他页面（如 /admin/rbac/resource）
        if (pathParts.length >= 3) {
          const currentPathParts = location.pathname.split('/').filter(Boolean);
          if (currentPathParts.length >= 3) {
            // 检查前两段是否相同（如 /admin/rbac）
            if (pathParts[0] === currentPathParts[0] && pathParts[1] === currentPathParts[1]) {
              return true;
            }
          }
        }

        // 对于两段路径，使用前两段作为模块前缀
        if (pathParts.length >= 2) {
          const modulePrefix = '/' + pathParts.slice(0, 2).join('/');
          if (location.pathname.startsWith(modulePrefix + '/')) {
            return true;
          }
        }
      }

      return false;
    };
  }, [isMenuItemActive, location.pathname]);

  return (
    <>
      {/* 菜单按钮 - 左侧 */}
      <div className="flex-shrink-0">
        <Sheet open={open} onOpenChange={setOpen}>
          <SheetTrigger asChild>
            <Button variant="ghost" size="icon" className={cn("h-9 w-9 ml-2")}>
              <Menu className="h-5 w-5" />
              <span className="sr-only">打开菜单</span>
            </Button>
          </SheetTrigger>
          <SheetContent side="left" className={cn("w-[280px] sm:w-[320px] p-0 [&>button]:hidden")}>
            <div className="flex flex-col h-full">
              {/* Header */}
              <a
                href={getHomeUrl()}
                onClick={() => setOpen(false)}
                className="flex items-center gap-3 p-4 border-b hover:bg-accent/50 transition-colors"
              >
                <div className="app-logo-wrapper flex items-center justify-center h-10 w-10 rounded-lg bg-primary/10">
                  <AppLogo alt="lsys Logo" className="app-logo h-6 w-6 object-contain" />
                </div>
                <div className="flex-1 min-w-0">
                  <div className="font-semibold text-sm truncate">lsys 管理系统</div>
                  <div className="text-xs text-muted-foreground truncate">统一身份认证和应用管理</div>
                </div>
              </a>

              {/* Menu Items */}
              <nav className="flex-1 overflow-y-auto px-3 py-3">
                {/* 用户中心 */}
                {userMenusLoading ? (
                  <div className="mb-6">
                    <CenteredLoading variant="content" iconSize="xs" className="px-3 py-2" />
                  </div>
                ) : userMenusError ? (
                  <div className="mb-6">
                    <div className="flex items-center gap-2 px-3 py-2 mb-2">
                      <div className="h-5 w-1 bg-primary rounded-full" />
                      <div className="text-sm font-bold text-foreground">用户中心</div>
                    </div>
                    <div className="px-3 py-2">
                      <CenteredError variant="inline" error={userMenusErrorMsg} />
                    </div>
                  </div>
                ) : userMenus.length > 0 && (
                  <div className="mb-6">
                    <Collapsible open={userMenuOpen} onOpenChange={setUserMenuOpen}>
                      <CollapsibleTrigger asChild>
                        <button className={cn("flex items-center justify-between w-full px-3 py-2 mb-2 hover:bg-accent/50 rounded-md transition-colors")}>
                          <div className={cn("flex items-center gap-2")}>
                            <div className={cn("h-5 w-1 bg-primary rounded-full")} />
                            <div className={cn("text-sm font-bold text-foreground")}>用户中心</div>
                          </div>
                          <ChevronDown className={cn(
                            "h-4 w-4 text-muted-foreground transition-transform duration-200",
                            userMenuOpen && "transform rotate-180"
                          )} />
                        </button>
                      </CollapsibleTrigger>
                      <CollapsibleContent>
                        <div className={cn("space-y-3")}>
                          {userMenus.map((group) => {
                            const groupActive = isGroupActive(group.children);
                            const GroupIcon = group.icon;
                            return (
                              <div key={group.name} className={cn("space-y-1")}>
                                <div className={cn(
                                  "px-3 py-1 text-xs font-semibold rounded-md flex items-center gap-1.5",
                                  groupActive
                                    ? "text-primary bg-primary/10"
                                    : "text-primary/80 bg-primary/5"
                                )}>
                                  {GroupIcon && <GroupIcon className="h-3.5 w-3.5" />}
                                  {group.name}
                                </div>
                                <div className={cn("space-y-0.5")}>
                                  {group.children?.map((child) => {
                                    if (!child.path) return null;
                                    const isActive = isMenuItemActive(child.path, child.params);
                                    const ChildIcon = child.icon;

                                    return (
                                      <Link
                                        key={child.name}
                                        to={child.path}
                                        params={child.params}
                                        onClick={() => setOpen(false)}
                                        className={cn(
                                          "flex items-center gap-2 py-2 px-3 ml-3 rounded-md transition-colors text-sm",
                                          "hover:bg-accent hover:text-accent-foreground",
                                          "active:bg-accent/80",
                                          isActive && "bg-accent text-accent-foreground font-medium"
                                        )}
                                      >
                                        {ChildIcon && <ChildIcon className="h-4 w-4" />}
                                        {child.name}
                                      </Link>
                                    );
                                  })}
                                </div>
                              </div>
                            );
                          })}
                        </div>
                      </CollapsibleContent>
                    </Collapsible>
                  </div>
                )}

                {/* 系统管理 */}
                {systemMenusLoading ? (
                  <div className="mb-6">
                    <CenteredLoading variant="content" iconSize="xs" className="px-3 py-2" />
                  </div>
                ) : systemMenusError ? (
                  <div className="mb-6">
                    <div className="flex items-center gap-2 px-3 py-2 mb-2">
                      <div className="h-5 w-1 bg-primary rounded-full" />
                      <div className="text-sm font-bold text-foreground">系统管理</div>
                    </div>
                    <div className="px-3 py-2">
                      <CenteredError variant="inline" error={systemMenusErrorMsg} />
                    </div>
                  </div>
                ) : systemMenus.length > 0 && (
                  <div className={cn("mb-6 pt-4 border-t")}>
                    <Collapsible open={systemMenuOpen} onOpenChange={setSystemMenuOpen}>
                      <CollapsibleTrigger asChild>
                        <button className={cn("flex items-center justify-between w-full px-3 py-2 mb-2 hover:bg-accent/50 rounded-md transition-colors")}>
                          <div className={cn("flex items-center gap-2")}>
                            <div className={cn("h-5 w-1 bg-primary rounded-full")} />
                            <div className={cn("text-sm font-bold text-foreground")}>系统管理</div>
                          </div>
                          <ChevronDown className={cn(
                            "h-4 w-4 text-muted-foreground transition-transform duration-200",
                            systemMenuOpen && "transform rotate-180"
                          )} />
                        </button>
                      </CollapsibleTrigger>
                      <CollapsibleContent>
                        <div className={cn("space-y-3")}>
                          {systemMenus.map((menu) => {
                            // 处理有子菜单的组
                            if (menu.children && menu.children.length > 0) {
                              const groupActive = isGroupActive(menu.children);
                              const MenuIcon = menu.icon;
                              return (
                                <div key={menu.name} className={cn("space-y-1")}>
                                  <div className={cn(
                                    "px-3 py-1 text-xs font-semibold rounded-md flex items-center gap-1.5",
                                    groupActive
                                      ? "text-primary bg-primary/10"
                                      : "text-primary/80 bg-primary/5"
                                  )}>
                                    {MenuIcon && <MenuIcon className="h-3.5 w-3.5" />}
                                    {menu.name}
                                  </div>
                                  <div className={cn("space-y-0.5")}>
                                    {menu.children.map((child) => {
                                      if (!child.path) return null;
                                      const isActive = isMenuItemActive(child.path, child.params);
                                      const ChildIcon = child.icon;

                                      return (
                                        <Link
                                          key={child.name}
                                          to={child.path}
                                          params={child.params}
                                          onClick={() => setOpen(false)}
                                          className={cn(
                                            "flex items-center gap-2 py-2 px-3 ml-3 rounded-md transition-colors text-sm",
                                            "hover:bg-accent hover:text-accent-foreground",
                                            "active:bg-accent/80",
                                            isActive && "bg-accent text-accent-foreground font-medium"
                                          )}
                                        >
                                          {ChildIcon && <ChildIcon className="h-4 w-4" />}
                                          {child.name}
                                        </Link>
                                      );
                                    })}
                                  </div>
                                </div>
                              );
                            }

                            // 处理没有子菜单的独立项
                            if (menu.path) {
                              const isActive = isMenuItemActive(menu.path, menu.params);
                              const MenuIcon = menu.icon;

                              return (
                                <Link
                                  key={menu.name}
                                  to={menu.path}
                                  params={menu.params}
                                  onClick={() => setOpen(false)}
                                  className={cn(
                                    "flex items-center gap-2 py-2 px-3 rounded-md transition-colors text-sm font-medium",
                                    "hover:bg-accent hover:text-accent-foreground",
                                    "active:bg-accent/80",
                                    isActive && "bg-accent text-accent-foreground"
                                  )}
                                >
                                  {MenuIcon && <MenuIcon className="h-4 w-4" />}
                                  {menu.name}
                                </Link>
                              );
                            }

                            return null;
                          })}
                        </div>
                      </CollapsibleContent>
                    </Collapsible>
                  </div>
                )}
              </nav>

              {/* Footer - 主题切换 */}
              <div className="border-t p-3">
                {/* 主题切换 */}
                <div className="space-y-2">
                  <div className="text-xs text-muted-foreground px-3 py-1">主题设置</div>
                  <div className="grid grid-cols-3 gap-1">
                    {[
                      { value: 'light' as Theme, label: '浅色', icon: Sun },
                      { value: 'dark' as Theme, label: '深色', icon: Moon },
                      { value: 'system' as Theme, label: '系统', icon: Monitor },
                    ].map(({ value, label, icon: Icon }) => (
                      <Button
                        key={value}
                        variant={theme === value ? "default" : "outline"}
                        size="sm"
                        className="h-9 text-xs"
                        onClick={() => setTheme(value)}
                      >
                        <Icon className="h-4 w-4 mr-1" />
                        {label}
                      </Button>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          </SheetContent>
        </Sheet>
      </div>

      {/* Logo - 绝对居中在整个导航栏 */}
      <div className="absolute left-1/2 -translate-x-1/2 pointer-events-none">
        <a
          href={getHomeUrl()}
          className="flex items-center hover:opacity-80 transition-opacity duration-200 pointer-events-auto"
          title="lsys 管理系统"
        >
          <AppLogo alt="lsys Logo" className="h-6 w-6 object-contain" />
        </a>
      </div>
    </>
  );
}


export function MainNavInfo() {
  const isMobile = useIsMobile();

  if (isMobile) {
    // 移动端只显示用户头像
    return <MobileUserInfo />;
  }

  return (
    <div className="ml-auto flex items-center gap-1 px-4 py-2 flex-shrink-0">
      <MainNavThemeToggle />
      <MainNavUserInfo />
    </div>
  )
}

