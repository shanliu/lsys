import { Button } from "@shared/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@shared/components/ui/dropdown-menu";
import { Separator } from "@shared/components/ui/separator";
import { useIsMobile } from "@shared/hooks/use-mobile";
import { cn } from "@shared/lib/utils";
import { MoreHorizontal } from "lucide-react";
import React, { Children, isValidElement } from "react";

/** 显示模式 */
export type DisplayMode = "display" | "collapsed";

/** 操作项接口 */
export interface IDataTableActionItem {
  /** 移动端显示方式 */
  mobileDisplay: DisplayMode;
  /** PC端显示方式 */
  desktopDisplay: DisplayMode;
}

/** DataTableAction 组件属性 */
export interface DataTableActionProps {
  /** 子元素必须实现 IDataTableActionItem 接口，或为 null，支持单个元素或数组 */
  children: React.ReactElement<IDataTableActionItem> | (React.ReactElement<IDataTableActionItem> | null | false)[] | null;
  /** 自定义类名 */
  className?: string;
}

/**
 * DataTableAction 组件
 * 用于管理表格操作按钮的显示方式，支持移动端和PC端不同的显示策略
 */
export function DataTableAction({ children, className }: DataTableActionProps) {
  const isMobile = useIsMobile();

  // 确保子元素是数组
  const childrenArray = Children.toArray(children).filter((child) =>
    isValidElement(child)
  ) as React.ReactElement<IDataTableActionItem>[];

  if (childrenArray.length === 0) {
    return null;
  }

  // 根据当前设备类型选择显示模式
  const displayMode = isMobile ? "mobileDisplay" : "desktopDisplay";

  // 分离直接显示和自动折叠的元素
  const directItems = childrenArray.filter(
    (child) => child.props[displayMode] === "display"
  );
  const collapsedItems = childrenArray.filter(
    (child) => child.props[displayMode] === "collapsed"
  );

  // 移动端：元素在一行显示，顶部带分隔符
  if (isMobile) {
    return (
      <div className={cn("mt-1")}>
        <Separator className="mb-3" />
        <div className={cn("flex items-center gap-1 flex-wrap", className)}>
          {directItems.map((item, index) => (
            <React.Fragment key={index}>{item}</React.Fragment>
          ))}
          {collapsedItems.length > 0 && (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="sm" className={cn("h-8 w-8 p-0")}>
                  <MoreHorizontal className="h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="p-1 space-y-1">
                {collapsedItems.map((item, index) => (
                  <DropdownMenuItem
                    key={index}
                    className="p-0 focus:bg-transparent flex [&>button]:w-full [&>button]:justify-start"
                    onSelect={(e) => {
                      e.preventDefault();
                      // 触发子元素的点击事件
                      const button = item as React.ReactElement<any>;
                      if (button.props.onClick) {
                        button.props.onClick(e);
                      }
                    }}
                  >
                    {item}
                  </DropdownMenuItem>
                ))}
              </DropdownMenuContent>
            </DropdownMenu>
          )}
        </div>
      </div>
    );
  }

  // PC端：直接显示的元素正常显示，折叠的元素显示为...按钮
  return (
    <div className={cn("flex items-center gap-2", className)}>
      {directItems.map((item, index) => (
        <React.Fragment key={index}>{item}</React.Fragment>
      ))}
      {collapsedItems.length > 0 && (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="sm" className={cn("h-8 w-8 p-0")}>
              <MoreHorizontal className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="space-y-1">
            {collapsedItems.map((item, index) => (
              <DropdownMenuItem
                key={index}
                className="p-0 focus:bg-transparent flex [&>button]:w-full [&>button]:justify-start"
                onSelect={(e) => {
                  e.preventDefault();
                  // 触发子元素的点击事件
                  const button = item as React.ReactElement<any>;
                  if (button.props.onClick) {
                    button.props.onClick(e);
                  }
                }}
              >
                {item}
              </DropdownMenuItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>
      )}
    </div>
  );
}

/**
 * DataTableActionItem 组件
 * 用于将任意组件包装为 DataTableAction 的子元素
 */
export interface DataTableActionItemProps {
  /** 子元素 */
  children: React.ReactNode;
  /** 移动端显示方式 */
  mobileDisplay: DisplayMode;
  /** PC端显示方式 */
  desktopDisplay: DisplayMode;
}

export function DataTableActionItem({
  children,
  mobileDisplay,
  desktopDisplay,
}: DataTableActionItemProps) {
  const childrenArray = Children.toArray(children).filter(isValidElement) as React.ReactElement[];
  if (childrenArray.length === 0) {
    return null;
  }
  return <>{childrenArray.map((child, index) => React.cloneElement(child, { key: index }))}</>;
}

// 添加类型标记
DataTableAction.displayName = "DataTableAction";
DataTableActionItem.displayName = "DataTableActionItem";
