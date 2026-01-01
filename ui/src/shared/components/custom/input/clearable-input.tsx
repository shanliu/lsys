import { Button } from "@shared/components/ui/button"
import { Input } from "@shared/components/ui/input"
import { cn } from "@shared/lib/utils"
import { X } from "lucide-react"
import * as React from "react"

interface ClearableInputProps extends React.ComponentProps<"input"> {
  onClear?: () => void
  showClearButton?: boolean
  clearButtonClassName?: string
  enableDoubleClickPaste?: boolean
}

const ClearableInput = React.forwardRef<HTMLInputElement, ClearableInputProps>(
  ({
    className,
    onClear,
    showClearButton = true,
    clearButtonClassName,
    enableDoubleClickPaste = false,
    value,
    onChange,
    ...props
  }, ref) => {
    // 统一使用内部状态管理输入值
    const [inputValue, setInputValue] = React.useState(value ?? "")

    // 同步外部 value 变化到内部状态
    React.useEffect(() => {
      setInputValue(value ?? "")
    }, [value])

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      setInputValue(e.target.value)
      onChange?.(e)
    }

    const handleClear = () => {
      setInputValue("")
      // 创建一个模拟的change事件来触发onChange
      if (onChange) {
        const syntheticEvent = {
          target: { value: "" },
          currentTarget: { value: "" }
        } as React.ChangeEvent<HTMLInputElement>
        onChange(syntheticEvent)
      }
      onClear?.()
    }

    const handleDoubleClick = async () => {
      // 只有在启用双击粘贴功能且输入框为空时才执行
      if (!enableDoubleClickPaste || (inputValue && String(inputValue).length > 0)) {
        return
      }

      try {
        // 检查浏览器是否支持 Clipboard API
        if (navigator.clipboard && navigator.clipboard.readText) {
          const clipboardText = await navigator.clipboard.readText()
          if (clipboardText) {
            setInputValue(clipboardText)

            // 创建一个模拟的change事件来触发onChange
            if (onChange) {
              const syntheticEvent = {
                target: { value: clipboardText },
                currentTarget: { value: clipboardText }
              } as React.ChangeEvent<HTMLInputElement>
              onChange(syntheticEvent)
            }
          }
        }
      } catch (error) {
        // 静默处理错误，可能是权限问题或浏览器不支持
        console.warn('Failed to read clipboard:', error)
      }
    }

    const shouldShowClearButton = showClearButton && inputValue && String(inputValue).length > 0

    return (
      <div className="relative">
        <Input
          ref={ref}
          className={cn(
            "pr-8", // 始终预留清除按钮空间，避免出现/消失时宽度抖动
            className
          )}
          value={inputValue}
          onChange={handleChange}
          onDoubleClick={enableDoubleClickPaste ? handleDoubleClick : undefined}
          {...props}
        />
        {shouldShowClearButton && (
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className={cn(
              "absolute right-0 top-0 h-full px-2 py-0 hover:bg-transparent",
              clearButtonClassName
            )}
            onClick={handleClear}
            tabIndex={-1}
          >
            <X className={cn("h-4 w-4 text-muted-foreground hover:text-foreground")} />
          </Button>
        )}
      </div>
    )
  }
)

ClearableInput.displayName = "ClearableInput"

export { ClearableInput }
export type { ClearableInputProps }

