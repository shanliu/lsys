import type { AppListResType } from '@shared/apis/user/app'
import CopyableText from '@shared/components/custom/text/copyable-text'
import { Card, CardContent, CardHeader, CardTitle } from '@shared/components/ui/card'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@shared/components/ui/tooltip'
import { useAuthData } from '@apps/main/hooks/use-auth-data'
import { TypedDictData } from '@apps/main/hooks/use-dict-data'
import { cn, formatTime, TIME_STYLE } from '@shared/lib/utils'
import { Link } from '@tanstack/react-router'
import { LayoutDashboard, PackageX, TreePine, Trees } from 'lucide-react'
import React from 'react'

export interface AppCardProps {
  app: AppListResType['data'][0]
  dictData: TypedDictData<["user_app"]>
}

// 单个应用卡片组件
export const AppCard: React.FC<AppCardProps> = ({ app, dictData }) => {
  const authData = useAuthData()
  // 如果用户 appData 不为空，则不显示 parent_app 相关内容
  const hideParentApp = !!authData?.appData
  const hasParent = !hideParentApp && !!app.parent_app

  // --- 状态逻辑（仅使用服务端字典 key，不做文本匹配） ---
  // 当前已知映射：1=审核中(待审核), 2=正常(启用), 3=被禁用(disable)
  // 还未拿到"已拒绝"对应的 key，先占位（未来提供后补充）
  const statusCode = Number(app.status)
  const statusItem = dictData.app_status.find((item: any) => String(item.key) === String(statusCode))
  const statusText = statusItem ? statusItem.val : `状态${statusCode}`

  const isPendingReview = statusCode === 1
  // 用户确认：key=3 (被禁用) 语义上即"已拒绝"
  const isRejected = statusCode === 3

  // 外部功能（exter_feature）不为空则展示数量或标签
  const hasExterFeature = Array.isArray(app.exter_feature) && app.exter_feature.length > 0

  // 非子应用 (无 parent_app) 时显示子应用申请待处理数量角标（假设字段 sub_req_pending_count 存在于 user_data 或 app 对象上）
  const pendingSubReqCount: number | undefined = (app as any).sub_req_pending_count ?? (app as any).user_data?.sub_req_pending_count
  const showPendingBadge = !hasParent && typeof pendingSubReqCount === 'number' && pendingSubReqCount > 0

  return (
    <Card className={cn('hover:shadow-md transition-shadow relative flex flex-col')}>
      {showPendingBadge && (
        <div className='absolute -top-2 -right-2 bg-destructive text-destructive-foreground text-[10px] font-semibold rounded-full min-w-5 h-5 px-1 flex items-center justify-center shadow'>
          {pendingSubReqCount}
        </div>
      )}

      {/* Header Section */}
      <CardHeader className={cn('pb-3')}>
        <Link
          to='/user/app/$appId'
          params={{ appId: Number(app.id) }}
          aria-label={`管理应用 ${app.name}`}
          title={`管理应用 ${app.name}`}
          className='flex items-start gap-3 min-w-0 group focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring rounded-md'
        >
          <div
            className={cn(
              'flex size-10 shrink-0 items-center justify-center rounded-lg p-2 ring-1 ring-border/60 transition-colors group-hover:bg-primary/15',
              'bg-primary/10 text-primary'
            )}
          >
            {/* 图标规则：
          1. 已拒绝: PackageX (破盒子) 红色
          2. 待审核: 显示应用名称前2个字
          3. 已审核(启用): 子应用=Trees, 普通应用=TreePine */}
            {isRejected ? (
              <PackageX className='h-4 w-4 text-destructive' />
            ) : isPendingReview ? (
              <span className='text-xs font-bold opacity-70'>{app.name.substring(0, 2).toUpperCase()}</span>
            ) : hasParent ? (
              <Trees className='h-4 w-4' />
            ) : (
              <TreePine className='h-4 w-4' />
            )}
          </div>
          <div className='space-y-1 min-w-0'>
            <CardTitle className='text-sm font-semibold truncate group-hover:text-foreground' title={app.name}>
              {app.name}
            </CardTitle>
            <div className='flex flex-wrap items-center gap-x-2 gap-y-0.5 text-[11px] text-muted-foreground'>
              <span>{hasParent ? '子应用' : '普通应用'}</span>
              <span className='inline-block w-px h-3 bg-border' />
              <span className={`${isPendingReview ? 'text-amber-600' : ''} ${isRejected ? 'text-destructive' : ''}`}>{statusText}</span>
            </div>
          </div>
        </Link>
      </CardHeader>

      <CardContent className={cn('pt-0 pb-3 flex-1 flex flex-col')}>
        <div className='space-y-4 text-xs flex-1'>
          {/* Basic Info */}
          <div className='grid grid-cols-1 gap-2'>
            <div className='flex items-center justify-between gap-2'>
              <span className='text-muted-foreground whitespace-nowrap shrink-0'>Client ID</span>
              <CopyableText
                value={app.client_id}
                message='Client ID 已复制'
                title='点击复制 Client ID'
                className='font-mono'
                maxWidthClassName='max-w-[140px]'
                showIcon
              />
            </div>
            {hasParent && app.parent_app && (
              <div className='flex items-center justify-between gap-2'>
                <span className='text-muted-foreground'>父应用</span>
                <Link
                  to='.'
                  search={{ page: 1, parent_app_id: app.parent_app.id } as any}
                  className='truncate max-w-[140px] text-blue-600 hover:underline'
                  title={`查看 ${app.parent_app.name} 的子应用`}
                >
                  {app.parent_app.name}
                </Link>
              </div>
            )}
            {!hasParent && (
              <div className='flex items-center justify-between gap-2'>
                <span className='text-muted-foreground'>子应用数量</span>
                <span className='flex items-center gap-1'>
                  {!app.sup_app ? (
                    <span className='text-muted-foreground/70'>未开通</span>
                  ) : app.sub_app_count ? (
                    <>
                      <span>{app.sub_app_count.enable + app.sub_app_count.disable + app.sub_app_count.init}</span>
                      {app.sub_app_count.init > 0 && (
                        <span className='text-[10px] text-amber-600 bg-amber-50 border border-amber-200 rounded px-1 py-0.5'>待审核 {app.sub_app_count.init}</span>
                      )}
                    </>
                  ) : (
                    <span>0</span>
                  )}
                </span>
              </div>
            )}
          </div>

          {/* Capability Pills */}
          <div className='space-y-2'>
            <div className='text-[11px] font-medium text-muted-foreground'>功能状态</div>
            <div className='flex flex-wrap gap-1'>
              <FeaturePill label='OAuth登陆' active={!!app.oauth_client} />
              <FeaturePill label='登陆本系统' active={!!app.exter_login} />
              <FeaturePill label='OAuth服务' active={!!app.oauth_server} />
              <FeaturePill label='子应用功能' active={!!app.sup_app} />
              {hasExterFeature && (
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <span>
                        <FeaturePill label={`扩展功能×${app.exter_feature.length}`} active />
                      </span>
                    </TooltipTrigger>
                    <TooltipContent side='right' sideOffset={6} className='max-w-[260px] whitespace-pre-line break-words bg-popover text-popover-foreground border border-border shadow-md px-3 py-2 rounded-md leading-relaxed text-[11px]'>
                      {(() => {
                        try {
                          // 假设 exter_feature 里每项可能是对象或字符串
                          return app.exter_feature
                            .map((f: any, idx: number) => {
                              if (f == null) return null
                              if (typeof f === 'string') return f
                              if (typeof f === 'object') {
                                return f.name || f.title || f.key || `#${idx + 1}`
                              }
                              return String(f)
                            })
                            .filter(Boolean)
                            .join('\n') || '无数据'
                        } catch {
                          return '无数据'
                        }
                      })()}
                    </TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              )}
            </div>
          </div>
        </div>

        {/* Meta / Time & Management Link */}
        <div className='mt-4 pt-2.5 flex items-center justify-between gap-2 border-t border-border/40'>
          <div className='flex items-center gap-2 text-[11px]'>
            <span className='text-muted-foreground'>更新时间</span>
            <span className='text-foreground'>{formatTime(app.change_time, TIME_STYLE.RELATIVE_ELEMENT)}</span>
          </div>
          <Link
            to='/user/app/$appId'
            params={{ appId: Number(app.id) }}
            aria-label='管理应用'
            title='管理应用'
            className='inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md border border-input bg-background text-xs text-muted-foreground hover:text-foreground hover:bg-accent transition-colors whitespace-nowrap'
          >
            <LayoutDashboard className='h-3.5 w-3.5' />
            <span>管理</span>
          </Link>
        </div>
      </CardContent>
    </Card>
  )
}
export default AppCard

// --- Internal small component ---
const FeaturePill: React.FC<{ label: string; active: boolean }> = ({ label, active }) => (
  <span
    className={cn(
      'px-2 py-0.5 rounded-md border text-[10px] font-medium tracking-wide select-none transition-colors',
      active
        ? 'bg-emerald-50 text-emerald-700 border-emerald-200'
        : 'bg-muted/40 text-muted-foreground/60 border-dashed border-border/40 line-through'
    )}
  >
    {label}
  </span>
)
