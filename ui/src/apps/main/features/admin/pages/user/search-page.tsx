import { FilterContainer } from "@apps/main/components/filter-container/container";
import { FilterActions } from "@apps/main/components/filter-container/filter-actions";
import { FilterInput } from "@apps/main/components/filter-container/filter-input";
import { FilterSelect } from "@apps/main/components/filter-container/filter-select";
import { FilterTotalCount } from "@apps/main/components/filter-container/filter-total-count";
import { useDictData, type TypedDictData } from "@apps/main/hooks/use-dict-data";
import {
  DEFAULT_PAGE_SIZE,
  PAGE_SIZE_OPTIONS,
  useCountNumManager,
  useOffsetPaginationHandlers,
  useSearchNavigate,
} from "@apps/main/lib/pagination-utils";
import { createStatusMapper } from "@apps/main/lib/status-utils";
import { Route } from "@apps/main/routes/_main/admin/user/search";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  systemUserAccountSearch,
  type SystemUserAccountItemType,
  type SystemUserAccountSearchParamType,
} from "@shared/apis/admin/user";
import { CenteredError } from "@shared/components/custom/page-placeholder/centered-error";
import { PageSkeletonTable } from "@shared/components/custom/page-placeholder/skeleton-table";
import { OffsetPagination } from "@shared/components/custom/pagination";
import { DataTable, DataTableAction, DataTableActionItem } from "@shared/components/custom/table";
import CopyableText from "@shared/components/custom/text/copyable-text";
import { Badge } from "@shared/components/ui/badge";
import { Button } from "@shared/components/ui/button";
import { useIsMobile } from "@shared/hooks/use-mobile";
import {
  cn,
  extractMinMax,
  formatTime,
  getQueryResponseData,
  getQueryResponseNext,
  TIME_STYLE,
} from "@shared/lib/utils";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { type ColumnDef } from "@tanstack/react-table";
import { Eye, Mail, Phone, Users } from "lucide-react";
import { useCallback, useMemo, useState } from "react";
import { UserAccountFilterFormSchema } from "./search-schema";
import { UserDetailDrawer } from "./user-detail-drawer";

// ==================== 主页面组件 ====================

export function UserSearchPage() {
  // 字典数据获取 - 统一在最顶层获取一次
  const {
    dictData,
    isLoading: dictIsLoading,
    isError: dictError,
    errors: dictErrors,
    refetch: refetchDict,
  } = useDictData(["admin_user"] as const);

  // 如果字典加载失败，显示错误页面
  if (dictError && dictErrors.length > 0) {
    return (
      <CenteredError
        variant="page"
        error={dictErrors}
        onReset={refetchDict}
        className={cn("m-4 md:m-6")}
      />
    );
  }

  // 如果字典加载中，显示骨架屏
  if (dictIsLoading) {
    return (
      <PageSkeletonTable
        variant="page"
        rows={6}
        columns={8}
        className={cn("container mx-auto m-4 md:m-6")}
      />
    );
  }

  // 字典加载成功，渲染内容组件
  return <UserSearchContent dictData={dictData} />;
}

// ==================== 类型定义 ====================

interface UserSearchContentProps {
  dictData: TypedDictData<["admin_user"]>;
}

// ==================== 用户搜索内容组件 ====================

function UserSearchContent({ dictData }: UserSearchContentProps) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const isMobile = useIsMobile();

  // 获取 URL search 参数
  const filterParam = Route.useSearch();

  // 用户详情抽屉状态
  const [detailDrawer, setDetailDrawer] = useState({
    open: false,
    userId: 0,
  });

  // 过滤条件从 URL 参数获取
  const filters = {
    key_word: filterParam.key_word || null,
    enable: filterParam.enable, // 保持 undefined，不转为 null
  };

  const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;

  // 分页状态 - 直接从 URL 参数派生
  const pagination = {
    pos: filterParam.pos || null,
    limit: currentLimit,
    forward: filterParam.forward || false,
    more: true,
    eq_pos: filterParam.eq_pos || false,
  };

  // 搜索导航函数
  const searchGo = useSearchNavigate(navigate, filterParam);

  // count_num 优化管理器
  const countNumManager = useCountNumManager(filters);

  // 计算 enable 参数值
  // 当 enable 为 undefined 时，默认查询启用用户
  // 当 enable 为 true 时，只查询启用用户
  // 当 enable 为 false 时，只查询未启用用户
  const enableValue = filters.enable ?? true;

  // 构建查询参数
  const queryParams: SystemUserAccountSearchParamType = {
    limit: {
      eq_pos: pagination.eq_pos,
      pos: pagination.pos,
      limit: pagination.limit,
      forward: pagination.forward,
      more: pagination.more,
    },
    count_num: countNumManager.getCountNum(),
    enable: enableValue,
    base: true,
    name: true,
    email: [1, 2],
    mobile: [1, 2],
    ...(filters.key_word && { key_word: filters.key_word }),
  };

  // 获取用户数据
  const {
    data: userData,
    isSuccess: userIsSuccess,
    isLoading: userIsLoading,
    isError,
    error,
  } = useQuery({
    queryKey: ["systemUserAccountSearch", queryParams],
    queryFn: ({ signal }) => systemUserAccountSearch(queryParams, { signal }),
  });

  // 处理 Limit 分页查询结果
  userIsSuccess && countNumManager.handleLimitQueryResult(userData);

  // 刷新数据
  const refreshData = () => {
    queryClient.refetchQueries({ queryKey: ["systemUserAccountSearch"] });
  };

  // 清除缓存并重新加载数据
  const clearCacheAndReload = () => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ["systemUserAccountSearch"] });
  };

  // 获取API响应数据
  const users = getQueryResponseData<SystemUserAccountItemType[]>(userData, []);
  const nextPageStartPos = getQueryResponseNext(userData);

  // 获取字典数据
  const accountStatusDict = dictData.account_status;

  // 处理查看详情
  const handleViewDetail = useCallback((userId: number) => {
    setDetailDrawer({ open: true, userId });
  }, []);

  // 处理详情抽屉开关
  const handleDetailDrawerOpenChange = (open: boolean) => {
    if (!open) {
      setDetailDrawer({ open: false, userId: 0 });
    }
  };

  // 创建用户状态映射器
  const userStatus = useMemo(
    () =>
      accountStatusDict
        ? createStatusMapper(
          {
            1: "neutral",
            2: "success",
          },
          (status) =>
            accountStatusDict.getLabel(String(status)) || String(status)
        )
        : null,
    [accountStatusDict]
  );

  // 定义表格列
  const columns = useMemo<ColumnDef<SystemUserAccountItemType>[]>(
    () => [
      {
        accessorKey: "user.id",
        header: () => <div className={cn(isMobile ? "" : "text-right")}>ID</div>,
        size: 80,
        cell: ({ getValue }) => (
          <div className={cn("font-mono text-xs", isMobile ? "" : "text-right")}>{getValue<number>()}</div>
        ),
      },
      {
        accessorKey: "user.nickname",
        header: "昵称",
        cell: ({ getValue }) => {
          const nickname = getValue<string>();
          return (
            <div className="max-w-xs truncate text-sm" title={nickname}>
              {nickname || "-"}
            </div>
          );
        },
      },
      {
        accessorKey: "name.username",
        header: "用户名",
        cell: ({ getValue }) => {
          const username = getValue<string>();
          return username ? (
            <CopyableText value={username} className="text-sm" />
          ) : (
            <span className="text-muted-foreground">-</span>
          );
        },
      },
      {
        header: "联系方式",
        cell: ({ row }) => {
          const user = row.original;
          const emails = user.email || [];
          const mobiles = user.mobile || [];

          return (
            <div className="text-xs space-y-1">
              {emails.length > 0 && (
                <div className="flex items-center gap-1 text-blue-600 dark:text-blue-400">
                  <Mail className="h-3 w-3" />
                  <span className="truncate max-w-[150px]">
                    {emails[0].email}
                  </span>
                  {emails.length > 1 && (
                    <span className="text-muted-foreground">
                      +{emails.length - 1}
                    </span>
                  )}
                </div>
              )}
              {mobiles.length > 0 && (
                <div className="flex items-center gap-1 text-green-600 dark:text-green-400">
                  <Phone className="h-3 w-3" />
                  <span>
                    +{mobiles[0].area_code} {mobiles[0].mobile}
                  </span>
                  {mobiles.length > 1 && (
                    <span className="text-muted-foreground">
                      +{mobiles.length - 1}
                    </span>
                  )}
                </div>
              )}
              {emails.length === 0 && mobiles.length === 0 && (
                <span className="text-muted-foreground">-</span>
              )}
            </div>
          );
        },
      },
      {
        header: "匹配信息",
        cell: ({ row }) => {
          const cat = row.original.cat;
          if (!cat || cat.length === 0) {
            return <span className="text-muted-foreground">-</span>;
          }
          return (
            <div className="text-xs space-y-1">
              {cat.map((c, idx) => (
                <div key={idx} className="flex items-center gap-1">
                  <Badge variant="outline" className="text-xs">
                    {c.type}
                  </Badge>
                  <span className="truncate max-w-[100px]">{c.val}</span>
                </div>
              ))}
            </div>
          );
        },
      },
      {
        accessorKey: "user.status",
        header: "状态",
        cell: ({ getValue }) => {
          const status = getValue<number>();
          if (!userStatus) {
            return (
              <Badge variant={status === 2 ? "default" : "secondary"}>
                {status === 2 ? "启用" : "初始"}
              </Badge>
            );
          }
          return (
            <Badge className={userStatus.getClass(status)}>
              {userStatus.getText(status)}
            </Badge>
          );
        },
      },
      {
        accessorKey: "user.add_time",
        header: "注册时间",
        cell: ({ getValue }) => {
          const date = getValue<Date>();
          const timeElement = formatTime(date, TIME_STYLE.RELATIVE_ELEMENT);
          return (
            <div className="text-xs text-muted-foreground">{timeElement}</div>
          );
        },
      },
      {
        id: "actions",
        header: () => <div className={cn(isMobile ? "text-right" : "text-center")}>操作</div>,
        cell: ({ row }) => {
          const user = row.original;

          return (
            <DataTableAction className={cn(isMobile ? "justify-end" : "justify-center")}>
              <DataTableActionItem
                mobileDisplay="display"
                desktopDisplay="display"
              >
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => handleViewDetail(user.user.id)}
                  title="查看详情"
                >
                  <Eye className="h-4 w-4 mr-1" />
                  {!isMobile && "详情"}
                </Button>
              </DataTableActionItem>
            </DataTableAction>
          );
        },
      },
    ],
    [userStatus, handleViewDetail, isMobile]
  );

  // 使用统一的分页处理器
  const { handleNextPage, handlePrevPage, canGoNext, canGoPrev } =
    useOffsetPaginationHandlers({
      ...extractMinMax(
        users.map((u) => ({ id: u.user.id })),
        "id",
        "minId",
        "maxId"
      ),
      pagination,
      nextPageStartPos,
      searchGo,
      defaultForward: false,
    });

  const isLoading = userIsLoading;

  // 表单默认值
  const defaultValues = {
    key_word: filters.key_word ?? "",
    enable: filters.enable === true ? "true" : filters.enable === false ? "false" : "",
  };

  return (
    <div className="container mx-auto p-4  max-w-[1600px] flex flex-col min-h-0 space-y-5">
      {/* 页面标题 */}
      <div className="flex-shrink-0">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold flex items-center gap-2">
              <Users className="h-6 w-6" />
              用户管理
            </h1>
            <p className="text-muted-foreground">管理系统用户和权限</p>
          </div>
        </div>
      </div>

      {/* 筛选区域 */}
      <div className="flex-shrink-0">
        <FilterContainer
          defaultValues={defaultValues}
          resolver={zodResolver(UserAccountFilterFormSchema) as any}
          onSubmit={(data) => {
            // 转换 enable 字段：空字符串视为 undefined（全部）
            const enableValue = data.enable === "true" ? true : data.enable === "false" ? false : undefined;
            navigate({
              search: {
                ...filterParam,
                key_word: data.key_word || undefined,
                enable: enableValue,
                pos: null,
                forward: false,
                eq_pos: false,
              } as any,
            });
          }}
          onReset={() => {
            navigate({
              search: {
                pos: null,
                limit: currentLimit,
                forward: false,
                eq_pos: false,
              } as any,
            });
          }}
          countComponent={<FilterTotalCount total={countNumManager.getTotal() ?? 0} loading={isLoading} />}
          className={cn("bg-card rounded-lg border shadow-sm relative")}
        >
          {(layoutParams, form) => (
            <div
              className={cn(
                "flex items-end gap-4 flex-wrap",
                layoutParams.isMobile && "flex-col items-stretch"
              )}
            >
              <div className="flex-1 min-w-[180px] max-w-[280px]">
                <FilterInput
                  name="key_word"
                  label="关键词搜索"
                  placeholder="搜索昵称/用户名/邮箱/手机"
                  disabled={isLoading}
                  layoutParams={layoutParams}
                />
              </div>

              <div className="flex-1 min-w-[140px] max-w-[180px]">
                <FilterSelect
                  name="enable"
                  label="账号状态"
                  placeholder="全部状态"
                  disabled={isLoading}
                  layoutParams={layoutParams}
                  allLabel="全部"
                  options={[
                    { label: "启用", value: "true" },
                    { label: "未启用", value: "false" },
                  ]}
                />
              </div>

              {/* 动作按钮区域 */}
              <div className={cn(layoutParams.isMobile ? "w-full" : "flex-shrink-0")}>
                <FilterActions
                  form={form}
                  loading={isLoading}
                  layoutParams={layoutParams}
                  onRefreshSearch={clearCacheAndReload}
                />
              </div>
            </div>
          )}
        </FilterContainer>
      </div>

      {/* 表格和分页容器 */}
      <div className="flex-1 flex flex-col overflow-hidden min-h-0">
        {/* 数据表格 */}
        <div className="flex-1 min-h-0">
          <DataTable
            data={users}
            columns={columns}
            loading={isLoading}
            error={
              isError ? (
                <CenteredError
                  error={error}
                  variant="content"
                  onReset={refreshData}
                />
              ) : null
            }
            className="h-full [&_.data-table-row]:h-12 [&_td]:py-2 [&_th]:py-2 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b [&_.data-table-wrapper]:overflow-auto [&_.data-table-wrapper]:h-full"
          />
        </div>

        {/* 分页控件 */}
        <div className="flex-shrink-0 pt-4">
          {(countNumManager.getTotal() ?? 0) > 0 && (
            <OffsetPagination
              limit={pagination.limit}
              hasNext={canGoNext}
              canGoPrev={canGoPrev}
              total={countNumManager.getTotal()}
              currentPageSize={users.length}
              loading={isLoading}
              onPrevious={handlePrevPage}
              onNext={handleNextPage}
              onRefresh={refreshData}
              showRefresh={true}
              showPageSize={true}
              pageSizeOptions={PAGE_SIZE_OPTIONS}
              onPageSizeChange={(pageSize) => {
                searchGo({
                  limit: pageSize,
                  pos: null,
                  forward: false,
                  eq_pos: false,
                });
              }}
            />
          )}
        </div>
      </div>

      {/* 用户详情抽屉 */}
      <UserDetailDrawer
        userId={detailDrawer.userId}
        open={detailDrawer.open}
        onOpenChange={handleDetailDrawerOpenChange}
        dictData={dictData}
      />
    </div>
  );
}
