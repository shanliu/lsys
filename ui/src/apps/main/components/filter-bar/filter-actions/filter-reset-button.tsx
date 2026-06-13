import { ExtendedFormReturn, FilterFieldContext, LayoutParams } from '../context'
import { Button } from '@shared/components/ui/button'
import { cn } from '@shared/lib/utils'
import { RotateCcw } from 'lucide-react'
import React from 'react'
import { FieldValues } from 'react-hook-form'

export interface FilterResetButtonProps<TFieldValues extends FieldValues = FieldValues> {
  form?: ExtendedFormReturn<TFieldValues>
  loading?: boolean
  className?: string
  layoutParams?: LayoutParams
}

export function FilterResetButton<TFieldValues extends FieldValues = FieldValues>({
  form,
  loading = false,
  className,
  layoutParams,
}: FilterResetButtonProps<TFieldValues>) {
  const filterCtx = React.useContext(FilterFieldContext)
  const effectiveForm = form ?? filterCtx?.form
  const effectiveLayoutParams = layoutParams ?? filterCtx?.layoutParams ?? { isMobile: false }
  const isMobile = effectiveLayoutParams.isMobile

  // 监听表单值变化以触发重新渲染
  effectiveForm?.watch?.()

  const hasFormValues = React.useMemo(() => {
    if (!effectiveForm) return false

    const values = effectiveForm.getValues()
    return Object.values(values).some(value => {
      if (value === null || value === undefined || value === '') return false
      if (typeof value === 'string' && value.trim() === '') return false
      if (Array.isArray(value) && value.length === 0) return false
      return true
    })
  }, [effectiveForm])

  const handleClear = React.useCallback(() => {
    if (effectiveForm) {
      effectiveForm.handleFormReset()
    }
  }, [effectiveForm])

  if (!hasFormValues) return null

  if (isMobile) {
    return (
      <Button
        onClick={handleClear}
        disabled={loading}
        variant="outline"
        className={cn("w-full", className)}
      >
        <RotateCcw className="h-4 w-4" />
        重置
      </Button>
    )
  }

  return (
    <Button
      onClick={handleClear}
      disabled={loading}
      variant="outline"
      size="lg"
      className={cn("w-20", className)}
    >
      <RotateCcw className="h-4 w-4" />
      重置
    </Button>
  )
}
