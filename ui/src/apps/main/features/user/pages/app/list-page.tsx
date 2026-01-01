import { useAuthData } from '@apps/main/hooks/use-auth-data'
import { useDictData, type TypedDictData } from '@apps/main/hooks/use-dict-data'
import { Route } from '@apps/main/routes/_main/user/app/_root/list'
import { zodResolver } from '@hookform/resolvers/zod'
import { useNavigate } from '@tanstack/react-router'
import { useMemo } from 'react'
// App list grid inlined (removed separate component)
import { FilterContainer } from '@apps/main/components/filter-container/container'
import { FilterActions } from '@apps/main/components/filter-container/filter-actions'
import { FilterDictSelect } from '@apps/main/components/filter-container/filter-dict-select'
import { FilterInput } from '@apps/main/components/filter-container/filter-input'
import { FilterTotalCount } from '@apps/main/components/filter-container/filter-total-count'
import { FilterUserParentAppSelector } from '@apps/main/components/filter-container/filter-user-parent-app-selector'
import { userQueryKey } from '@apps/main/lib/auth-utils'
import { PagePagination, useCountNumManager } from '@apps/main/lib/pagination-utils'
import { appList, type AppListItemType, type AppListParamType } from '@shared/apis/user/app'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { PageSkeletonCard } from '@shared/components/custom/page-placeholder/skeleton-card'
import { Button } from '@shared/components/ui/button'
import { cn, getQueryResponseData } from '@shared/lib/utils'
import { useQuery } from '@tanstack/react-query'
import { Plus, Shield } from 'lucide-react'
import { AppCard } from './list-app-card'
import { AppListFilterFormSchema } from './list-schema'

// 应用列表页面组件
export default function AppListPage() {
  //docs\api\user\app\list.md
  // 字典数据获取 - 统一在最顶层获取一次
  const { dictData, isLoading: isDictLoading, isError: isDictError, errors: dictErrors, refetch: refetchDict } = useDictData(['user_app'] as const);

  // 如果字典加载失败，显示错误页面
  if (isDictError) {
    return (
      <CenteredError
        variant="page"
        error={dictErrors}
        onReset={refetchDict}
      />
    );
  }

  // 如果字典正在加载，显示骨架屏
  if (isDictLoading) {
    return <PageSkeletonCard variant="page" className={cn("m-4")} />;
  }

  // 字典加载成功，渲染内容组件
  return <AppListContent dictData={dictData} />;
}

// 内容组件：负责内容加载和渲染
interface AppListContentProps {
  dictData: TypedDictData<["user_app"]>;
}

function AppListContent({ dictData }: AppListContentProps) {
  const navigate = useNavigate()
  const currentFilter = Route.useSearch()
  const authData = useAuthData()
  // 如果用户 appData 不为空，则隐藏父应用过滤器
  const hideParentAppFilter = !!authData?.appData

  const currentPage = currentFilter.page || 1
  const currentLimit = currentFilter.limit || 12

  // 过滤条件
  const filters = {
    parent_app_id: currentFilter.parent_app_id ?? null,
    status: currentFilter.status ?? null,
    client_id: currentFilter.client_id || null,
    app_id: currentFilter.app_id ?? null,
  };

  // count_num 优化管理器（传入 filters 自动监听变化）
  const countNumManager = useCountNumManager(filters);

  const appListQueryParam: AppListParamType = useMemo(() => ({
    page: { page: currentPage, limit: currentLimit },
    count_num: countNumManager.getCountNum(),
    parent_app_id: currentFilter.parent_app_id,
    status: currentFilter.status || null,
    client_id: currentFilter.client_id,
    app_id: currentFilter.app_id,
    attr_inner_feature: true,
    attr_oauth_client_data: true,
    attr_oauth_server_data: true,
    attr_exter_feature: true,
    attr_parent_app: true,
    attr_sub_app_count: true,
  }), [currentPage, currentLimit, currentFilter.parent_app_id, currentFilter.status, currentFilter.client_id, currentFilter.app_id, countNumManager])

  const { data: appListResult, isSuccess, isLoading: isAppListLoading, isError: isAppListError, error: appListError, refetch } = useQuery({
    queryKey: userQueryKey('appList', appListQueryParam),
    queryFn: async ({ signal }) => {
      const result = await appList(appListQueryParam, { signal })
      return result
    },
    staleTime: 3000,
  })

  // 处理 Page 分页查询结果（自动提取 total）
  isSuccess && countNumManager.handlePageQueryResult(appListResult);

  const apps = getQueryResponseData<AppListItemType[]>(appListResult, [])

  return (
    <div className='container mx-auto px-4  py-6 max-w-[1600px] space-y-5'>
      {/* 过滤区域 */}
      <FilterContainer
        defaultValues={{
          status: currentFilter.status?.toString(),
          parent_app_id: currentFilter.parent_app_id?.toString(),
          client_id: currentFilter.client_id,
          app_id: currentFilter.app_id?.toString(),
        }}
        resolver={zodResolver(AppListFilterFormSchema) as any}
        onSubmit={(data) => {
          // zod schema 已经处理了类型转换和空值清理，直接使用数据
          navigate({
            search: { ...data, page: 1 } as any,
          })
        }}
        onReset={() => {
          navigate({
            search: { page: 1, limit: currentLimit } as any,
          })
        }}
        countComponent={
          <FilterTotalCount
            total={countNumManager.getTotal() ?? 0}
            loading={isAppListLoading}
          />
        }
      >
        {(layoutParams, form) => (
          <>
            {/* 表单字段区域 */}
            <div className="flex-1 grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
              {/* 状态过滤 */}
              <FilterDictSelect
                name="status"
                placeholder="选择状态"
                label="状态"
                disabled={isAppListLoading}
                dictData={dictData.app_status}
                layoutParams={layoutParams}
                allLabel='全部'
              />

              {/* 父应用过滤 */}
              {!hideParentAppFilter && (
                <FilterUserParentAppSelector
                  name="parent_app_id"
                  placeholder="请选择"
                  label="父级应用"
                  disabled={isAppListLoading}
                  layoutParams={layoutParams}
                />
              )}

              {/* Client ID 过滤 */}
              <FilterInput
                name="client_id"
                placeholder="输入 Client ID"
                label="Client ID"
                disabled={isAppListLoading}
                layoutParams={layoutParams}
              />

              {/* App ID 过滤 */}
              <FilterInput
                name="app_id"
                placeholder="输入 App ID"
                type="number"
                label="App ID"
                disabled={isAppListLoading}
                layoutParams={layoutParams}
              />
            </div>

            {/* 动作按钮区域 */}
            <FilterActions
              form={form}
              layoutParams={layoutParams}
            />
          </>
        )}
      </FilterContainer>

      {/* 应用列表区域 */}
      <div className="flex-1 flex flex-col space-y-4">
        {/* 应用列表区域（内联原 AppListGrid 逻辑） */}
        {(() => {
          // 错误
          if (isAppListError) {
            return (
              <CenteredError
                variant="content"
                error={appListResult || appListError}
                onReset={() => refetch()}
              />
            )
          }

          // 初次加载 skeleton
          if (isAppListLoading && apps.length === 0) {
            // 通过自定义 class 去掉内部 container 额外的 padding，使骨架屏与上方过滤块宽度对齐
            return (
              <PageSkeletonCard
                variant="content"
                itemCount={8}
              />
            )
          }

          const hasFilters = (
            currentFilter.status !== undefined ||
            currentFilter.parent_app_id !== undefined ||
            (currentFilter.client_id && currentFilter.client_id.trim() !== '') ||
            currentFilter.app_id !== undefined
          )

          if (!isAppListLoading && apps.length === 0) {
            return (
              <div className="text-center py-16">
                <div className="bg-muted/50 rounded-full w-16 h-16 flex items-center justify-center mx-auto mb-6">
                  <Shield className={cn('h-8 w-8 text-muted-foreground')} />
                </div>
                <h3 className="text-lg font-semibold mb-3">
                  {hasFilters ? '暂无相关应用' : '暂无应用'}
                </h3>
                <p className="text-muted-foreground mb-8">
                  {hasFilters
                    ? '当前过滤条件下没有找到匹配的应用，请尝试调整过滤条件。'
                    : '您还没有创建任何应用，开始创建您的第一个应用吧！'}
                </p>
                {!hasFilters && (
                  <Button onClick={() => navigate({ to: '/user/app/create' })}>
                    <Plus className={cn('h-4 w-4 mr-2')} />
                    创建第一个应用
                  </Button>
                )}
              </div>
            )
          }

          // 正常数据网格
          return (
            <div className={`space-y-8 ${isAppListLoading ? 'opacity-60 transition-opacity duration-300' : ''}`}>
              <div className='grid gap-6 sm:grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4'>
                {apps.map((app: AppListItemType) => (
                  <AppCard
                    key={app.id}
                    app={app}
                    dictData={dictData}
                  />
                ))}
              </div>
            </div>
          )
        })()}

        {/* 分页区域 */}
        <PagePagination
          currentPage={currentPage}
          pageSize={currentLimit}
          total={countNumManager.getTotal() ?? 0}
          loading={isAppListLoading}
          onChange={(page) => {
            navigate({
              search: { ...currentFilter, page } as any,
            })
          }}
          onPageSizeChange={(pageSize) => {
            navigate({
              search: { ...currentFilter, limit: pageSize, page: 1 } as any,
            })
          }}
          onRefresh={() => refetch()}
          showRefresh={true}
          pageSizeOptions={[12, 24, 48]}
          showPageSize={true}
          className="pb-4"
        />
      </div>
    </div>
  )
}

// 导出 schema 供路由使用
export { AppListFilterParamSchema } from './list-schema'

