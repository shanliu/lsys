import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@shared/components/ui/collapsible"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@shared/components/ui/dropdown-menu"
import { Separator } from "@shared/components/ui/separator"
import {
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu as SidebarMenuPrimitive,
  SidebarMenuAction,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
  useSidebar
} from "@shared/components/ui/sidebar"
import { Skeleton } from "@shared/components/ui/skeleton"
import { cn } from "@shared/lib/utils"
import { generateMenuKey } from "@apps/main/lib/menu-utils"
import { MenuItem, MenuItemState } from "@apps/main/types/ui-menu"
import { MenuTipIcon } from "@apps/main/components/text/menu-tip-icon"
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error"
import { Link } from "@tanstack/react-router"
import { ChevronRight, X } from "lucide-react"
import { useEffect, useState } from "react"

export interface SidebarMenuSkeletonProps {
  className?: string
  count?: number
}

export function SidebarMenuSkeleton({
  className,
  count = 5
}: SidebarMenuSkeletonProps) {
  return (
    <SidebarGroup className={className}>
      <SidebarGroupContent>
        <div className="space-y-2 px-2">
          {Array.from({ length: count }).map((_, i) => (
            <Skeleton key={i} className={cn("h-8 w-full")} />
          ))}
        </div>
      </SidebarGroupContent>
    </SidebarGroup>
  )
}

export interface SidebarMenuErrorProps {
  className?: string
  error?: unknown
}

export function SidebarMenuError({
  className,
  error
}: SidebarMenuErrorProps) {
  return (
    <SidebarGroup className={cn("p-1", className)}>
      <SidebarGroupContent>
        <div className="p-2">
          <CenteredError error={error || "加载菜单失败"} variant="content" />
        </div>
      </SidebarGroupContent>
    </SidebarGroup>
  )
}


export interface SidebarMenuProps {
  className?: string
  groupLabel?: string
  headerContent?: React.ReactNode
  menuItems: MenuItem[]
  isItemClosable?: (item: MenuItem) => boolean
  onItemClose?: (item: MenuItem, e: React.MouseEvent) => void
}

export function SidebarMenu({
  className,
  groupLabel,
  headerContent,
  menuItems = [],
  isItemClosable,
  onItemClose
}: SidebarMenuProps) {
  return (
    <SidebarGroup className={cn(className)}>
      {groupLabel && (
        <SidebarGroupLabel
          className={cn(
            // 禁用默认的负margin和透明度过度效果
            "group-data-[collapsible=icon]:!mt-0 group-data-[collapsible=icon]:!opacity-0",
            // 使用高度和溢出隐藏来实现平滑过渡，避免占位问题
            "group-data-[collapsible=icon]:!h-0 group-data-[collapsible=icon]:!overflow-hidden",
            "group-data-[collapsible=icon]:!min-h-0 group-data-[collapsible=icon]:!py-0",
            // 添加平滑的过渡动画
            "transition-all duration-200 ease-in-out"
          )}
        >
          {groupLabel}
        </SidebarGroupLabel>
      )}
      <SidebarGroupContent>
        {headerContent && (
          <>
            {headerContent}
            <Separator />
          </>
        )}
        <SidebarMenuPrimitive className={cn("pt-2 gap-2")}>
          {menuItems.map((item, index) => (
            <SidebarMenuItemComponent
              key={generateMenuKey(item, 'sidebar-item', index)}
              item={item}
              isItemClosable={isItemClosable}
              onItemClose={onItemClose}
            />
          ))}
        </SidebarMenuPrimitive>
      </SidebarGroupContent>
    </SidebarGroup>
  )
}


interface SidebarMenuItemProps {
  item: MenuItem
  level?: number
  isItemClosable?: (item: MenuItem) => boolean
  onItemClose?: (item: MenuItem, e: React.MouseEvent) => void
}

function SidebarMenuItemComponent({ item, level = 0, isItemClosable, onItemClose }: SidebarMenuItemProps) {
  const { state, isMobile, setOpen, setOpenMobile } = useSidebar()
  const hasChildren = item.children && item.children.length > 0

  // 递归检查是否有任何子菜单或后代菜单处于活跃状态
  const hasActiveDescendant = (menuItem: MenuItem): boolean => {
    if (menuItem.isActive) {
      return true
    }

    if (menuItem.children && menuItem.children.length > 0) {
      return menuItem.children.some(child => hasActiveDescendant(child))
    }

    return false
  }

  const hasActiveChild = item.children?.some((child) => hasActiveDescendant(child))
  // 使用状态管理展开/收起
  const [isOpen, setIsOpen] = useState(hasActiveChild)

  // 当路径变化时，自动更新展开状态
  useEffect(() => {
    if (hasChildren) {
      // 如果有子菜单处于激活状态，确保展开
      if (hasActiveChild) {
        setIsOpen(true)
      }
    }
  }, [hasActiveChild, hasChildren])

  if (hasChildren) {

    // Heuristic: use a compact popover menu when sidebar is collapsed on desktop
    // and the child list is reasonably small. Otherwise expand the sidebar.
    const CHILD_POPPER_THRESHOLD = 8
    const collapsed = state === "collapsed"
    const usePopover = collapsed && !isMobile && (item.children?.length ?? 0) <= CHILD_POPPER_THRESHOLD

    // If collapsed and a large/complex group or on mobile, open the sidebar instead.
    const handleOpenSidebar = () => {
      if (isMobile) {
        setOpenMobile(true)
      } else {
        setOpen(true)
      }
    }

    // When sidebar is expanded (not collapsed), keep original collapsible behavior
    if (!collapsed) {
      return (
        <SidebarMenuItem>
          <Collapsible
            className={cn("group/collapsible")}
            open={isOpen}
            onOpenChange={setIsOpen}
          >
            {/* Split Button Pattern: If item has a path + children */}
            {item.path ? (
              <>
                <SidebarMenuButton
                  asChild
                  tooltip={item.name}
                  className={cn(
                    "overflow-hidden transition-colors duration-200",
                    "group-data-[state=collapsed]/sidebar:justify-center group-data-[state=collapsed]/sidebar:!p-1.5",
                    // Use active state if exactly matching path (optional, or rely on active descendant)
                    item.isActive && "font-medium"
                  )}
                  // If we want it to look active when children are active:
                  data-active={item.isActive}
                >
                  <Link
                    to={item.path}
                    onClick={() => {
                      if (isMobile) setOpenMobile(false)
                    }}
                  >
                    {item.icon && <item.icon className="!size-4 shrink-0" />}
                    <span className="transition-all duration-200 group-data-[state=collapsed]/sidebar:opacity-0 group-data-[state=collapsed]/sidebar:w-0 truncate">
                      {item.name}
                    </span>
                    {/* No Chevron here, it moves to SidebarMenuAction */}
                  </Link>
                </SidebarMenuButton>

                <CollapsibleTrigger asChild>
                  <SidebarMenuAction
                    className="data-[state=open]:rotate-90 transition-transform duration-200"
                    showOnHover
                  >
                    <ChevronRight className="h-4 w-4" />
                    <span className="sr-only">Toggle</span>
                  </SidebarMenuAction>
                </CollapsibleTrigger>
              </>
            ) : (
              /* Standard Pattern: Whole row is toggle */
              <CollapsibleTrigger asChild>
                <SidebarMenuButton
                  tooltip={item.name}
                  className={cn(
                    "overflow-hidden transition-colors duration-200",
                    "group-data-[state=collapsed]/sidebar:justify-center group-data-[state=collapsed]/sidebar:!p-1.5",
                    hasActiveChild && "border-l-2 border-foreground/20"
                  )}
                >
                  {item.icon && <item.icon className="!size-4 shrink-0" />}
                  <span className="transition-all duration-200 group-data-[state=collapsed]/sidebar:opacity-0 group-data-[state=collapsed]/sidebar:w-0 truncate">
                    {item.name}
                  </span>
                  <ChevronRight className={cn(
                    "ml-auto !size-4 transition-transform duration-200 shrink-0",
                    "group-data-[state=open]/collapsible:rotate-90",
                    "group-data-[state=collapsed]/sidebar:hidden"
                  )} />
                </SidebarMenuButton>
              </CollapsibleTrigger>
            )}

            <CollapsibleContent className={cn("transition-all duration-200 group-data-[state=collapsed]/sidebar:hidden")}>
              <SidebarMenuSub className={cn("mt-1")}>
                {item.children?.map((child: MenuItem, index: number) => (
                  <SidebarMenuItemComponent
                    key={generateMenuKey(child, 'sidebar-item', index)}
                    item={child}
                    level={level + 1}
                    isItemClosable={isItemClosable}
                    onItemClose={onItemClose}
                  />
                ))}
              </SidebarMenuSub>
            </CollapsibleContent>
          </Collapsible>
        </SidebarMenuItem>
      )
    }

    if (usePopover) {
      return (
        <SidebarMenuItem>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <SidebarMenuButton
                tooltip={item.name}
                className={cn(
                  "overflow-hidden transition-colors duration-200",
                  "group-data-[state=collapsed]/sidebar:justify-center group-data-[state=collapsed]/sidebar:!p-1.5",
                  hasActiveChild && "border-l-2 border-foreground/20"
                )}
              >
                {item.icon && <item.icon className="!size-4 shrink-0" />}
                <span className="transition-all duration-200 group-data-[state=collapsed]/sidebar:opacity-0 group-data-[state=collapsed]/sidebar:w-0 truncate">
                  {item.name}
                </span>
                <ChevronRight className={cn(
                  "ml-auto !size-4 transition-transform duration-200 shrink-0",
                  "group-data-[state=open]/collapsible:rotate-90",
                  "group-data-[state=collapsed]/sidebar:hidden"
                )} />
              </SidebarMenuButton>
            </DropdownMenuTrigger>
            <DropdownMenuContent sideOffset={6} align="start" className={cn("p-0 bg-popover text-popover-foreground min-w-[12rem] max-w-xs rounded-md border shadow-lg overflow-hidden")}>
              {/* Content container uses compact list styling */}
              <div className="p-1">
                {item.children?.map((child: MenuItem, index: number) => {
                  const childActive = hasActiveDescendant(child)
                  return (
                    <DropdownMenuItem
                      asChild
                      key={child.id || `sidebar-child-${child.name}-${index}`}
                      className={cn(
                        "px-0 py-0.5 rounded-sm",
                        "hover:bg-sidebar-accent/50",
                        childActive && "bg-sidebar-accent text-sidebar-accent-foreground font-medium"
                      )}
                    >
                      <Link
                        to={child.path}
                        params={child.params as any}
                        onClick={() => {
                          if (isMobile) {
                            setOpenMobile(false)
                          }
                        }}
                      >
                        <div className="flex items-center gap-2 px-3 py-1.5 w-full">
                          {child.icon && <child.icon className={cn("!size-4 shrink-0")} />}
                          <span className="text-sm truncate flex-1">{child.name}</span>
                          <MenuTipIcon tipsMessage={child.tipsMessage} />
                        </div>
                      </Link>
                    </DropdownMenuItem>
                  )
                })}
              </div>
            </DropdownMenuContent>
          </DropdownMenu>
        </SidebarMenuItem>
      )
    }

    // Fallback: expand/open sidebar so users see full group
    return (
      <SidebarMenuItem>
        <SidebarMenuButton
          tooltip={item.name}
          onClick={handleOpenSidebar}
          className={cn(
            "overflow-hidden transition-colors duration-200",
            "group-data-[state=collapsed]/sidebar:justify-center group-data-[state=collapsed]/sidebar:!p-1.5",
            hasActiveChild && "border-l-2 border-foreground/20"
          )}
        >
          {item.icon && <item.icon className="!size-4 shrink-0" />}
          <span className="transition-all duration-200 group-data-[state=collapsed]/sidebar:opacity-0 group-data-[state=collapsed]/sidebar:w-0 truncate">
            {item.name}
          </span>
          <ChevronRight className={cn(
            "ml-auto !size-4 transition-transform duration-200 shrink-0",
            "group-data-[state=open]/collapsible:rotate-90",
            "group-data-[state=collapsed]/sidebar:hidden"
          )} />
        </SidebarMenuButton>
      </SidebarMenuItem>
    )
  }

  if (level > 0) {
    const isDisabled = item.state === MenuItemState.DISABLED

    if (isDisabled) {
      return (
        <SidebarMenuSubItem>

          <SidebarMenuSubButton
            className={cn(
              "overflow-hidden cursor-not-allowed opacity-50",
              "text-muted-foreground hover:text-muted-foreground hover:bg-transparent"
            )}
          >
            {item.icon && <item.icon className="!size-4 shrink-0" />}
            <span className="transition-all duration-200 group-data-[state=collapsed]/sidebar:opacity-0 group-data-[state=collapsed]/sidebar:w-0 truncate">
              {item.name}
            </span>
            <MenuTipIcon tipsMessage={item.tipsMessage} />
          </SidebarMenuSubButton>
        </SidebarMenuSubItem >
      )
    }

    // Checking if sub-item is closable
    const showCloseButton = isItemClosable?.(item) && onItemClose

    return (
      <SidebarMenuSubItem className="relative group/sub-item">
        <SidebarMenuSubButton
          asChild
          className={cn(
            "overflow-hidden transition-colors duration-200",
            "[&[data-active=true]]:font-medium",
            showCloseButton && "pr-8"
          )}
          data-active={item.isActive}
        >
          <Link
            to={item.path?.split('?')[0] || item.path}
            search={item.path?.includes('?') ? Object.fromEntries(new URLSearchParams(item.path.split('?')[1])) as any : undefined}
            params={item.params as any}
            activeProps={{ 'data-active': true }}
            activeOptions={{
              exact: true,
              includeSearch: false,
              includeHash: false
            }}
            onClick={() => {
              if (isMobile) {
                setOpenMobile(false)
              }
            }}
          >
            {item.icon && <item.icon className="!size-4 shrink-0" />}
            <span className="transition-all duration-200 group-data-[state=collapsed]/sidebar:opacity-0 group-data-[state=collapsed]/sidebar:w-0 truncate">
              {item.name}
            </span>
            <MenuTipIcon tipsMessage={item.tipsMessage} />
          </Link>
        </SidebarMenuSubButton>

        {showCloseButton && (
          <div
            role="button"
            tabIndex={0}
            className="absolute right-1 top-1/2 -translate-y-1/2 p-1 rounded-sm opacity-0 group-hover/sub-item:opacity-100 transition-opacity hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus:opacity-100 focus:outline-none z-10"
            onClick={(e) => {
              e.preventDefault()
              e.stopPropagation()
              onItemClose!(item, e)
            }}
          >
            <X className="h-3.5 w-3.5" />
          </div>
        )}
      </SidebarMenuSubItem>
    )
  }

  const isDisabled = item.state === MenuItemState.DISABLED

  if (isDisabled) {
    return (
      <SidebarMenuItem>
        <SidebarMenuButton
          tooltip={item.name}
          className={cn(
            "overflow-hidden cursor-not-allowed opacity-50",
            "text-muted-foreground hover:text-muted-foreground hover:bg-transparent",
            "group-data-[state=collapsed]/sidebar:justify-center group-data-[state=collapsed]/sidebar:!p-1.5"
          )}
        >
          {item.icon && <item.icon className="!size-4 shrink-0" />}
          <span className="transition-all duration-200 group-data-[state=collapsed]/sidebar:opacity-0 group-data-[state=collapsed]/sidebar:w-0 truncate">
            {item.name}
          </span>
          <MenuTipIcon tipsMessage={item.tipsMessage} />
        </SidebarMenuButton>
      </SidebarMenuItem>
    )
  }

  // 正常状态的顶级菜单项
  const isActive = item.isActive || false
  const showCloseButton = isItemClosable?.(item) && onItemClose

  return (
    <SidebarMenuItem className="relative group/item">
      <SidebarMenuButton
        asChild
        tooltip={item.name}
        className={cn(
          "overflow-hidden transition-colors duration-200",
          "group-data-[state=collapsed]/sidebar:justify-center group-data-[state=collapsed]/sidebar:!p-1.5",
          // 选中时只加粗字体，不要背景
          "[&[data-active=true]]:font-medium",
          showCloseButton && "pr-8"
        )}
        data-active={isActive}
      >
        <Link
          to={item.path?.split('?')[0] || item.path}
          search={item.path?.includes('?') ? Object.fromEntries(new URLSearchParams(item.path.split('?')[1])) as any : undefined}
          params={item.params as any}
          activeProps={{ 'data-active': true }}
          activeOptions={{
            exact: true,
            includeSearch: false,
            includeHash: false
          }}
          onClick={() => {
            if (isMobile) {
              setOpenMobile(false)
            }
          }}
        >
          {item.icon && <item.icon className="!size-4 shrink-0" />}
          <span className="transition-all duration-200 group-data-[state=collapsed]/sidebar:opacity-0 group-data-[state=collapsed]/sidebar:w-0 truncate">
            {item.name}
          </span>
          <MenuTipIcon tipsMessage={item.tipsMessage} />
        </Link>
      </SidebarMenuButton>

      {showCloseButton && (
        <div
          role="button"
          tabIndex={0}
          className="absolute right-1 top-1/2 -translate-y-1/2 p-1 rounded-sm opacity-0 group-hover/item:opacity-100 transition-opacity hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus:opacity-100 focus:outline-none z-10"
          onClick={(e) => {
            e.preventDefault()
            e.stopPropagation()
            onItemClose!(item, e)
          }}
        >
          <X className="h-3.5 w-3.5" />
        </div>
      )}
    </SidebarMenuItem>
  )
}
