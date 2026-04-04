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
import type { CursorPageResponseType } from "@shared/lib/utils/tools-utils";
import type { TotalRecrodResType } from "@shared/types/base-schema";
import { ChevronLeft, ChevronRight, Loader2, RefreshCw } from "lucide-react";

export interface CursorPaginationProps {
  /** 每页限制数量 */
  limit: number;
  /** 游标数据（自动计算分页状态） */
  cursorData: CursorPageResponseType | null;
  /** 搜索导航函数（自动处理分页跳转） */
  searchGo: (param: { pos: number | null; forward?: boolean } & Record<string, any>) => void;
  /** 总记录数信息（含 exact / over） */
  totalInfo?: TotalRecrodResType | null;
  /** 当前页实际记录数 */
  currentPageSize: number;
  /** 加载状态 */
  loading?: boolean;
  /** 刷新回调 */
  onRefresh?: () => void;
  /** 是否显示刷新按钮 */
  showRefresh?: boolean;
  /** 自定义样式类名 */
  className?: string;
  /** 页面大小选项，默认使用  [20, 50, 100] */
  pageSizeOptions?: readonly number[];
  /** 是否显示页面大小选择器 */
  showPageSize?: boolean;
  /** 页面大小变化回调 */
  onPageSizeChange?: (pageSize: number) => void;
}

export function CursorPagination({
  limit,
  cursorData,
  searchGo,
  totalInfo = null,
  currentPageSize,
  loading = false,
  onRefresh,
  showRefresh = false,
  className,
  pageSizeOptions = [20, 50, 100],
  showPageSize = true,
  onPageSizeChange,
}: CursorPaginationProps) {
  const isMobile = useIsMobile();

  // 根据 cursorData 自动计算分页状态
  const hasNext = cursorData?.next != null;
  const canGoPrev = cursorData?.prev != null;

  // 分页处理函数
  const onPrevious = () => {
    if (cursorData?.prev != null) {
      searchGo({ pos: cursorData.prev, forward: false });
    }
  };

  const onNext = () => {
    if (cursorData?.next != null) {
      searchGo({ pos: cursorData.next, forward: true });
    }
  };

  // 解析总数显示
  const exactTotal = totalInfo?.exact ?? null;
  const overTotal = totalInfo?.over ?? null;
  const isExact = exactTotal !== null;
  const isOver = !isExact && overTotal !== null;


  // 计算总页数
  const exactPages = isExact && exactTotal ? Math.ceil(exactTotal / limit) : null;
  const overPages = isOver && overTotal ? Math.ceil(overTotal / limit) : null;

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
            onClick={onPrevious}
            disabled={!canGoPrev || loading}
            className={cn("flex-1 flex items-center justify-center gap-1 h-9")}
            title="上一页"
          >
            <ChevronLeft className={cn("h-4 w-4")} />
            <span>上一页</span>
          </Button>

          {/* 中间：记录信息 */}
          <div className="text-xs text-muted-foreground text-center whitespace-nowrap px-2">
            {loading ? (
              <Loader2 className={cn("h-3 w-3 animate-spin [animation-duration:0.5s]")} />
            ) : currentPageSize && currentPageSize > 0 ? (
              isExact && exactPages ? (
                <span>{exactTotal!.toLocaleString()}条·{exactPages}页</span>
              ) : isOver ? (
                <span>超{overTotal!.toLocaleString()}条·{overPages}+页</span>
              ) : (
                <span>—</span>
              )
            ) : (
              <span>无数据</span>
            )}
          </div>

          {/* 下一页按钮 */}
          <Button
            variant="outline"
            size="sm"
            onClick={onNext}
            disabled={!hasNext || loading}
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
      <div className="text-sm text-muted-foreground">
        {loading ? (
          <div className="flex items-center gap-2">
            <Loader2 className={cn("h-4 w-4 animate-spin [animation-duration:0.5s]")} />
            <span className="hidden lg:inline">加载中</span>
          </div>
        ) : currentPageSize && currentPageSize > 0 ? (
          <div className="flex items-center gap-1">
            {isExact && exactPages ? (
              <>
                <span className="hidden lg:inline">
                  共 {exactTotal!.toLocaleString()} 条，共 {exactPages} 页
                </span>
                <span className="lg:hidden">
                  {exactTotal!.toLocaleString()}条·{exactPages}页
                </span>
              </>
            ) : isOver ? (
              <>
                <span className="hidden lg:inline">
                  超过 {overTotal!.toLocaleString()} 条，超过 {overPages} 页
                </span>
                <span className="lg:hidden">
                  超{overTotal!.toLocaleString()}条·{overPages}+页
                </span>
              </>
            ) : (
              <span>第 1 页</span>
            )}
          </div>
        ) : (
          <div className="hidden lg:block">当前页无数据</div>
        )}
      </div>

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
              value={String(limit)}
              onValueChange={(value) => onPageSizeChange(Number(value))}
              disabled={loading || (isExact && exactTotal !== null && exactTotal < Math.min(...pageSizeOptions)) || (isOver && overTotal !== null && overTotal < Math.min(...pageSizeOptions))}
            >
              <SelectTrigger className={cn("h-8 w-20 text-center")}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent className="max-h-[300px]">
                {pageSizeOptions.map((size) => (
                  <SelectItem
                    key={size}
                    value={String(size)}
                    disabled={(isExact && exactTotal !== null && exactTotal < size) || (isOver && overTotal !== null && overTotal < size)}
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
          onClick={onPrevious}
          disabled={!canGoPrev || loading}
          className={cn("flex items-center gap-1 h-8 lg:px-3 px-2")}
          title="上一页"
        >
          <ChevronLeft className={cn("h-4 w-4")} />
          <span className="hidden lg:inline">上一页</span>
        </Button>

        {/* 下一页按钮 */}
        <Button
          variant="outline"
          size="sm"
          onClick={onNext}
          disabled={!hasNext || loading}
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
