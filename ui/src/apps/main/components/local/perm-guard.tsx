import { useToast } from '@shared/contexts/toast-context'
import { PermissionKey, usePerm } from '@apps/main/hooks/use-perm'
import { cn } from '@shared/lib/utils'
import { ReactNode } from 'react'

type PermGuardPermission = string | PermissionKey | Array<string | PermissionKey>

type PermGuardFallback =
  | ReactNode
  | ((error: string | null) => ReactNode)
  | 'disabled'  // 显示禁用状态
  | 'hidden'    // 隐藏元素（默认）

interface PermGuardProps {
  permission: PermGuardPermission
  children: ReactNode
  fallback?: PermGuardFallback
}

function normalizePermissions(
  permission: PermGuardPermission
): PermissionKey[] {
  if (Array.isArray(permission)) {
    return permission.map((p) =>
      typeof p === 'string' ? { name: p } : p
    )
  }
  if (typeof permission === 'string') {
    return [{ name: permission }]
  }
  return [permission]
}

/**
 * 权限守卫组件
 * 支持字符串、字符串数组、PermissionKey、PermissionKey[]
 * fallback 支持:
 * - ReactNode: 自定义未授权显示内容
 * - Function: 动态生成未授权内容
 * - 'disabled': 显示禁用状态（点击时显示权限错误信息）
 * - 'hidden' 或 null: 隐藏元素（默认）
 */
export function PermGuard({ permission, children, fallback = 'hidden' }: PermGuardProps) {
  const permList = normalizePermissions(permission)
  const { isLoading, isError, error, result } = usePerm(permList)
  const toast = useToast()

  // 渲染禁用状态的公共函数
  const renderDisabled = (errorMessage?: string) => {
    const handleClick = (e: React.MouseEvent) => {
      e.preventDefault()
      e.stopPropagation()

      // 显示权限错误信息
      if (errorMessage) {
        toast.error(errorMessage)
      } else {
        // 从权限检查结果中获取错误信息
        // status: 0=失败不显示, 1=成功, 2=失败提示错误
        const deniedPermissions = result.filter(r => r.status !== 1)
        const reasons = deniedPermissions.map(p => p.reason).filter(Boolean)
        const message = reasons.length > 0
          ? reasons.join('; ')
          : '您没有执行此操作的权限'
        toast.error(message)
      }
    }

    return (
      <div
        className={cn("relative")}
        style={{ position: 'relative' }}
      >
        {children}
        <div
          className={cn("absolute inset-0 cursor-not-allowed")}
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            cursor: 'not-allowed',
            pointerEvents: 'all',
            zIndex: 10
          }}
          onClick={handleClick}
          onPointerDown={(e) => e.preventDefault()}
          onMouseDown={(e) => e.preventDefault()}
          onTouchStart={(e) => e.preventDefault()}
        />
      </div>
    )
  }

  if (isLoading) return null

  if (isError) {
    if (typeof fallback === 'function') {
      return <>{fallback(error)}</>
    }
    if (fallback === 'disabled') {
      return renderDisabled(error || '权限检查失败，请重试')
    }
    if (fallback === 'hidden') return null
    return <>{fallback}</>
  }

  // 只要有一个权限通过就允许 (status === 1)
  const allowed = result.some((r) => r.status === 1)
  if (allowed) return <>{children}</>

  // 未授权时的处理
  if (fallback === 'disabled') {
    return renderDisabled()
  }

  if (typeof fallback === 'function') {
    return <>{fallback(null)}</>
  }

  if (fallback === 'hidden') return null

  return <>{fallback}</>
}
