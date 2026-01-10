import { cn } from '@shared/lib/utils'
import { useIsMobile } from '@shared/hooks/use-mobile'
import { Eye, EyeClosed } from 'lucide-react'
import * as React from 'react'
import { Button } from '../../ui/button'
import { Input } from '../../ui/input'

type PasswordInputProps = Omit<
  React.InputHTMLAttributes<HTMLInputElement>,
  'type'
>

const PasswordInput = React.forwardRef<HTMLInputElement, PasswordInputProps>(
  ({ className, disabled, onChange, value, ...props }, ref) => {
    const isMobile = useIsMobile()
    const [showPassword, setShowPassword] = React.useState(false)
    const inputRef = React.useRef<HTMLInputElement>(null)

    // 桌面端专用状态 - 字符叠加层功能
    const [lastChar, setLastChar] = React.useState('')
    const [showLastChar, setShowLastChar] = React.useState(false)
    const [cursorPosition, setCursorPosition] = React.useState(0)
    const [prevLength, setPrevLength] = React.useState(0)
    const timeoutRef = React.useRef<NodeJS.Timeout | null>(null)

    // 合并refs
    React.useImperativeHandle(ref, () => inputRef.current!, [])

    // 桌面端专用 - 清理定时器
    const clearCharTimeout = React.useCallback(() => {
      if (isMobile) return
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current)
        timeoutRef.current = null
      }
    }, [isMobile])

    const handleChange = React.useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
      const newValue = e.target.value
      const currentLength = newValue.length
      // 获取实际光标位置
      const actualCursorPos = e.target.selectionStart ?? currentLength

      // 调用外部onChange
      onChange?.(e)

      // 移动端不需要字符叠加层功能
      if (isMobile) return

      if (showPassword) {
        // 显示密码模式，不需要特殊处理
        setPrevLength(currentLength)
        return
      }

      // 检测是否有新增字符
      if (currentLength > prevLength && currentLength > 0) {
        // 清理之前的定时器（只在有新字符时清理）
        clearCharTimeout()

        // 有新字符输入 - 使用光标位置前一个字符（刚输入的字符）
        const newCharIndex = actualCursorPos - 1
        const newChar = newValue[newCharIndex] ?? ''
        setLastChar(newChar)
        setShowLastChar(true)
        setCursorPosition(actualCursorPos)

        // 0.5秒后隐藏字符
        timeoutRef.current = setTimeout(() => {
          setShowLastChar(false)
          setLastChar('')
        }, 500)
      } else if (currentLength < prevLength) {
        // 只有在删除字符时才清理显示状态
        clearCharTimeout()
        setShowLastChar(false)
        setLastChar('')
      }
      // 如果长度相同，不做任何处理，让字符继续显示

      setPrevLength(currentLength)
    }, [isMobile, showPassword, onChange, clearCharTimeout, prevLength])

    const togglePasswordVisibility = React.useCallback(() => {
      setShowPassword(prev => {
        if (!prev && !isMobile) {
          // 切换到显示模式时，清理字符显示（仅桌面端）
          clearCharTimeout()
          setShowLastChar(false)
          setLastChar('')
        }
        return !prev
      })
    }, [isMobile, clearCharTimeout])

    // 桌面端 - 组件卸载时清理定时器
    React.useEffect(() => {
      if (isMobile) return
      return clearCharTimeout
    }, [isMobile, clearCharTimeout])

    // 桌面端 - 初始化和同步外部value
    React.useEffect(() => {
      if (isMobile) return
      if (value !== undefined) {
        const valueLength = String(value).length
        const currentPrevLength = prevLength

        // 只有在value长度发生实际变化时才更新prevLength
        if (valueLength !== currentPrevLength) {
          setPrevLength(valueLength)

          // 只在value变短时清理显示状态（用户可能通过外部方式清空了输入）
          if (valueLength < currentPrevLength) {
            setShowLastChar(false)
            setLastChar('')
            clearCharTimeout()
          }
        }
      }
    }, [isMobile, value, prevLength, clearCharTimeout])

    // 桌面端专用 - 计算字符显示位置
    const getCharPosition = React.useCallback(() => {
      if (isMobile || !inputRef.current || !showLastChar || cursorPosition === 0) {
        return { left: 0, opacity: 0, charWidth: 14 }
      }

      const input = inputRef.current
      const style = window.getComputedStyle(input)

      // 创建临时测量元素来准确计算宽度
      const canvas = document.createElement('canvas')
      const context = canvas.getContext('2d')

      if (context) {
        // 设置与输入框相同的字体
        context.font = `${style.fontSize} ${style.fontFamily}`

        // 测量密码字符的实际宽度
        const bulletChar = '•' // 或者用 '●'
        const charWidth = context.measureText(bulletChar).width

        const paddingLeft = parseFloat(style.paddingLeft) || 12

        // 计算前面所有字符的总宽度
        const charIndex = cursorPosition - 1
        const left = paddingLeft + (charIndex * charWidth)

        return {
          left: Math.max(left, paddingLeft),
          opacity: 1,
          charWidth // 返回字符宽度用于设置叠加层宽度
        }
      }

      return { left: 0, opacity: 0, charWidth: 14 }
    }, [isMobile, showLastChar, cursorPosition])

    // 仅在桌面端计算字符位置
    const charPosition = !isMobile ? getCharPosition() : null

    return (
      <div className={`relative ${className || ''}`}>
        <Input
          type={showPassword ? 'text' : 'password'}
          className={cn("pr-10")}
          ref={inputRef}
          disabled={disabled}
          value={value}
          onChange={handleChange}
          {...props}
        />

        {/* 桌面端专用 - 显示当前输入字符的叠加层 */}
        {!isMobile && !showPassword && showLastChar && lastChar && charPosition && (
          <div
            className="absolute top-1/2 -translate-y-1/2 pointer-events-none select-none text-foreground flex items-center justify-center bg-background dark:bg-background rounded-sm z-10 h-5 text-sm"
            style={{
              left: `${charPosition.left}px`,
              opacity: charPosition.opacity,
              width: `${charPosition.charWidth}px`
            }}
          >
            {lastChar}
          </div>
        )}

        <Button
          type='button'
          size='icon'
          variant='ghost'
          disabled={disabled}
          className={cn('text-muted-foreground absolute top-1/2 right-1 h-6 w-6 -translate-y-1/2 rounded-md')}
          onClick={togglePasswordVisibility}
        >
          {showPassword ? <Eye size={18} /> : <EyeClosed size={18} />}
        </Button>
      </div>
    )
  }
)

export { PasswordInput }
