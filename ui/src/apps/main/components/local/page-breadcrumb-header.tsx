

import {
  Breadcrumb,
  BreadcrumbEllipsis,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@shared/components/ui/breadcrumb"
import { Button } from '@shared/components/ui/button'
import { Popover, PopoverContent, PopoverTrigger } from '@shared/components/ui/popover'
import { Separator } from '@shared/components/ui/separator'
import { Sheet, SheetClose, SheetContent, SheetTrigger } from '@shared/components/ui/sheet'
import { Tooltip, TooltipContent, TooltipTrigger } from '@shared/components/ui/tooltip'
import { useMobileMenu } from '@apps/main/contexts/mobile-menu-context'
import { useIsMobile } from '@shared/hooks/use-mobile'
import { cn } from '@shared/lib/utils'
import React from 'react'
import { SmartSidebarTrigger } from './smart-sidebar-trigger'

export interface BreadcrumbNode {
  name: string
  href?: string
  onClick?: () => void
}

interface BreadcrumbContentProps {
  node: BreadcrumbNode
  /** 是否是第一个面包屑项 */
  isFirst?: boolean
  /** 移动端点击第一个面包屑时的回调 */
  onMobileClick?: () => void
}

const BreadcrumbContent = ({ node, isFirst, onMobileClick }: BreadcrumbContentProps) => {
  const isMobile = useIsMobile()
  
  // 链接样式：显眼的颜色
  const linkClassName = "text-foreground font-medium hover:text-primary"
  
  // 移动端的第一个面包屑：点击打开菜单
  if (isFirst && isMobile && onMobileClick) {
    return (
      <BreadcrumbLink
        className={cn("cursor-pointer", linkClassName)}
        onClick={(e) => {
          e.preventDefault()
          onMobileClick()
        }}
      >
        {node.name}
      </BreadcrumbLink>
    )
  }
  
  if (node.href) {
    return <BreadcrumbLink href={node.href} className={linkClassName}>{node.name}</BreadcrumbLink>
  }
  if (node.onClick) {
    return (
      <BreadcrumbLink
        className={cn("cursor-pointer", linkClassName)}
        onClick={(e) => {
          e.preventDefault()
          node.onClick?.()
        }}
      >
        {node.name}
      </BreadcrumbLink>
    )
  }
  // 普通文本：平淡的颜色
  return <BreadcrumbPage className="font-normal text-muted-foreground">{node.name}</BreadcrumbPage>
}


export interface PageBreadcrumbHeaderProps extends React.HTMLAttributes<HTMLElement> {
  breadcrumbData: BreadcrumbNode[]
  /** 是否折叠中间的面包屑项（默认 true，显示省略号；false 则展开所有项） */
  collapseMiddle?: boolean
}

export const PageBreadcrumbHeader = ({
  breadcrumbData,
  collapseMiddle = true,
  className,
  ...props
}: PageBreadcrumbHeaderProps) => {
  const items = breadcrumbData ?? []
  const first = items[0]
  const last = items[items.length - 1]
  const middle = items.length > 2 ? items.slice(1, items.length - 1) : []
  const { setOpen: openMobileMenu } = useMobileMenu()

  return (
    <header
      className={cn(
        'bg-background flex h-16 items-center gap-3 p-4 sm:gap-4 transition-all duration-200 ease-linear',
        'border-b border-border/40',
        className
      )}
      {...props}
    >
      <SmartSidebarTrigger
        variant='outline'
        className={cn('hidden md:flex scale-125 sm:scale-100 transition-transform duration-200')}
      />
      {items.length > 0 && (
        <>
          <Separator orientation="vertical" className="hidden md:block" />
          <Breadcrumb className={cn('flex-nowrap')}>
            <BreadcrumbList>
              {/* Render first item */}
              <BreadcrumbItem>
                <BreadcrumbContent node={first} isFirst onMobileClick={() => openMobileMenu(true)} />
              </BreadcrumbItem>

              {/* If we have middle items, show ellipsis on small screens and inline items on sm+ */}
              {middle.length > 0 && (
                <>
                  <BreadcrumbSeparator />
                  {collapseMiddle ? (
                    <>
                      {/* For very small screens use a bottom sheet (touch-friendly). For sm+ use a Popover. */}
                      <BreadcrumbItem className={cn("sm:hidden")}>
                        <Sheet>
                          <Tooltip>
                            <TooltipTrigger asChild>
                              <SheetTrigger asChild>
                                <Button variant="ghost" size="icon" aria-label="Show breadcrumb path">
                                  <BreadcrumbEllipsis />
                                </Button>
                              </SheetTrigger>
                            </TooltipTrigger>
                            <TooltipContent>显示路径</TooltipContent>
                          </Tooltip>
                          <SheetContent>
                            <div className="p-4">
                              <div className="mb-2 flex items-center justify-between">
                                <h3 className="text-sm font-medium">Path</h3>
                                <SheetClose asChild>
                                  <Button variant="ghost" size="sm">Close</Button>
                                </SheetClose>
                              </div>
                              <ol className="flex flex-col gap-2">
                                {middle.map((c, idx) => (
                                  <li key={`sheet-crumb-${idx}`}>
                                    <BreadcrumbContent node={c} />
                                  </li>
                                ))}
                              </ol>
                            </div>
                          </SheetContent>
                        </Sheet>
                      </BreadcrumbItem>

                      {/* Popover for sm+ */}
                      <BreadcrumbItem className={cn("hidden sm:flex")}>
                        <Popover>
                          <Tooltip>
                            <TooltipTrigger asChild>
                              <PopoverTrigger asChild>
                                <Button variant="ghost" size="icon" aria-label="Show breadcrumb path">
                                  <BreadcrumbEllipsis />
                                </Button>
                              </PopoverTrigger>
                            </TooltipTrigger>
                            <TooltipContent>显示路径</TooltipContent>
                          </Tooltip>
                          <PopoverContent className={cn("p-2")}>
                            <ol className="flex flex-col gap-1">
                              {middle.map((c, idx) => (
                                <li key={`hidden-crumb-pop-${idx}`}>
                                  <BreadcrumbContent node={c} />
                                </li>
                              ))}
                            </ol>
                          </PopoverContent>
                        </Popover>
                      </BreadcrumbItem>

                      <span className="hidden sm:inline">
                        {middle.map((c, idx) => (
                          <React.Fragment key={`crumb-group-${idx}`}>
                            <BreadcrumbSeparator />
                            <BreadcrumbItem key={`crumb-${idx}`}>
                              <BreadcrumbContent node={c} />
                            </BreadcrumbItem>
                          </React.Fragment>
                        ))}
                      </span>
                    </>
                  ) : (
                    /* 不折叠，直接展开所有中间项 */
                    middle.map((c, idx) => (
                      <React.Fragment key={`crumb-expand-${idx}`}>
                        {idx > 0 && <BreadcrumbSeparator />}
                        <BreadcrumbItem>
                          <BreadcrumbContent node={c} />
                        </BreadcrumbItem>
                      </React.Fragment>
                    ))
                  )}
                </>
              )}

              {/* Only render separator and last item when there are multiple breadcrumbs */}
              {items.length > 1 && (
                <>
                  <BreadcrumbSeparator />
                  <BreadcrumbItem>
                    <BreadcrumbContent node={last} />
                  </BreadcrumbItem>
                </>
              )}
            </BreadcrumbList>
          </Breadcrumb>
        </>
      )}
    </header>
  )
}

PageBreadcrumbHeader.displayName = 'PageBreadcrumbHeader'
