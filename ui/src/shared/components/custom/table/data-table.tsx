import { Card, CardContent } from "@shared/components/ui/card";
import { Separator } from "@shared/components/ui/separator";
import { useIsMobile } from "@shared/hooks/use-mobile";
import { cn } from "@shared/lib/utils";
import {
  type ColumnDef,
  flexRender,
  getCoreRowModel,
  type Table as TanStackTable,
  useReactTable,
} from "@tanstack/react-table";
import { useCallback, useEffect, useMemo, useRef } from "react";
import { PageSkeletonTable } from "../page-placeholder/skeleton-table";
import { isDataTableAction } from "./utils";

/** 固定列配置 */
export interface StickyColumnConfig {
  /** 列标识：字段名或列索引 */
  column: string | number;
  /** 最小宽度 */
  minWidth: string;
  /** 最大宽度 */
  maxWidth: string;
}

export interface DataTableProps<TData> {
  /** 表格数据 */
  data: TData[];
  /** 表格列定义 */
  columns: ColumnDef<TData>[];
  /** 是否显示加载状态 */
  loading?: boolean;
  /** 错误元素，如果有错误会显示该元素 */
  error?: React.ReactNode;
  /** 空数据时显示的组件 */
  emptyComponent?: React.ReactNode;
  /** 是否启用滚动吸附功能 */
  enableScrollSnap?: boolean;
  /** 滚动吸附延迟时间(ms) */
  scrollSnapDelay?: number;
  /** 自定义类名 */
  className?: string;
  /** 表格容器自定义类名 */
  tableContainerClassName?: string;
  /** 表格自定义类名 */
  tableClassName?: string;
  /** 表头自定义类名 */
  headerClassName?: string;
  /** 表体自定义类名 */
  bodyClassName?: string;
  /** 表格行自定义类名 */
  rowClassName?: string;
  /** 表格单元格自定义类名 */
  cellClassName?: string;
  /** 左侧固定列配置数组 */
  leftStickyColumns?: StickyColumnConfig[];
  /** 右侧固定列配置数组 */
  rightStickyColumns?: StickyColumnConfig[];
  /** 是否启用列宽调整功能（仅PC端生效） */
  enableColumnResizing?: boolean;
  /** 获取表格实例的回调 */
  onTableReady?: (table: TanStackTable<TData>) => void;
  /** 渲染展开行内容的函数，返回 React.ReactNode 或 null，返回有效节点时才渲染展开行 */
  expandedRowRender?: (record: TData, index: number) => React.ReactNode | null;
}

export function DataTable<TData>({
  data,
  columns,
  loading = false,
  error,
  emptyComponent = <span className={cn("text-muted-foreground")}>暂无数据</span>,
  enableScrollSnap = true,
  scrollSnapDelay = 300,
  className,
  tableContainerClassName,
  tableClassName,
  headerClassName,
  bodyClassName,
  rowClassName,
  cellClassName,
  leftStickyColumns = [{ column: 0, minWidth: "60px", maxWidth: "60px" }],
  rightStickyColumns = [],
  enableColumnResizing = true,
  onTableReady,
  expandedRowRender,
}: DataTableProps<TData>) {
  // 检测是否为移动端
  const isMobile = useIsMobile();

  // 表格滚动容器引用
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const scrollTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // 创建表格实例
  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    manualPagination: true, // 手动分页
    enableColumnResizing: enableColumnResizing && !isMobile,
    columnResizeMode: "onChange",
  });

  /**
   * 使用官方推荐的方式：通过 CSS 变量定义列宽，而不是在每个单元格上调用 getSize()
   * 这样可以大幅提升性能，特别是在大数据量表格中
   */
  const columnSizeVars = useMemo(() => {
    const headers = table.getFlatHeaders();
    const colSizes: { [key: string]: number } = {};
    for (let i = 0; i < headers.length; i++) {
      const header = headers[i]!;
      colSizes[`--header-${header.id}-size`] = header.getSize();
      colSizes[`--col-${header.column.id}-size`] = header.column.getSize();
    }
    return colSizes;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [table.getState().columnSizingInfo, table.getState().columnSizing]);

  // 辅助函数:检查列是否为左侧固定列
  const getStickyColumnConfig = useCallback(
    (headerOrCell: any, index: number): StickyColumnConfig | null => {
      for (const config of leftStickyColumns) {
        if (typeof config.column === "number" && config.column === index) {
          return config;
        }
        if (
          typeof config.column === "string" &&
          headerOrCell.column.id === config.column
        ) {
          return config;
        }
      }
      return null;
    },
    [leftStickyColumns],
  );

  // 计算固定列的left偏移量
  const getStickyColumnLeftOffset = useCallback(
    (headerOrCell: any, index: number): string => {
      let offset = 0;
      for (let i = 0; i < index; i++) {
        const config = getStickyColumnConfig(
          table.getHeaderGroups()[0]?.headers[i],
          i,
        );
        if (config) {
          // 简单处理：假设每个固定列宽度为其maxWidth
          const width = parseInt(config.maxWidth.replace("px", ""));
          offset += width;
        }
      }
      return `${offset}px`;
    },
    [getStickyColumnConfig, table],
  );

  // 辅助函数:检查列是否为右侧固定列
  const getRightStickyColumnConfig = useCallback(
    (headerOrCell: any, index: number): StickyColumnConfig | null => {
      for (const config of rightStickyColumns) {
        if (typeof config.column === "number" && config.column === index) {
          return config;
        }
        if (
          typeof config.column === "string" &&
          headerOrCell.column.id === config.column
        ) {
          return config;
        }
      }
      return null;
    },
    [rightStickyColumns],
  );

  // 计算右侧固定列的right偏移量
  const getRightStickyColumnRightOffset = useCallback(
    (headerOrCell: any, index: number): string => {
      let offset = 0;
      const headers = table.getHeaderGroups()[0]?.headers || [];
      for (let i = index + 1; i < headers.length; i++) {
        const config = getRightStickyColumnConfig(headers[i], i);
        if (config) {
          const width = parseInt(config.maxWidth.replace("px", ""));
          offset += width;
        }
      }
      return `${offset}px`;
    },
    [getRightStickyColumnConfig, table],
  );

  // 当表格实例准备好时通知父组件
  useEffect(() => {
    if (onTableReady) {
      onTableReady(table);
    }
  }, [table, onTableReady]);

  // 滚动吸附逻辑
  const handleScrollSnap = useCallback(() => {
    if (!enableScrollSnap) return;

    const container = scrollContainerRef.current;
    if (!container) return;

    const tableElement = container.querySelector("table");
    const thead = tableElement?.querySelector("thead");
    const tbody = tableElement?.querySelector("tbody");

    if (!tableElement || !thead || !tbody) return;

    const theadHeight = thead.offsetHeight;
    const rows = tbody.querySelectorAll("tr");

    if (rows.length === 0) return;

    // 获取当前滚动位置
    const scrollTop = container.scrollTop;
    const maxScrollTop = container.scrollHeight - container.clientHeight;

    // 如果已经在顶部或底部，不需要吸附
    if (scrollTop <= 3) {
      // 确保完全在顶部
      if (scrollTop > 0) {
        container.scrollTo({ top: 0, behavior: "smooth" });
      }
      return;
    }

    if (scrollTop >= maxScrollTop - 3) {
      // 已经在底部，不需要吸附
      return;
    }

    // 查找当前在视口中的第一个可见行
    let firstVisibleRowIndex = -1;
    let firstVisibleRow: HTMLElement | null = null;

    for (let i = 0; i < rows.length; i++) {
      const row = rows[i] as HTMLElement;
      const rowRect = row.getBoundingClientRect();
      const containerRect = container.getBoundingClientRect();

      // 计算行在容器中的实际位置
      const rowTopInContainer = rowRect.top - containerRect.top;
      const rowBottomInContainer = rowRect.bottom - containerRect.top;

      // 检查行是否与表头区域后的可视区域有交集
      if (
        rowBottomInContainer > theadHeight &&
        rowTopInContainer < container.clientHeight
      ) {
        firstVisibleRowIndex = i;
        firstVisibleRow = row;
        break;
      }
    }

    if (firstVisibleRowIndex === -1 || !firstVisibleRow) return;

    // 计算行在表头下方的可见部分
    const rowRect = firstVisibleRow.getBoundingClientRect();
    const containerRect = container.getBoundingClientRect();
    const rowTopInContainer = rowRect.top - containerRect.top;
    const rowHeight = firstVisibleRow.offsetHeight;

    // 计算行顶部相对于表头底部的位置
    const rowTopRelativeToHeader = rowTopInContainer - theadHeight;

    // 计算可见高度
    let visibleHeight: number;
    if (rowTopRelativeToHeader >= 0) {
      // 行完全在表头下方
      visibleHeight = rowHeight;
    } else {
      // 行部分被表头遮挡
      visibleHeight = rowHeight + rowTopRelativeToHeader;
    }

    const halfRowHeight = rowHeight / 2;
    let targetScrollTop: number;

    // 边框偏移量，确保行的顶部边框可见
    const borderOffset = 1;

    if (visibleHeight < halfRowHeight) {
      // 露出不到一半，向下吸附到下一行（如果存在）
      if (firstVisibleRowIndex + 1 < rows.length) {
        const nextRow = rows[firstVisibleRowIndex + 1] as HTMLElement;
        const nextRowRect = nextRow.getBoundingClientRect();
        const nextRowTopRelativeToContainer =
          nextRowRect.top - containerRect.top;
        // 计算需要滚动的距离，让下一行的顶部对齐表头底部，并留出边框空间
        targetScrollTop =
          scrollTop +
          (nextRowTopRelativeToContainer - theadHeight) -
          borderOffset;
      } else {
        // 没有下一行，吸附到当前行
        targetScrollTop =
          scrollTop + (rowTopInContainer - theadHeight) - borderOffset;
      }
    } else {
      // 露出超过一半，向上吸附到当前行，留出边框空间
      targetScrollTop =
        scrollTop + (rowTopInContainer - theadHeight) - borderOffset;
    }

    // 确保不会滚动到负值或超出边界
    targetScrollTop = Math.max(0, Math.min(maxScrollTop, targetScrollTop));

    // 只有当需要调整的距离超过阈值时才进行滚动
    const scrollDistance = Math.abs(targetScrollTop - scrollTop);
    if (scrollDistance > 2) {
      // 平滑滚动到目标位置
      container.scrollTo({
        top: targetScrollTop,
        behavior: "smooth",
      });
    }
  }, [enableScrollSnap]);

  // 滚动事件处理
  const handleScroll = useCallback(() => {
    if (!enableScrollSnap) return;

    // 清除之前的定时器
    if (scrollTimeoutRef.current) {
      clearTimeout(scrollTimeoutRef.current);
    }

    // 设置新的定时器，在滚动停止后执行吸附
    scrollTimeoutRef.current = setTimeout(() => {
      handleScrollSnap();
    }, scrollSnapDelay);
  }, [handleScrollSnap, scrollSnapDelay, enableScrollSnap]);

  // 添加和清理滚动监听器
  useEffect(() => {
    if (!enableScrollSnap) return;

    const container = scrollContainerRef.current;
    if (!container) return;

    container.addEventListener("scroll", handleScroll);

    return () => {
      container.removeEventListener("scroll", handleScroll);
      if (scrollTimeoutRef.current) {
        clearTimeout(scrollTimeoutRef.current);
      }
    };
  }, [handleScroll, enableScrollSnap]);

  // 渲染移动端卡片视图
  const renderMobileCardView = () => {
    if (loading) {
      return (
        <div className={cn("md:p-4")}>
          <PageSkeletonTable variant="content" rows={columns.length} />
        </div>
      );
    }

    if (error) {
      return (
        <div className={cn("md:p-4")}>
          {error}
        </div>
      );
    }

    if (table.getRowModel().rows?.length === 0) {
      return (
        <div className={cn("md:p-4 text-center text-muted-foreground")}>
          {emptyComponent}
        </div>
      );
    }

    return (
      <div className={cn("py-4 space-y-4")}>
        {table.getRowModel().rows.map((row) => {
          const cells = row.getVisibleCells();

          return (
            <Card key={row.id} className={cn("pb-0 px-0")}>
              <CardContent className={cn("p-4")}>
                {/* 数据字段 */}
                <div className={cn("space-y-3")}>
                  {cells.map((cell) => {
                    const header = cell.column.columnDef.header;
                    const headerGroup = table.getHeaderGroups()[0];
                    const headerCell = headerGroup?.headers.find((h) => h.column.id === cell.column.id);
                    const cellContent = cell.column.columnDef.cell;

                    // 获取渲染后的单元格内容
                    const renderedCell = typeof cellContent === "function"
                      ? cellContent(cell.getContext())
                      : cellContent;

                    // 检查是否是 DataTableAction 组件
                    const isActionCell = isDataTableAction(renderedCell);

                    // 如果是 action cell，保持一栏布局
                    if (isActionCell) {
                      return (
                        <div key={cell.id} className={cn("text-sm")}>
                          {flexRender(
                            cell.column.columnDef.cell,
                            cell.getContext(),
                          )}
                        </div>
                      );
                    }

                    // 非 action cell，使用左右布局
                    return (
                      <div key={cell.id} className={cn("flex items-start gap-3")}>
                        <div className={cn("text-xs font-medium text-muted-foreground whitespace-nowrap min-w-[80px] shrink-0 pt-0.5")}>
                          {typeof header === "function" && headerCell
                            ? flexRender(header, headerCell.getContext())
                            : (header as React.ReactNode)}
                        </div>
                        <div className={cn("text-sm flex-1 min-w-0 overflow-x-auto")}>
                          <div className={cn("break-all")}>
                            {flexRender(
                              cell.column.columnDef.cell,
                              cell.getContext(),
                            )}
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>
    );
  };

  // 如果是移动端，使用卡片视图
  if (isMobile) {
    return (
      <div
        ref={scrollContainerRef}
        className={cn(
          "relative overflow-auto flex-1 min-h-0",
          tableContainerClassName,
          className,
        )}
      >
        {renderMobileCardView()}
      </div>
    );
  }

  // 桌面端使用表格视图
  return (
    <div
      ref={scrollContainerRef}
      className={cn(
        "rounded-md border relative overflow-auto flex-1 min-h-0 custom-scrollbar",
        tableContainerClassName,
        className,
      )}
    >
      <table
        className={cn(
          "w-full caption-bottom text-sm relative min-w-max",
          tableClassName,
          loading || error || table.getRowModel().rows?.length === 0
            ? "h-full"
            : undefined,
        )}
        style={{
          ...columnSizeVars,
        }}
      >
        <thead
          className={cn(
            "sticky top-0 bg-background z-10 [&_tr]:border-b",
            headerClassName,
          )}
        >
          {table.getHeaderGroups().map((headerGroup) => (
            <tr
              key={headerGroup.id}
              className={cn("border-b hover:[background-color:var(--primary-foreground)] hover:border-muted data-[state=selected]:bg-muted group table-header-row")}
            >
              {headerGroup.headers.map((header, index) => {
                const stickyConfig = getStickyColumnConfig(header, index);
                const rightStickyConfig = getRightStickyColumnConfig(header, index);
                const isLeftSticky = !!stickyConfig;
                const isRightSticky = !!rightStickyConfig;
                const isSticky = isLeftSticky || isRightSticky;

                return (
                  <th
                    key={header.id}
                    className={cn(
                      "h-12 px-4 text-left align-middle font-medium text-muted-foreground [&:has([role=checkbox])]:pr-0 relative whitespace-nowrap",
                      isSticky &&
                      "sticky bg-background group-hover:[background-color:var(--primary-foreground)] z-20 border-b",
                      isLeftSticky && "border-r",
                      isRightSticky && "border-l",
                      header.column.getIsResizing() && "select-none",
                    )}
                    style={
                      isLeftSticky
                        ? ({
                          "--sticky-left": getStickyColumnLeftOffset(
                            header,
                            index,
                          ),
                          "--min-width": stickyConfig.minWidth,
                          "--max-width": stickyConfig.maxWidth,
                          left: "var(--sticky-left)",
                          minWidth: "var(--min-width)",
                          maxWidth: "var(--max-width)",
                          width: `calc(var(--header-${header.id}-size) * 1px)`,
                          zIndex: 20,
                        } as React.CSSProperties)
                        : isRightSticky
                          ? ({
                            "--sticky-right": getRightStickyColumnRightOffset(
                              header,
                              index,
                            ),
                            "--min-width": rightStickyConfig.minWidth,
                            "--max-width": rightStickyConfig.maxWidth,
                            right: "var(--sticky-right)",
                            minWidth: "var(--min-width)",
                            maxWidth: "var(--max-width)",
                            width: `calc(var(--header-${header.id}-size) * 1px)`,
                            zIndex: 30,
                          } as React.CSSProperties)
                          : ({
                            width: `calc(var(--header-${header.id}-size) * 1px)`,
                          } as React.CSSProperties)
                    }
                  >
                    {header.isPlaceholder
                      ? null
                      : flexRender(
                        header.column.columnDef.header,
                        header.getContext(),
                      )}
                    {/* 使用官方的列宽调整手柄 */}
                    {enableColumnResizing && !isMobile && !isSticky && (
                      <div
                        className="absolute right-0 top-0 h-full w-3 cursor-col-resize group/resizer flex items-center justify-center"
                        onDoubleClick={() => header.column.resetSize()}
                        onMouseDown={header.getResizeHandler()}
                        onTouchStart={header.getResizeHandler()}
                        style={{
                          userSelect: "none",
                          touchAction: "none",
                        }}
                      >
                        <Separator
                          orientation="vertical"
                          className={cn(
                            "resizer h-[70%] w-[1px] group-hover/resizer:w-[3px] select-none touch-none transition-all pointer-events-none",
                            header.column.getIsResizing()
                              ? "bg-primary opacity-100 w-[3px]"
                              : "bg-border opacity-0 group-hover/resizer:bg-primary group-hover/resizer:opacity-100"
                          )}
                        />
                      </div>
                    )}
                  </th>
                );
              })}
            </tr>
          ))}
        </thead>
        <tbody className={cn("[&_tr:last-child]:border-0", bodyClassName)}>
          {loading ? (
            <tr
              className={cn(
                "border-b hover:[background-color:var(--primary-foreground)] data-[state=selected]:bg-muted",
                rowClassName,
              )}
            >
              <td
                colSpan={table.getAllColumns().length}
                className={cn(
                  "align-middle [&:has([role=checkbox])]:pr-0 text-center h-60 p-4",
                  cellClassName,
                )}
              >
                <PageSkeletonTable
                  variant="content"
                  rows={columns.length}
                />
              </td>
            </tr>
          ) : error ? (
            <tr
              className={cn(
                "border-b hover:[background-color:var(--primary-foreground)] data-[state=selected]:bg-muted",
                rowClassName,
              )}
            >
              <td
                colSpan={table.getAllColumns().length}
                className={cn(
                  "p-4 align-middle [&:has([role=checkbox])]:pr-0 h-24 text-center",
                  cellClassName,
                )}
              >
                {error}
              </td>
            </tr>
          ) : table.getRowModel().rows?.length === 0 ? (
            <tr
              className={cn(
                "border-b hover:[background-color:var(--primary-foreground)] data-[state=selected]:bg-muted",
                rowClassName,
              )}
            >
              <td
                colSpan={table.getAllColumns().length}
                className={cn(
                  "p-4 align-middle [&:has([role=checkbox])]:pr-0 h-24 text-center text-muted-foreground",
                  cellClassName,
                )}
              >
                {emptyComponent}
              </td>
            </tr>
          ) : (
            table.getRowModel().rows.map((row, rowIndex) => {
              // 获取展开内容
              const expandedContent = expandedRowRender?.(row.original, rowIndex);

              // 判断是否有展开内容（有效节点）
              const hasExpandable = expandedContent != null;

              const elements = [
                // 主要数据行
                <tr
                  key={row.id}
                  className={cn(
                    "table-main-row data-[state=selected]:bg-muted",
                    !hasExpandable && "border-b", // 只有在没有展开内容时才显示下边框
                    rowClassName,
                  )}
                >
                  {row.getVisibleCells().map((cell, index) => {
                    const stickyConfig = getStickyColumnConfig(cell, index);
                    const rightStickyConfig = getRightStickyColumnConfig(cell, index);
                    const isLeftSticky = !!stickyConfig;
                    const isRightSticky = !!rightStickyConfig;
                    const isSticky = isLeftSticky || isRightSticky;

                    return (
                      <td
                        key={cell.id}
                        className={cn(
                          "p-4 align-middle [&:has([role=checkbox])]:pr-0",
                          isSticky &&
                          "sticky bg-background table-sticky-cell",
                          isLeftSticky && "border-r",
                          isRightSticky && "border-l",
                          cellClassName,
                        )}
                        style={
                          isLeftSticky
                            ? {
                              left: getStickyColumnLeftOffset(cell, index),
                              minWidth: stickyConfig.minWidth,
                              maxWidth: stickyConfig.maxWidth,
                              width: `calc(var(--col-${cell.column.id}-size) * 1px)`,
                              zIndex: 20,
                            }
                            : isRightSticky
                              ? {
                                right: getRightStickyColumnRightOffset(cell, index),
                                minWidth: rightStickyConfig.minWidth,
                                maxWidth: rightStickyConfig.maxWidth,
                                width: `calc(var(--col-${cell.column.id}-size) * 1px)`,
                                zIndex: 30,
                              }
                              : {
                                width: `calc(var(--col-${cell.column.id}-size) * 1px)`,
                                maxWidth: `calc(var(--col-${cell.column.id}-size) * 1px)`,
                              }
                        }
                      >
                        {flexRender(
                          cell.column.columnDef.cell,
                          cell.getContext(),
                        )}
                      </td>
                    );
                  })}
                </tr>,
              ];

              // 展开行 - 只有当有展开内容时才显示
              if (hasExpandable) {
                elements.push(
                  <tr
                    key={`${row.id}-expanded`}
                    className={cn(
                      "table-expanded-row border-b data-[state=selected]:bg-muted",
                    )}
                  >
                    {/* 为每个固定列创建空白单元格以避免内容覆盖 */}
                    {table.getAllColumns().map((column, columnIndex) => {
                      const leftStickyConfig = getStickyColumnConfig(
                        { column },
                        columnIndex,
                      );
                      const rightStickyConfig = getRightStickyColumnConfig(
                        { column },
                        columnIndex,
                      );
                      const isLeftSticky = !!leftStickyConfig;
                      const isRightSticky = !!rightStickyConfig;

                      if (isLeftSticky) {
                        // 左侧固定列：创建空白单元格
                        return (
                          <td
                            key={`${column.id}-expanded-sticky-left`}
                            className={cn("p-0 align-top border-t border-dashed border-muted border-r border-r-border sticky bg-background table-sticky-cell")}
                            style={{
                              left: getStickyColumnLeftOffset(
                                { column },
                                columnIndex,
                              ),
                              minWidth: leftStickyConfig.minWidth,
                              maxWidth: leftStickyConfig.maxWidth,
                              zIndex: 25,
                            }}
                          >
                            {/* 空白内容，只显示边框 */}
                          </td>
                        );
                      } else if (isRightSticky) {
                        // 右侧固定列：创建空白单元格
                        return (
                          <td
                            key={`${column.id}-expanded-sticky-right`}
                            className={cn("p-0 align-top border-t border-dashed border-muted border-l border-l-border sticky bg-background table-sticky-cell")}
                            style={{
                              right: getRightStickyColumnRightOffset(
                                { column },
                                columnIndex,
                              ),
                              minWidth: rightStickyConfig.minWidth,
                              maxWidth: rightStickyConfig.maxWidth,
                              zIndex: 35,
                            }}
                          >
                            {/* 空白内容，只显示边框 */}
                          </td>
                        );
                      } else if (columnIndex === leftStickyColumns.length) {
                        // 第一个非固定列：包含展开内容
                        return (
                          <td
                            key={`${column.id}-expanded-content`}
                            colSpan={
                              table.getAllColumns().length -
                              leftStickyColumns.length -
                              rightStickyColumns.length
                            }
                            className={cn(
                              "p-0 align-top border-0 border-t border-dashed border-muted",
                              cellClassName,
                            )}
                          >
                            {expandedContent}
                          </td>
                        );
                      } else {
                        // 其他列，返回null避免重复渲染
                        return null;
                      }
                    })}
                  </tr>,
                );
              }

              return elements;
            })
          )}
        </tbody>
      </table>
    </div>
  );
}

export default DataTable;
