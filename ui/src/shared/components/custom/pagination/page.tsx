import { Button } from "@shared/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@shared/components/ui/select";
import { useIsMobile } from "@shared/hooks/use-mobile";
import { cn } from "@shared/lib/utils";
import { ChevronLeft, ChevronRight, Loader2, RefreshCw } from "lucide-react";
import React from "react";

export interface PagePaginationProps {
  /** 当前页码（从1开始） */
  currentPage: number;
  /** 每页显示数量 */
  pageSize: number;
  /** 总记录数 */
  total: number;
  /** 是否显示总数信息 */
  showTotal?: boolean;
  /** 最多显示的页码数量 */
  showSizeCount?: number;
  /** 加载状态 */
  loading?: boolean;
  /** 页码变化回调 */
  onChange?: (page: number) => void;
  /** 刷新回调 */
  onRefresh?: () => void;
  /** 是否显示刷新按钮 */
  showRefresh?: boolean;
  /** 自定义类名 */
  className?: string;
  /** 页面大小选项，默认使用  [20, 50, 100] */
  pageSizeOptions?: readonly number[];
  /** 是否显示页面大小选择器 */
  showPageSize?: boolean;
  /** 页面大小变化回调 */
  onPageSizeChange?: (pageSize: number) => void;
}

export function PagePagination({
  currentPage,
  pageSize,
  total,
  showTotal = true,
  showSizeCount = 7,
  loading = false,
  onChange,
  onRefresh,
  showRefresh = false,
  className,
  pageSizeOptions = [20, 50, 100],
  showPageSize = true,
  onPageSizeChange,
}: PagePaginationProps) {
  const isMobile = useIsMobile();
  const totalPages = total > 0 ? Math.ceil(total / pageSize) : 1;

  // 计算显示的页码范围（用于PC端页码按钮和移动端下拉选项）
  const getPageNumbers = () => {
    const pages: (number | "ellipsis")[] = [];
    const maxShowCount = showSizeCount;

    if (totalPages <= maxShowCount) {
      // 总页数较少，显示所有页码
      for (let i = 1; i <= totalPages; i++) {
        pages.push(i);
      }
    } else {
      // 总页数较多，需要显示省略号
      const half = Math.floor(maxShowCount / 2);

      // 始终显示第一页
      pages.push(1);

      let start = Math.max(2, currentPage - half + 1);
      let end = Math.min(totalPages - 1, currentPage + half - 1);

      // 调整范围以确保显示指定数量的页码
      if (end - start + 1 < maxShowCount - 2) {
        if (start === 2) {
          end = Math.min(totalPages - 1, start + maxShowCount - 3);
        } else {
          start = Math.max(2, end - maxShowCount + 3);
        }
      }

      // 添加省略号和中间页码
      if (start > 2) {
        pages.push("ellipsis");
      }

      for (let i = start; i <= end; i++) {
        pages.push(i);
      }

      // 添加省略号和最后一页
      if (end < totalPages - 1) {
        pages.push("ellipsis");
      }

      if (totalPages > 1) {
        pages.push(totalPages);
      }
    }

    return pages;
  };

  // 获取Select下拉框应该显示的页码选项（只返回数字页码，不包含省略号）
  const getSelectPageOptions = () => {
    const pageNumbers = getPageNumbers();
    return pageNumbers.filter((page): page is number => page !== "ellipsis");
  };

  const handlePageChange = (page: number) => {
    if (page < 1 || page > totalPages || page === currentPage || loading) {
      return;
    }
    onChange?.(page);
  };

  const pageNumbers = getPageNumbers();

  // 移动端显示
  if (isMobile) {
    return (
      <div className={cn("flex flex-col gap-2 pt-2", className)}>
        {/* 分页控件 */}
        <div
          className={cn(
            "flex items-center justify-between gap-3 w-full",
            loading && "opacity-50 pointer-events-none",
          )}
        >
          {/* 上一页按钮 */}
          <Button
            variant="outline"
            size="sm"
            onClick={() => handlePageChange(currentPage - 1)}
            disabled={currentPage <= 1 || loading}
            className={cn("flex-1 flex items-center justify-center gap-1 h-9")}
            title="上一页"
          >
            <ChevronLeft className={cn("h-4 w-4")} />
            <span>上一页</span>
          </Button>

          {/* 中间：快速页码切换 */}
          <Select
            value={String(currentPage)}
            onValueChange={(value) => handlePageChange(Number(value))}
            disabled={loading}
          >
            <SelectTrigger className="h-9 w-[120px]">
              <SelectValue>
                <span className="text-xs">
                  {loading ? (
                    <Loader2 className="h-3 w-3 animate-spin [animation-duration:0.5s] inline-block" />
                  ) : (
                    <>
                      {currentPage}/{totalPages}
                      {showTotal && (
                        <span className="text-muted-foreground ml-1">
                          ·{total}条
                        </span>
                      )}
                    </>
                  )}
                </span>
              </SelectValue>
            </SelectTrigger>
            <SelectContent className="max-h-[300px]">
              {getSelectPageOptions().map((page, index, array) => {
                // 检查是否需要在当前页码前后显示省略号分隔
                const prevPage = index > 0 ? array[index - 1] : null;
                const showEllipsisBefore = prevPage !== null && page - prevPage > 1;

                return (
                  <React.Fragment key={page}>
                    {showEllipsisBefore && (
                      <div className="px-2 py-1.5 text-center text-muted-foreground text-sm">
                        ...
                      </div>
                    )}
                    <SelectItem value={String(page)}>
                      第 {page} 页
                    </SelectItem>
                  </React.Fragment>
                );
              })}
            </SelectContent>
          </Select>

          {/* 下一页按钮 */}
          <Button
            variant="outline"
            size="sm"
            onClick={() => handlePageChange(currentPage + 1)}
            disabled={currentPage >= totalPages || loading}
            className={cn("flex-1 flex items-center justify-center gap-1 h-9")}
            title="下一页"
          >
            <span>下一页</span>
            <ChevronRight className={cn("h-4 w-4")} />
          </Button>
        </div>
      </div>
    );
  }

  // PC端显示
  return (
    <div className={cn("flex items-center justify-between gap-4", className)}>
      {/* 左侧：记录信息 */}
      {showTotal && (
        <div className="text-sm text-muted-foreground">
          {loading ? (
            <div className="flex items-center gap-2">
              <Loader2 className={cn("h-4 w-4 animate-spin [animation-duration:0.5s]")} />
              <span className="hidden lg:inline">加载中</span>
            </div>
          ) : (
            <div className="flex items-center gap-1">
              <span className="hidden lg:inline">
                共 {total} 条，当前第 {currentPage} 页，共 {totalPages} 页
              </span>
              <span className="lg:hidden">
                {total}条·第{currentPage}/{totalPages}页
              </span>
            </div>
          )}
        </div>
      )}

      {/* 右侧：分页控件 */}
      <div
        className={cn(
          "flex items-center gap-2",
          loading && "opacity-50 pointer-events-none",
        )}
      >
        {/* 页面大小选择器 */}
        {showPageSize && onPageSizeChange && (
          <div className="flex items-center gap-2">
            <span className="hidden lg:inline text-sm text-muted-foreground whitespace-nowrap">每页</span>
            <Select
              value={String(pageSize)}
              onValueChange={(value) => onPageSizeChange(Number(value))}
              disabled={loading || total < Math.min(...pageSizeOptions)}
            >
              <SelectTrigger className={cn("h-8 w-16")}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent className="max-h-[300px]">
                {pageSizeOptions.map((size) => (
                  <SelectItem
                    key={size}
                    value={String(size)}
                    disabled={total < size}
                  >
                    {size}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <span className="hidden lg:inline text-sm text-muted-foreground whitespace-nowrap">条</span>
          </div>
        )}

        {/* 上一页按钮 */}
        <Button
          variant="outline"
          size="sm"
          onClick={() => handlePageChange(currentPage - 1)}
          disabled={currentPage <= 1 || loading}
          className={cn("flex items-center gap-1 h-8 lg:px-3 px-2")}
          title="上一页"
        >
          <ChevronLeft className={cn("h-4 w-4")} />
          <span className="hidden lg:inline">上一页</span>
        </Button>

        {/* 页码按钮 */}
        <div className="flex items-center gap-1">
          {pageNumbers.map((page, index) => {
            if (page === "ellipsis") {
              return (
                <span
                  key={`ellipsis-${index}`}
                  className="flex items-center justify-center text-muted-foreground h-8 w-8"
                >
                  ...
                </span>
              );
            }

            return (
              <Button
                key={page}
                variant={currentPage === page ? "default" : "outline"}
                size="sm"
                onClick={() => handlePageChange(page)}
                disabled={loading}
                className={cn("p-0 h-8 w-8")}
              >
                {page}
              </Button>
            );
          })}
        </div>

        {/* 下一页按钮 */}
        <Button
          variant="outline"
          size="sm"
          onClick={() => handlePageChange(currentPage + 1)}
          disabled={currentPage >= totalPages || loading}
          className={cn("flex items-center gap-1 h-8 lg:px-3 px-2")}
          title="下一页"
        >
          <span className="hidden lg:inline">下一页</span>
          <ChevronRight className={cn("h-4 w-4")} />
        </Button>

        {/* 刷新按钮 */}
        {showRefresh && onRefresh && (
          <Button
            variant="outline"
            size="sm"
            onClick={onRefresh}
            disabled={loading}
            className={cn("h-8 w-8 p-0")}
            title="刷新数据"
          >
            <RefreshCw className={cn("h-4 w-4", loading && "animate-spin")} />
          </Button>
        )}
      </div>
    </div>
  );
}
