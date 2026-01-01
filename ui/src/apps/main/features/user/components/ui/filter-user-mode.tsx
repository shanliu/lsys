import type { LayoutParams } from '@apps/main/components/filter-container/container'
import { ClearableInput } from '@shared/components/custom/input/clearable-input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@shared/components/ui/select'
import { cn } from '@shared/lib/utils'

/**
 * RBAC 用户模式上下文类型
 * - use_app_user: true 表示"应用本身"模式，false 表示"应用内用户"模式
 * - user_param: 当 use_app_user 为 false 时，需要输入的用户参数
 */
export interface RbacUserModeContext {
  use_app_user: boolean
  user_param: string
}

interface FilterUserModeProps {
  /** 当前值 */
  value: RbacUserModeContext
  /** 值变化回调 */
  onChange: (value: RbacUserModeContext) => void
  /** 是否禁用 */
  disabled?: boolean
  /** 布局参数（来自 FilterContainer） */
  layoutParams?: LayoutParams
  /** 自定义类名 */
  className?: string
}

/**
 * RBAC 用户模式选择组件
 * 
 * 用于在 FilterContainer 中选择 RBAC 操作的用户模式：
 * - 应用本身：使用应用自身的权限配置
 * - 应用内用户：指定应用内的用户参数
 * 
 * 支持移动端和桌面端响应式布局，与 FilterInput/FilterSelect 保持一致的样式
 */
export function FilterUserMode({
  value,
  onChange,
  disabled = false,
  layoutParams,
  className,
}: FilterUserModeProps) {
  const handleModeChange = (mode: string) => {
    const useAppUser = mode === 'app'
    onChange({
      use_app_user: useAppUser,
      user_param: useAppUser ? '' : value.user_param,
    })
  }

  const handleUserParamChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange({
      ...value,
      user_param: e.target.value,
    })
  }

  const handleUserParamClear = () => {
    onChange({
      ...value,
      user_param: '',
    })
  }

  // 移动端布局 - 与 FilterSelect/FilterInput 保持一致
  if (layoutParams?.isMobile) {
    return (
      <div className={cn('flex flex-col gap-3 w-full', className)}>
        {/* 用户模式选择 - 移动端水平布局 */}
        <div className="flex items-center gap-3 w-full min-w-0">
          <div className="text-xs font-medium text-muted-foreground leading-none flex-shrink-0 w-16">
            用户模式
          </div>
          <div className="relative flex-1 min-w-0">
            <Select
              value={value.use_app_user ? 'app' : 'user'}
              onValueChange={handleModeChange}
              disabled={disabled}
            >
              <SelectTrigger className="h-9">
                <SelectValue placeholder="选择模式" />
              </SelectTrigger>
              <SelectContent className="max-h-[300px]">
                <SelectItem value="app">应用本身</SelectItem>
                <SelectItem value="user">应用内用户</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        {/* 用户参数输入框（仅在"应用内用户"模式下显示） - 移动端水平布局 */}
        {!value.use_app_user && (
          <div className="flex items-center gap-3 w-full min-w-0">
            <div className="text-xs font-medium text-muted-foreground leading-none flex-shrink-0 w-16">
              用户参数
            </div>
            <div className="relative flex-1 min-w-0">
              <ClearableInput
                type="text"
                placeholder="输入用户参数"
                value={value.user_param}
                onChange={handleUserParamChange}
                onClear={handleUserParamClear}
                enableDoubleClickPaste={true}
                showClearButton={true}
                disabled={disabled}
                className="h-9 text-sm"
              />
            </div>
          </div>
        )}
      </div>
    )
  }

  // 桌面端布局 - 与 FilterInput/FilterSelect 保持一致的垂直布局
  return (
    <div className={cn('flex items-end gap-4', className)}>
      {/* 用户模式选择 - 桌面端垂直布局 */}
      <div className="flex flex-col">
        <div className="text-sm font-medium text-muted-foreground mb-3">
          用户模式
        </div>
        <Select
          value={value.use_app_user ? 'app' : 'user'}
          onValueChange={handleModeChange}
          disabled={disabled}
        >
          <SelectTrigger className="w-32 h-10">
            <SelectValue placeholder="选择模式" />
          </SelectTrigger>
          <SelectContent className="max-h-[300px]">
            <SelectItem value="app">应用本身</SelectItem>
            <SelectItem value="user">应用内用户</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* 用户参数输入框（仅在"应用内用户"模式下显示） - 桌面端垂直布局 */}
      {!value.use_app_user && (
        <div className="flex flex-col">
          <div className="text-sm font-medium text-muted-foreground mb-3">
            用户参数
          </div>
          <ClearableInput
            type="text"
            placeholder="输入用户参数"
            value={value.user_param}
            onChange={handleUserParamChange}
            onClear={handleUserParamClear}
            enableDoubleClickPaste={true}
            showClearButton={true}
            disabled={disabled}
            className="w-40 h-10 text-base"
          />
        </div>
      )}
    </div>
  )
}
