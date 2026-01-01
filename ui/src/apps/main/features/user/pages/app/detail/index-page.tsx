import { AppDetailNavContainer } from '@apps/main/features/user/components/ui/app-detail-nav'
import { useDictData } from '@apps/main/hooks/use-dict-data'
import { appQueryKey } from '@apps/main/lib/auth-utils'
import type { AppListItemType } from '@shared/apis/user/app'
import { appList, appStat } from '@shared/apis/user/app'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { CenteredLoading } from '@shared/components/custom/page-placeholder/centered-loading'
import { Badge } from '@shared/components/ui/badge'
import { Button } from '@shared/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@shared/components/ui/card'
import { ChartContainer, ChartTooltip, ChartTooltipContent, type ChartConfig } from '@shared/components/ui/chart'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@shared/components/ui/select'
import { cn, formatTime, TIME_STYLE } from '@shared/lib/utils'
import { useQuery } from '@tanstack/react-query'
import { Link, useParams } from '@tanstack/react-router'
import { useMemo, useState } from 'react'
import { CartesianGrid, Line, LineChart, ReferenceLine, XAxis, YAxis } from 'recharts'
import { indexModuleConfig } from './nav-info'

// Color palette for charts
const CHART_COLORS = ['#3b82f6', '#ef4444', '#10b981', '#f59e0b', '#8b5cf6', '#ec4899', '#14b8a6']

const getChartColor = (index: number): string => {
  return CHART_COLORS[index % CHART_COLORS.length]
}

interface DetailContentProps {
  appId: string
  appDetail: AppListItemType
  dictData: any
  days: number
  enabledFeatures: any[]
  disabledFeatures: any[]
  hasOAuthClient: boolean
  scopeArray: string[]
  hasParentApp: boolean
  hasSubAppFeature: boolean
  hasExterLoginFeature: boolean
}

type DailyStatItem = {
  date: string
  total?: number | string
}

const formatDateKey = (date: Date) => {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

const parseDateKey = (value?: string) => {
  if (!value) return null
  const [year, month, day] = value.split('-').map((item) => Number.parseInt(item, 10))
  if ([year, month, day].some((item) => Number.isNaN(item))) {
    return null
  }
  return new Date(year, (month as number) - 1, day)
}

const buildZeroFilledDailySeries = (data: DailyStatItem[] | undefined, days: number) => {
  const safeDays = Math.max(days, 1)
  const sortedData = Array.isArray(data) ? [...data].sort((a, b) => a.date.localeCompare(b.date)) : []
  const totalsByDate = new Map(sortedData.map((item) => [item.date, Number(item.total ?? 0)]))
  const lastDate = sortedData.length > 0 ? sortedData[sortedData.length - 1].date : undefined
  const endDate = parseDateKey(lastDate) ?? new Date()
  const series: { date: string; total: number }[] = []

  for (let i = safeDays - 1; i >= 0; i--) {
    const currentDate = new Date(endDate)
    currentDate.setDate(endDate.getDate() - i)
    const dateKey = formatDateKey(currentDate)
    series.push({
      date: dateKey,
      total: totalsByDate.get(dateKey) ?? 0
    })
  }

  return series
}

type ChartSeriesConfig = {
  key: string
  label: string
  color: string
  data: DailyStatItem[]
}

interface DetailLineChartProps {
  title: string
  subtitle?: string
  series: ChartSeriesConfig[]
  className?: string
  totals?: Array<{ label: string; value: number; color: string }>
}

type ChartPoint = {
  date: string
  [key: string]: string | number
}

const DetailLineChart = ({ title, subtitle, series, className, totals }: DetailLineChartProps) => {
  const chartConfig = useMemo(() => {
    return series.reduce((acc, item) => {
      acc[item.key] = {
        label: item.label,
        color: item.color
      }
      return acc
    }, {} as ChartConfig)
  }, [series])

  const mergedData = useMemo(() => {
    const dateSet = new Set<string>()
    const seriesMaps = series.map((item) => {
      const map = new Map<string, number>()
      item.data.forEach((dataPoint) => {
        dateSet.add(dataPoint.date)
        map.set(dataPoint.date, Number(dataPoint.total ?? 0))
      })
      return { key: item.key, map }
    })

    const sortedDates = Array.from(dateSet).sort((a, b) => a.localeCompare(b))
    if (sortedDates.length === 0) {
      return []
    }

    return sortedDates.map((date) => {
      const point: ChartPoint = { date }
      seriesMaps.forEach(({ key, map }) => {
        point[key] = map.get(date) ?? 0
      })
      return point
    })
  }, [series])

  const maxValue = useMemo(() => {
    if (!mergedData.length) return 0

    return mergedData.reduce((outerMax, point) => {
      const pointMax = series.reduce((innerMax, item) => {
        return Math.max(innerMax, Number(point[item.key] ?? 0))
      }, 0)
      return Math.max(outerMax, pointMax)
    }, 0)
  }, [mergedData, series])

  const showZeroBaseline = maxValue === 0

  return (
    <div className={cn("space-y-2")}>
      <div className={cn("flex items-start justify-between gap-2")}>
        <div className={cn("flex-shrink-0")}>
          <div className={cn("text-xs font-medium")}>{title}</div>
          {subtitle && <div className={cn("text-xs text-muted-foreground")}>{subtitle}</div>}
        </div>
        {totals && totals.length > 0 && (
          <div className={cn("flex flex-wrap gap-2 justify-end flex-shrink-0")}>
            {totals.map((item) => (
              <div key={item.label} className={cn("text-right")}>
                <div className={cn("text-xs text-muted-foreground")}>{item.label}</div>
                <div className={cn("flex items-center gap-1")}>
                  <div className={cn("w-1.5 h-1.5 rounded-full")} style={{ backgroundColor: item.color }}></div>
                  <div className={cn("text-xs font-semibold")}>{item.value}</div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
      <ChartContainer config={chartConfig} className={cn("h-[70px] w-full")}>
        <LineChart data={mergedData} margin={{ top: 2, right: 2, bottom: 6 }}>
          <CartesianGrid strokeDasharray="3 3" vertical={false} />
          <XAxis
            dataKey="date"
            tickLine={false}
            axisLine={false}
            tickMargin={3}
            tickFormatter={(value) => value.slice(5)}
            height={16}
          />
          <YAxis
            tickLine={false}
            axisLine={false}
            tickMargin={4}
            domain={[0, showZeroBaseline ? 1 : maxValue]}
            width={40}
          />
          <ChartTooltip content={<ChartTooltipContent />} labelFormatter={(value) => `日期: ${value}`} />
          {showZeroBaseline && (
            <ReferenceLine
              y={0}
              stroke="hsl(var(--muted-foreground))"
              strokeDasharray="6 4"
              strokeOpacity={0.8}
              isFront
            />
          )}
          {series.map((item) => (
            <Line
              key={item.key}
              type="monotone"
              dataKey={item.key}
              stroke={item.color}
              strokeWidth={2}
              dot={false}
              isAnimationActive={false}
              connectNulls={true}
              style={{ opacity: 0.9 }}
            />
          ))}
        </LineChart>
      </ChartContainer>
    </div>
  )
}

function DetailContent({
  appId,
  appDetail,
  dictData,
  days,
  enabledFeatures,
  disabledFeatures,
  hasOAuthClient,
  scopeArray,
  hasParentApp,
  hasSubAppFeature,
  hasExterLoginFeature
}: DetailContentProps) {
  // 获取应用统计数据
  const {
    data: statData,
    isLoading: isLoadingStat,
    isError: isStatError,
    error: statError,
    refetch: refetchStat
  } = useQuery({
    queryKey: ['app-stat', appId, days],
    queryFn: ({ signal }) =>
      appStat(
        {
          app_id: Number(appId),
          days
        },
        { signal }
      ),
    enabled: !!appId
  })

  const stats = statData?.response?.data

  const oauthAccessSeries = useMemo(() => {
    return [
      {
        key: 'oauthTotal',
        label: 'OAuth登陆总数',
        color: getChartColor(0),
        data: buildZeroFilledDailySeries(stats?.oauth_access, days)
      }
    ] satisfies ChartSeriesConfig[]
  }, [stats?.oauth_access, days])

  const oauthTotals = useMemo(() => {
    const total = stats?.oauth_access?.reduce((sum, item) => sum + Number(item.total ?? 0), 0) ?? 0
    return [{ label: 'OAuth登陆总数', value: total, color: getChartColor(0) }]
  }, [stats?.oauth_access])

  const subAppSeries = useMemo(() => {
    return [
      {
        key: 'subAppAll',
        label: '子应用总数',
        color: getChartColor(0),
        data: buildZeroFilledDailySeries(stats?.sub_app?.all, days)
      },
      {
        key: 'subAppEnable',
        label: '启用子应用数量',
        color: getChartColor(1),
        data: buildZeroFilledDailySeries(stats?.sub_app?.enable, days)
      }
    ] satisfies ChartSeriesConfig[]
  }, [stats?.sub_app?.all, stats?.sub_app?.enable, days])

  const subAppTotals = useMemo(() => {
    const allTotal = stats?.sub_app?.all?.reduce((sum, item) => sum + Number(item.total ?? 0), 0) ?? 0
    const enableTotal = stats?.sub_app?.enable?.reduce((sum, item) => sum + Number(item.total ?? 0), 0) ?? 0
    return [
      { label: '子应用总数', value: allTotal, color: getChartColor(0) },
      { label: '启用子应用数量', value: enableTotal, color: getChartColor(1) }
    ]
  }, [stats?.sub_app?.all, stats?.sub_app?.enable])

  const requestSeries = useMemo(() => {
    const processedByDate = new Map<string, number>()
    stats?.request?.processed?.forEach((item: any) => {
      processedByDate.set(item.date, (processedByDate.get(item.date) ?? 0) + Number(item.total ?? 0))
    })

    const processedArray = Array.from(processedByDate.entries()).map(([date, total]) => ({
      date,
      total
    }))

    return [
      {
        key: 'requestAll',
        label: '子应用请求总数',
        color: getChartColor(2),
        data: buildZeroFilledDailySeries(
          stats?.request?.all?.map((item: any) => ({
            date: item.date,
            total: Number(item.total ?? 0)
          })),
          days
        )
      },
      {
        key: 'requestProcessed',
        label: '子应用审核总数',
        color: getChartColor(3),
        data: buildZeroFilledDailySeries(processedArray, days)
      }
    ] satisfies ChartSeriesConfig[]
  }, [stats?.request?.all, stats?.request?.processed, days])

  const requestTotals = useMemo(() => {
    const allTotal = stats?.request?.all?.reduce((sum, item) => sum + Number(item.total ?? 0), 0) ?? 0
    const processedTotal = stats?.request?.processed?.reduce((sum, item) => sum + Number(item.total ?? 0), 0) ?? 0
    return [
      { label: '子应用请求总数', value: allTotal, color: getChartColor(2) },
      { label: '子应用审核总数', value: processedTotal, color: getChartColor(3) }
    ]
  }, [stats?.request?.all, stats?.request?.processed])

  const notifySeries = useMemo(() => {
    const allMap = new Map<string, number>()
    const successMap = new Map<string, number>()

    stats?.notify_data?.all?.forEach((item: any) => {
      if (item.status !== '0') return
      allMap.set(item.date, (allMap.get(item.date) ?? 0) + Number(item.total ?? 0))
    })

    stats?.notify_data?.success?.forEach((item: any) => {
      successMap.set(item.date, (successMap.get(item.date) ?? 0) + Number(item.total ?? 0))
    })

    const allArray = Array.from(allMap.entries()).map(([date, total]) => ({ date, total }))
    const successArray = Array.from(successMap.entries()).map(([date, total]) => ({ date, total }))

    return [
      {
        key: 'notifyAll',
        label: '回调总数',
        color: getChartColor(4),
        data: buildZeroFilledDailySeries(allArray, days)
      },
      {
        key: 'notifySuccess',
        label: '回调成功总数',
        color: getChartColor(5),
        data: buildZeroFilledDailySeries(successArray, days)
      }
    ] satisfies ChartSeriesConfig[]
  }, [stats?.notify_data?.all, stats?.notify_data?.success, days])

  const notifyTotals = useMemo(() => {
    let allTotal = 0
    let successTotal = 0
    stats?.notify_data?.all?.forEach((item: any) => {
      if (item.status === '0') {
        allTotal += Number(item.total ?? 0)
      }
    })
    stats?.notify_data?.success?.forEach((item: any) => {
      successTotal += Number(item.total ?? 0)
    })
    return [
      { label: '回调总数', value: allTotal, color: getChartColor(4) },
      { label: '回调成功总数', value: successTotal, color: getChartColor(5) }
    ]
  }, [stats?.notify_data?.all, stats?.notify_data?.success])

  // 加载状态
  if (isLoadingStat) {
    return <CenteredLoading variant="card" />
  }

  // 错误状态
  if (isStatError) {
    return <CenteredError variant="content" error={statError} onReset={refetchStat} />
  }

  return (
    <div>
      {/* Flex 布局 - 横向4列网格 */}
      <div className={cn("flex flex-wrap gap-3 md:gap-5 w-full justify-start py-4 px-0")}>
        {/* 应用基本信息 - 占2列 */}
        <Card className={cn("flex flex-col w-full md:flex-[2] md:min-w-[400px] gap-0 py-0 h-auto md:h-[200px]")}>
          <CardHeader className={cn("pb-2 px-3 pt-2 mt-2 mb-3 flex-shrink-0")}>
            <CardTitle className={cn("text-base leading-tight")}>应用信息</CardTitle>
          </CardHeader>
          <CardContent className={cn("px-3 py-2 flex-1 flex flex-col")}>
            <div className={cn("grid grid-cols-3 gap-x-2 gap-y-1 text-[15px] auto-rows-max")}>
              {/* 名称 */}
              <div className={cn("flex flex-col gap-2 min-h-fit mb-1")}>
                <label className={cn("text-muted-foreground font-medium text-[12px] leading-tight")}>应用名称</label>
                <span className={cn("text-[15px] font-semibold truncate text-foreground leading-tight")}>{appDetail.name}</span>
              </div>

              {/* 状态 */}
              <div className={cn("flex flex-col gap-2 min-h-fit mb-1")}>
                <label className={cn("text-muted-foreground font-medium text-[12px] leading-tight")}>应用状态</label>
                <span className={cn("text-[15px] font-medium leading-tight")}>
                  {dictData?.app_status?.find((s: any) => s.key === String(appDetail.status))?.val || '未知'}
                </span>
              </div>

              {/* 应用类型 */}
              <div className={cn("flex flex-col gap-2 min-h-fit mb-1")}>
                <label className={cn("text-muted-foreground font-medium text-[12px] leading-tight")}>应用类型</label>
                <span className={cn("text-[15px] font-medium leading-tight")}>{hasParentApp ? '子应用' : '普通应用'}</span>
              </div>

              {/* Client ID */}
              <div className={cn("flex flex-col gap-2 min-h-fit mb-1 col-span-2")}>
                <label className={cn("text-muted-foreground font-medium text-[12px] leading-tight")}>Client ID</label>
                <span className={cn("font-mono text-[13px] truncate px-0.5 py-0.5 rounded text-foreground leading-tight")}>{appDetail.client_id}</span>
              </div>

              {/* 最后更新 */}
              <div className={cn("flex flex-col gap-1 min-h-fit mb-1")}>
                <label className={cn("text-muted-foreground font-medium text-[12px] leading-tight")}>最后更新</label>
                <span className={cn("text-[15px] leading-tight")}>
                  {formatTime(appDetail.change_time, TIME_STYLE.RELATIVE_ELEMENT)}
                </span>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* 父应用信息 - 占1列 */}
        {hasParentApp && appDetail.parent_app && (
          <Card className={cn("flex flex-col h-auto md:h-[200px] w-full md:flex-1 md:min-w-[200px]")}>
            <CardHeader className={cn("pb-2 px-3 pt-2 mt-2 mb-3 flex-shrink-0")}>
              <CardTitle className={cn("text-base leading-tight")}>父应用信息</CardTitle>
            </CardHeader>
            <CardContent className={cn("space-y-2 p-3 pt-0")}>
              <div className={cn("flex items-center gap-2 text-xs")}>
                <span className={cn("text-muted-foreground")}>名称:</span>
                <span className={cn("font-medium truncate")}>{appDetail.parent_app.name}</span>
              </div>
              <div className={cn("flex items-center gap-2 text-xs")}>
                <span className={cn("text-muted-foreground")}>CLIENT_ID:</span>
                <span className={cn("font-mono text-xs truncate")}>{appDetail.parent_app.client_id}</span>
              </div>
              <div className={cn("flex items-center gap-2 text-xs")}>
                <span className={cn("text-muted-foreground")}>状态:</span>
                <Badge variant={appDetail.parent_app.status === 2 ? 'default' : 'secondary'}>
                  {dictData?.app_status?.find((s: any) => s.key === String(appDetail.parent_app?.status))?.val ||
                    '未知'}
                </Badge>
              </div>
            </CardContent>
          </Card>
        )
        }

        {/* 扩展功能 - 占1列 */}
        {
          (enabledFeatures.length > 0 || disabledFeatures.length > 0) && (
            <Card className={cn("flex pt-3 gap-1 flex-col h-auto md:h-[200px] w-full md:flex-1 md:min-w-[200px]")}>
              <CardHeader className={cn("pb-2 px-3 pt-2 mt-2 mb-3 flex-shrink-0")}>
                <CardTitle className={cn("text-base leading-tight")}>扩展功能</CardTitle>
              </CardHeader>
              <CardContent className={cn("space-y-2 p-3 pt-0 -mt-2")}>
                {enabledFeatures.length > 0 && (
                  <div>
                    <div className={cn("text-xs font-medium mb-1")}>已开通功能:</div>
                    <div className={cn("flex flex-wrap gap-2")}>
                      {enabledFeatures.map((feature: any) => (
                        <Badge key={feature.key} variant="default" className={cn("text-xs")}>
                          {feature.val}
                        </Badge>
                      ))}
                    </div>
                  </div>
                )}
                {disabledFeatures.length > 0 && (
                  <div>
                    <div className={cn("text-xs font-medium mb-1")}>未开通功能:</div>
                    <div className={cn("flex flex-wrap gap-1")}>
                      {disabledFeatures.map((feature: any) => (
                        <Badge key={feature.key} variant="outline" className={cn("text-xs")}>
                          {feature.val}
                        </Badge>
                      ))}
                    </div>
                    <Link to="/user/app/$appId/service/feature" params={{ appId: Number(appId) }}>
                      <Button variant="outline" size="sm" className={cn("mt-1 text-xs")}>
                        申请开通功能
                      </Button>
                    </Link>
                  </div>
                )}
              </CardContent>
            </Card>
          )
        }

        {/* OAuth 登录服务 - 占1列或2列 */}
        <Card className={cn(`flex flex-col gap-0 py-0 h-auto md:h-[200px] w-full ${hasOAuthClient ? 'md:flex-[2] md:min-w-[400px]' : 'md:flex-1 md:min-w-[200px]'}`)}>
          <CardHeader className={cn("pb-2 px-3 pt-2 mt-2 mb-3 flex-shrink-0")}>
            <CardTitle className={cn("pt-0 pb-0")}>OAuth 登录服务</CardTitle>
          </CardHeader>
          <CardContent className={cn("flex flex-col md:flex-row w-full gap-4 p-4 pt-0")}>
            {hasOAuthClient ? (
              <>
                <div className={cn("flex flex-row md:flex-col w-full md:w-1/3 justify-between md:justify-center items-start md:items-start")}>
                  <div className={cn("space-y-1 flex-1")}>
                    <div className={cn("flex items-center gap-1")}>
                      <span className={cn("text-xs text-muted-foreground")}>状态:</span>
                      <Badge variant="default" className={cn("text-xs h-5")}>已开通</Badge>
                    </div>
                  </div>
                  <div className={cn("text-xs text-muted-foreground space-y-0.5 flex-1")}>
                    <div className={cn("font-medium text-xs")}>授权范围:</div>
                    {scopeArray.slice(0, 2).map((scope: string) => (
                      <div key={scope} className={cn("truncate text-xs")}>{scope}</div>
                    ))}
                    {scopeArray.length > 2 && <div className={cn("text-xs")}>+{scopeArray.length - 2}</div>}
                  </div>
                  <Link to="/user/app/$appId/service/oauth-client" params={{ appId: Number(appId) }}>
                    <Button variant="link" size="sm" className={cn("p-0 h-auto text-xs")}>
                      权限 →
                    </Button>
                  </Link>
                </div>
                {/* OAuth 访问统计图表 */}
                {stats?.oauth_access && stats.oauth_access.length > 0 && (
                  <div className={cn("w-full md:w-2/3")}>
                    <DetailLineChart
                      title={`近${days}天访问统计`}
                      series={oauthAccessSeries}
                      totals={oauthTotals}
                    />
                  </div>
                )}
              </>
            ) : (
              <div className={cn("space-y-2 w-full flex flex-col justify-center")}>
                <div className={cn("flex items-center gap-2")}>
                  <span className={cn("text-sm text-muted-foreground")}>状态:</span>
                  <Badge variant="outline" className={cn("text-xs")}>未开通</Badge>
                </div>
                <Link to="/user/app/$appId/service/oauth-client" params={{ appId: Number(appId) }}>
                  <Button variant="outline" size="sm">
                    申请开通 OAuth 服务
                  </Button>
                </Link>
              </div>
            )}
          </CardContent>
        </Card>

        {/* OAuth 服务权限 (OAuth Server) - 占1列 */}
        {
          appDetail?.oauth_server_scope_data && appDetail.oauth_server_scope_data.length > 0 && (
            <Card className={cn("flex flex-col h-[200px] flex-1 min-w-[200px]")}>
              <CardHeader  className={cn("pb-2 px-3 pt-2 mt-2 mb-3 flex-shrink-0")}>
                <CardTitle className={cn("pt-6 pb-0")}>OAuth 授权服务</CardTitle>
              </CardHeader>
              <CardContent className={cn("space-y-4")}>
                <div>
                  <div className={cn("text-sm font-medium mb-2")}>授权权限列表:</div>
                  <div className={cn("flex flex-wrap gap-2")}>
                    {appDetail.oauth_server_scope_data.map((scope: any) => (
                      <div key={scope.scope_key} className={cn("flex items-center gap-2 p-2 bg-muted rounded")}>
                        <div className={cn("flex flex-col")}>
                          <span className={cn("font-semibold text-sm")}>{scope.scope_key}</span>
                          <span className={cn("text-xs text-muted-foreground")}>{scope.scope_name}</span>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </CardContent>
            </Card>
          )
        }

        {/* 子应用功能（仅父应用为空时显示，且必须开通子应用功能） */}
        {
          !hasParentApp && (
            <>
              {hasSubAppFeature && stats?.sub_app ? (
                <Card className={cn("flex flex-col h-auto md:h-[200px] w-full md:flex-[2] md:min-w-[400px] gap-0 py-0")}>
                  <CardHeader  className={cn("pb-2 px-3 pt-2 mt-2 mb-3 flex-shrink-0")}>
                    <CardTitle className={cn("p-0")}>子应用管理</CardTitle>
                  </CardHeader>
                  <CardContent className={cn("flex flex-col md:flex-row w-full gap-3 p-3")}>
                    <div className={cn("flex flex-row md:flex-col w-full md:w-1/3 justify-between md:justify-center items-start")}>
                      {appDetail.sub_app_count && (
                        <>
                          <div className={cn("flex items-center gap-1")}>
                            <span className={cn("text-xs text-muted-foreground")}>状态:</span>
                            <Badge variant="default" className={cn("text-xs h-5")}>已开通</Badge>
                          </div>
                          <div className={cn("text-xs text-muted-foreground space-y-0.5")}>
                            <div>已启用: {appDetail.sub_app_count.enable}</div>
                            <div>
                              总数: {appDetail.sub_app_count.enable + appDetail.sub_app_count.disable + appDetail.sub_app_count.init}
                            </div>
                          </div>
                        </>
                      )}
                    </div>
                    <div className={cn("w-full md:w-2/3")}>
                      <DetailLineChart
                        title={`近${days}天子应用统计`}
                        series={subAppSeries}
                        totals={subAppTotals}
                      />
                    </div>
                  </CardContent>
                </Card>
              ) : (
                <Card className={cn("flex flex-col gap-1 h-auto md:h-[200px] w-full md:flex-1 md:min-w-[200px]")}>
                  <CardHeader  className={cn("pb-2 px-3 pt-2 mt-2 mb-3 flex-shrink-0")}>
                    <CardTitle className={cn("p-0")}>子应用管理</CardTitle>
                  </CardHeader>
                  <CardContent className={cn("flex flex-1 items-center justify-center space-y-4")}>
                    <div className={cn("text-center")}>
                      <p className={cn("text-sm text-muted-foreground mb-4")}>需要开通子应用功能才能使用此功能</p>
                      <Link to="/user/app/$appId/service/feature" params={{ appId: Number(appId) }}>
                        <Button variant="default">
                          申请开通子应用功能
                        </Button>
                      </Link>
                    </div>
                  </CardContent>
                </Card>
              )}
            </>
          )
        }

        {/* 请求统计图表（仅开通子应用功能时显示） - 占2列 */}
        {
          stats?.request && hasSubAppFeature && (
            <Card className={cn("flex flex-col h-auto md:h-[200px] w-full md:flex-[2] md:min-w-[400px] gap-0 py-0")}>
              <CardHeader  className={cn("pb-2 px-3 pt-2 mt-2 mb-3 flex-shrink-0")}>
                <CardTitle className={cn("p-0")}>请求处理统计</CardTitle>
              </CardHeader>
              <CardContent className={cn("w-full p-0 px-3")}>
                <DetailLineChart
                  title={`近${days}天请求处理统计`}
                  series={requestSeries}
                  totals={requestTotals}
                />
              </CardContent>
            </Card>
          )
        }

        {/* 外部登录功能（仅父应用为空时显示） - 占1列 */}
        {
          !hasParentApp && (
            <Card className={cn("flex flex-col h-auto md:h-[200px] w-full md:flex-1 md:min-w-[200px]")}>
              <CardHeader  className={cn("pb-2 px-3 pt-2 mt-2 mb-3 flex-shrink-0")}>
                <CardTitle className={cn("pt-6 pb-0")}>外部登录</CardTitle>
              </CardHeader>
              <CardContent className={cn("space-y-2")}>
                {hasExterLoginFeature ? (
                  <div className={cn("flex items-center gap-4")}>
                    <span className={cn("text-sm text-muted-foreground")}>状态:</span>
                    <Badge variant="default">已开通</Badge>
                  </div>
                ) : (
                  <div className={cn("space-y-2")}>
                    <div className={cn("flex items-center gap-4")}>
                      <span className={cn("text-sm text-muted-foreground")}>状态:</span>
                      <Badge variant="outline">未开通</Badge>
                    </div>
                    <Link to="/user/app/$appId/service/exter-login" params={{ appId: Number(appId) }}>
                      <Button variant="outline" size="sm">
                        申请开通外部登录
                      </Button>
                    </Link>
                  </div>
                )}
              </CardContent>
            </Card>
          )
        }

        {/* 回调通知统计 - 占3列 */}
        {
          stats?.notify_data && (
            <Card className={cn("flex flex-col h-auto md:h-[200px] w-full md:flex-[3] md:min-w-[600px] gap-0 py-0")}>
              <CardHeader  className={cn("pb-2 px-3 pt-2 mt-2 mb-3 flex-shrink-0")}>
                <CardTitle className={cn("pt-0 pb-0")}>回调通知统计</CardTitle>
              </CardHeader>
              <CardContent className={cn("w-full p-3")}>
                <DetailLineChart
                  title={`近${days}天通知统计`}
                  series={notifySeries}
                  totals={notifyTotals}
                />
              </CardContent>
            </Card>
          )
        }
      </div >
    </div >
  )
}

export default function AppDetailIndexPage() {
  const { appId } = useParams({ from: '/_main/user/app/$appId' })
  const [days, setDays] = useState(7)

  // 获取应用详情（包含inner_feature和oauth_client_data）
  const {
    data: appQueryData,
    isLoading: isLoadingApp,
    isError: isAppError,
    error: appError,
    refetch: refetchApp
  } = useQuery({
    queryKey: appQueryKey(appId, {
      attr_inner_feature: true,
      attr_oauth_client_data: true,
      attr_parent_app: true,
      attr_sub_app_count: true,
    }),
    queryFn: ({ signal }) =>
      appList(
        {
          app_id: Number(appId),
          attr_inner_feature: true,
          attr_oauth_client_data: true,
          attr_parent_app: true,
          attr_sub_app_count: true,
          page: { page: 1, limit: 1 },
          count_num: false
        },
        { signal }
      ),
    enabled: !!appId
  })

  // 加载应用字典
  const {
    dictData,
    isLoading: dictIsLoading,
    isError: isDictError,
    errors: dictErrors,
    refetch: refetchDict
  } = useDictData(['user_app'] as const)

  // 统一重新加载函数
  const refetch = () => {
    refetchApp()
    refetchDict()
  }

  // 检查加载状态
  const isLoading = isLoadingApp || dictIsLoading



  // 获取应用详情数据
  const appDetail = appQueryData?.response?.data?.[0]


  // 计算扩展功能状态
  const { enabledFeatures, disabledFeatures } = useMemo(() => {
    if (!appDetail || !dictData) return { enabledFeatures: [], disabledFeatures: [] }

    const allFeatures = dictData?.exter_features || []
    const appFeatures = appDetail.exter_feature || [] // exter_feature is array of strings, not objects

    const enabled = allFeatures.filter((f: any) => appFeatures.includes(f.key))
    const disabled = allFeatures.filter((f: any) => !appFeatures.includes(f.key))

    return { enabledFeatures: enabled, disabledFeatures: disabled }
  }, [appDetail, dictData])

  // OAuth 客户端状态
  const oauthClientData = appDetail?.oauth_client_data
  const scopeArray = oauthClientData?.scope_data
    ? oauthClientData.scope_data.split(',').filter((s) => s.trim())
    : []
  const hasOAuthClient = scopeArray.length > 0

  // 是否为父应用
  const hasParentApp = !!appDetail?.parent_app

  // 是否开通子应用功能 (check sup_app field)
  const hasSubAppFeature = !!appDetail?.sup_app

  // 是否开通外部登录功能 (exter_login comes as string '1' or '0' from API)
  const hasExterLoginFeature = appDetail?.exter_login || appDetail?.exter_login === true

  // 时间范围选项
  const dayOptions = [
    { label: '1 天', value: 1 },
    { label: '7 天', value: 7 },
    { label: '1 个月', value: 30 },
    { label: '3 个月', value: 90 }
  ]

  // 导航容器配置（添加 actions）
  const navConfig = {
    ...indexModuleConfig,
    actions: (
      <div className={cn("flex items-center gap-2 w-full sm:w-auto")}>
        <Select value={String(days)} onValueChange={(value) => setDays(Number(value))}>
          <SelectTrigger className={cn("w-full sm:w-[180px]")}>
            <span className={cn("sm:hidden text-sm text-muted-foreground")}>统计范围:</span>
            <SelectValue placeholder="选择时间范围" />
          </SelectTrigger>
          <SelectContent className="max-h-[300px]">
            {dayOptions.map((option) => (
              <SelectItem key={option.value} value={String(option.value)}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
    )
  }

  // 显示加载状态
  if (isLoading) {
    return (
      <AppDetailNavContainer {...navConfig}>
        <CenteredLoading variant="card" />
      </AppDetailNavContainer>
    )
  }

  // 显示错误状态
  if (isAppError || isDictError || !appDetail) {
    let error = appError || dictErrors
    if (!appDetail) {
      error = new Error('应用不存在或已被删除')
    }
    return (
      <AppDetailNavContainer {...navConfig}>
        <CenteredError variant="content" error={error} onReset={refetch} />
      </AppDetailNavContainer>
    )
  }

  return (
    <AppDetailNavContainer {...navConfig}>
      <DetailContent
        appId={String(appId)}
        appDetail={appDetail}
        dictData={dictData}
        days={days}
        enabledFeatures={enabledFeatures}
        disabledFeatures={disabledFeatures}
        hasOAuthClient={hasOAuthClient}
        scopeArray={scopeArray}
        hasParentApp={hasParentApp}
        hasSubAppFeature={hasSubAppFeature ?? false}
        hasExterLoginFeature={hasExterLoginFeature ?? false}
      />
    </AppDetailNavContainer>
  )
}

