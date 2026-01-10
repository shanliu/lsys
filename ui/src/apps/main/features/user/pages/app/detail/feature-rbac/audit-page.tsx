import { FilterContainer } from '@apps/main/components/filter-container/container';
import { FilterActions } from '@apps/main/components/filter-container/filter-actions';
import { FilterDictSelect } from '@apps/main/components/filter-container/filter-dict-select';
import { FilterInput } from '@apps/main/components/filter-container/filter-input';
import { FilterTotalCount } from '@apps/main/components/filter-container/filter-total-count';
import { AuditDetailTooltip } from '@apps/main/components/local/audit-detail-tooltip';
import { UserDataTooltip } from '@apps/main/components/local/user-data-tooltip';
import { AppDetailNavContainer } from '@apps/main/features/user/components/ui/app-detail-nav';
import { useDictData } from '@apps/main/hooks/use-dict-data';
import {
  DEFAULT_PAGE_SIZE,
  OffsetPagination,
  PAGE_SIZE_OPTIONS,
  useCountNumManager,
  useOffsetPaginationHandlers,
  useSearchNavigate,
} from '@apps/main/lib/pagination-utils';
import { createStatusMapper } from '@apps/main/lib/status-utils';
import { Route } from '@apps/main/routes/_main/user/app/$appId/features-rbac/audit';
import { zodResolver } from '@hookform/resolvers/zod';
import {
  appRbacBaseAuditData,
  type AppRbacAuditDataItemType,
} from '@shared/apis/user/rbac';
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error';
import { PageSkeletonTable } from '@shared/components/custom/page-placeholder/skeleton-table';
import { DataTable, DataTableAction, DataTableActionItem } from '@shared/components/custom/table';
import { Badge } from '@shared/components/ui/badge';
import { Button } from '@shared/components/ui/button';
import {
  cn,
  extractMinMax,
  formatTime,
  getQueryResponseData,
  getQueryResponseNext,
  TIME_STYLE,
} from '@shared/lib/utils';
import type { LimitType } from '@shared/types/base-schema';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import type { ColumnDef } from '@tanstack/react-table';
import { Eye } from 'lucide-react';
import { useState } from 'react';
import { featureRbacModuleConfig } from '../nav-info';
import { AuditDetailDrawer } from './audit-detail-drawer';
import { AuditListFilterFormSchema } from './audit-schema';

export default function AppDetailFeatureRbacAuditPage() {
  const { appId } = Route.useParams();

  // 在 page 组件中加载字典数据
  const {
    dictData,
    isLoading: dictIsLoading,
    isError: dictError,
    errors: dictErrors,
    refetch: refetchDict,
  } = useDictData(['app_rbac'] as const);

  // 字典数据加载错误
  if (dictError) {
    return (
      <AppDetailNavContainer {...featureRbacModuleConfig}>
        <CenteredError
          variant="content"
          error={dictErrors?.[0] || '加载字典数据失败'}
          onReset={refetchDict}
        />
      </AppDetailNavContainer>
    );
  }

  // 字典数据正在加载
  if (dictIsLoading) {
    return (
      <AppDetailNavContainer {...featureRbacModuleConfig}>
        <PageSkeletonTable variant="content" />
      </AppDetailNavContainer>
    );
  }

  return (
    <AppDetailNavContainer {...featureRbacModuleConfig}>
      <AppDetailFeatureRbacAuditContent appId={Number(appId)} dictData={dictData} />
    </AppDetailNavContainer>
  );
}

interface AppDetailFeatureRbacAuditContentProps {
  appId: number;
  dictData: ReturnType<typeof useDictData<['app_rbac']>>['dictData'];
}

function AppDetailFeatureRbacAuditContent({
  appId,
  dictData,
}: AppDetailFeatureRbacAuditContentProps) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();

  // 从 URL 搜索参数获取过滤条件
  const filterParam = Route.useSearch();
  const currentLimit = filterParam.limit || DEFAULT_PAGE_SIZE;

  // 详情抽屉状态
  const [detailDrawerOpen, setDetailDrawerOpen] = useState(false);
  const [selectedAudit, setSelectedAudit] = useState<AppRbacAuditDataItemType | null>(null);

  // 过滤条件从 URL 参数获取
  const filters = {
    user_ip: filterParam.user_ip || null,
    request_id: filterParam.request_id || null,
    check_result: filterParam.check_result || null,
  };

  // 分页状态 - 直接从 URL 参数派生
  const pagination: LimitType = {
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

  // 获取审计列表数据
  const {
    data: auditData,
    isSuccess,
    isLoading: dataLoading,
    isError,
    error,
  } = useQuery({
    queryKey: [
      'appRbacAuditData',
      appId,
      pagination.pos,
      currentLimit,
      pagination.forward,
      pagination.more,
      pagination.eq_pos,
      filters.user_ip,
      filters.request_id,
      filters.check_result,
    ],
    queryFn: ({ signal }) =>
      appRbacBaseAuditData(
        {
          app_id: appId,
          limit: {
            eq_pos: pagination.eq_pos,
            pos: pagination.pos,
            limit: currentLimit,
            forward: pagination.forward,
            more: pagination.more,
          },
          count_num: countNumManager.getCountNum(),
          user_ip: filters.user_ip || undefined,
          request_id: filters.request_id || undefined,
          check_result: filters.check_result ? Number(filters.check_result) : undefined,
          res_data: { res_id: 0 },
        },
        { signal }
      ),
    placeholderData: (previousData) => previousData,
  });

  // 处理 Limit 分页查询结果
  isSuccess && countNumManager.handleLimitQueryResult(auditData);

  // 获取审计列表数据
  const audits = getQueryResponseData<AppRbacAuditDataItemType[]>(auditData, []);
  const nextPageStartPos = getQueryResponseNext(auditData);

  // 打开详情抽屉
  const handleOpenDetail = (audit: AppRbacAuditDataItemType) => {
    setSelectedAudit(audit);
    setDetailDrawerOpen(true);
  };

  // 使用统一的分页处理器
  const { handleNextPage, handlePrevPage, canGoNext, canGoPrev } = useOffsetPaginationHandlers({
    ...extractMinMax(
      audits.map((a) => ({ id: a.audit.id })),
      'id',
      'minId',
      'maxId'
    ),
    pagination,
    nextPageStartPos,
    searchGo,
    defaultForward: false,
  });

  // 刷新数据
  const refreshData = () => {
    queryClient.refetchQueries({ queryKey: ['appRbacAuditData'] });
  };

  // 清除缓存并重新加载数据
  const clearCacheAndReload = () => {
    countNumManager.reset();
    queryClient.invalidateQueries({ queryKey: ['appRbacAuditData'] });
  };

  // loading 状态
  const isLoading = dataLoading;

  // 审计结果状态映射：1=授权通过(success), 0=授权失败(danger)
  const checkResultStatus = createStatusMapper<string>(
    {
      '1': 'success',
      '0': 'danger',
    } as Record<string, 'success' | 'danger'>,
    (result) => dictData.audit_result?.getLabel(String(result)) || String(result)
  );

  // 定义表格列配置
  const columns: ColumnDef<AppRbacAuditDataItemType>[] = [
    {
      id: 'id',
      accessorFn: (row) => row.audit.id,
      header: () => <div className="sm:text-right min-w-[50px] py-1">ID</div>,
      size: 60,
      cell: ({ getValue }) => (
        <div className="font-mono text-xs sm:text-right min-w-[50px] py-1">{getValue<number>()}</div>
      ),
    },
    {
      id: 'check_result',
      accessorFn: (row) => row.audit.check_result,
      header: '授权结果',
       size: 100,
      cell: ({ getValue }) => {
        const result = getValue<string>();
        return (
          <div className="py-1">
            <Badge className={checkResultStatus.getClass(result)}>
              {checkResultStatus.getText(result)}
            </Badge>
          </div>
        );
      },
    },
    {
      id: 'user_info',
      header: '用户',
         size: 100,
      cell: ({ row }) => {
        const { user } = row.original;
        return (
          <div className="py-1">
            <UserDataTooltip userData={user} />
          </div>
        );
      },
    },
    {
      id: 'user_ip',
      accessorFn: (row) => row.audit.user_ip,
      header: 'IP地址',
      cell: ({ getValue }) => (
        <div className="font-mono text-xs py-1">{getValue<string>() || '-'}</div>
      ),
    },
    {
      id: 'request_id',
      accessorFn: (row) => row.audit.request_id,
      header: '请求ID',
      cell: ({ getValue }) => (
        <div className="font-mono text-xs py-1 max-w-[120px] truncate" title={getValue<string>() || ''}>
          {getValue<string>() || '-'}
        </div>
      ),
    },
    {
      id: 'detail_count',
      header: '详情数',
        size: 60,
      cell: ({ row }) => {
        return (
          <div className="py-1">
            <AuditDetailTooltip details={row.original.detail} />
          </div>
        );
      },
    },
    {
      id: 'add_time',
      accessorFn: (row) => row.audit.add_time,
      header: '授权时间',
      cell: ({ getValue }) => {
        const addTime = getValue<Date | null>();
        return (
          <div className="text-xs py-1">
            {addTime ? formatTime(addTime, TIME_STYLE.RELATIVE_ELEMENT) : '-'}
          </div>
        );
      },
    },
    {
      id: 'actions',
      header: () => <div className="text-center py-1">操作</div>,
      cell: ({ row }) => {
        const audit = row.original;

        return (
          <DataTableAction className="justify-end sm:justify-center gap-1">
            <DataTableActionItem mobileDisplay="display" desktopDisplay="collapsed">
              <Button
                variant="ghost"
                size="sm"
                className={cn('h-auto px-2 py-1')}
                title="查看详情"
                onClick={() => handleOpenDetail(audit)}
              >
                <Eye className="h-3 w-3" />
                <span className="text-xs ml-1">查看详情</span>
              </Button>
            </DataTableActionItem>
          </DataTableAction>
        );
      },
    },
  ];

  return (
    <>
      <div className="flex flex-col min-h-0 space-y-6">
        <div className="flex-shrink-0 mb-1 sm:mb-4">
          {/* 过滤器 */}
          <FilterContainer
            defaultValues={{
              user_ip: filterParam.user_ip,
              request_id: filterParam.request_id,
              check_result: filterParam.check_result,
            }}
            resolver={zodResolver(AuditListFilterFormSchema) as any}
            onSubmit={(data) => {
              const transformedData = data as {
                user_ip?: string;
                request_id?: string;
                check_result?: string;
              };
              searchGo({
                user_ip: transformedData.user_ip,
                request_id: transformedData.request_id,
                check_result: transformedData.check_result,
                pos: null,
                forward: false,
                eq_pos: false,
              });
            }}
            onReset={() => {
              searchGo({
                pos: null,
                limit: currentLimit,
                forward: false,
                eq_pos: false,
                user_ip: undefined,
                request_id: undefined,
                check_result: undefined,
              });
            }}
            countComponent={
              <FilterTotalCount total={countNumManager.getTotal() ?? 0} loading={isLoading} />
            }
            className="bg-card rounded-lg border shadow-sm relative"
          >
            {(layoutParams, form) => (
              <div className="flex-1 flex flex-wrap items-end gap-3">
                {/* 审计结果过滤 */}
                {dictData.audit_result && (
                  <FilterDictSelect
                    name="check_result"
                    placeholder="选择结果"
                    label="授权结果"
                    disabled={isLoading}
                    dictData={dictData.audit_result}
                    layoutParams={layoutParams}
                    allLabel="全部"
                    className="w-28"
                  />
                )}

                {/* IP地址过滤 */}
                <FilterInput
                  name="user_ip"
                  placeholder="输入IP地址"
                  label="IP地址"
                  disabled={isLoading}
                  layoutParams={layoutParams}
                  className="w-[13rem]"
                />

                {/* 请求ID过滤 */}
                <FilterInput
                  name="request_id"
                  placeholder="输入请求ID"
                  label="请求ID"
                  disabled={isLoading}
                  layoutParams={layoutParams}
                  className="w-[13rem]"
                />

                {/* 动作按钮区域 */}
                <div className={cn(layoutParams.isMobile ? 'w-full' : 'flex-shrink-0')}>
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

        {/* 数据表格 */}
        <div className="flex-1 flex flex-col overflow-hidden min-h-0">
          <DataTable
            data={audits}
            columns={columns}
            loading={isLoading}
            error={
              isError ? <CenteredError error={error} variant="content" onReset={refreshData} /> : null
            }
            scrollSnapDelay={300}
            leftStickyColumns={[{ column: 0, minWidth: '80px', maxWidth: '80px' }]}
            className="[&_tr]:h-11 [&_td]:py-1 [&_th]:py-1 [&_table]:border-0 [&_.table-container]:border-0 [&_tbody_tr:last-child]:border-b"
          />

          {/* 分页控件 */}
          <div className="flex-shrink-0 pt-4">
            {(countNumManager.getTotal() ?? 0) > 0 && (
              <OffsetPagination
                limit={currentLimit}
                hasNext={canGoNext}
                canGoPrev={canGoPrev}
                total={countNumManager.getTotal()}
                currentPageSize={audits.length}
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

        {/* 详情抽屉 */}
        <AuditDetailDrawer
          open={detailDrawerOpen}
          onOpenChange={setDetailDrawerOpen}
          audit={selectedAudit}
        />
      </div>
    </>
  );
}
