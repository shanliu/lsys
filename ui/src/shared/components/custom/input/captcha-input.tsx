import { getCaptchaImage, validCaptchaData } from '@shared/apis/public/captcha'
import { cn, generateRandomString } from '@shared/lib/utils'
import type { ApiResult } from '@shared/types/apis-rest'
import { useMutation, useQuery } from '@tanstack/react-query'
import { AlertCircle, CheckCircle, Loader2, XCircle } from 'lucide-react'
import * as React from 'react'
import { Input } from '../../ui/input'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '../../ui/tooltip'

export interface CaptchaData {
  code: string
  key: string
  validation?: 'success' | 'error' | 'pending' | null // 验证状态
}

interface CaptchaInputProps {
  /** 验证码类型 */
  captchaType: string
  value?: CaptchaData
  /** 验证码数据变化回调 */
  onChange?: (captcha: CaptchaData) => void
  /** 自定义类名 */
  className?: string
  /** 输入框占位符 */
  placeholder?: string
}

const CaptchaInput = React.forwardRef<HTMLInputElement, CaptchaInputProps>(
  ({
    captchaType,
    value,
    onChange,
    className,
    placeholder = "请输入验证码",
    ...props
  }, ref) => {
    const [captchaTag, setCaptchaTag] = React.useState<string>(() => generateRandomString(8))
    const [hasImageLoadError, setHasImageLoadError] = React.useState(false)
    // 修改为存储验证结果的数组，包含验证码和结果
    const [validationResults, setValidationResults] = React.useState<Array<{ code: string; success: boolean }>>([])

    // 使用 useDeferredValue 防止输入抖动
    const deferredInputCode = React.useDeferredValue(value?.code || '')

    // 使用 ref 来存储最新的 onChange 回调，避免依赖项变化导致无限循环
    const onChangeRef = React.useRef(onChange)
    onChangeRef.current = onChange

    // 使用 tanstack-query 获取验证码，但不缓存
    const { data: captchaResult, isLoading, error: queryError } = useQuery({
      queryKey: ['captcha', captchaType, captchaTag],
      queryFn: async ({ signal }) => {
        if (!captchaType || !captchaTag) {
          throw new Error('captchaType and captchaTag are required')
        }
        return await getCaptchaImage({
          captcha_type: captchaType,
          captcha_tag: captchaTag
        }, { signal })
      },
      enabled: !!captchaType && !!captchaTag, // 移除失败计数限制，允许重试
      retry: false, // 不自动重试
      staleTime: 0, // 立即过期
      gcTime: 0, // 不缓存 (新版本使用 gcTime 而不是 cacheTime)
      refetchOnMount: false,
      refetchOnWindowFocus: false,
      refetchOnReconnect: false
    })

    // 监听查询错误或成功，重置图片加载错误状态
    React.useEffect(() => {
      if (queryError || captchaResult?.response) {
        setHasImageLoadError(false)
      }
    }, [queryError, captchaResult])

    // 移除自动更新验证码数据的useEffect，避免循环依赖
    // 验证码数据将只在用户输入时更新

    // 计算图片URL
    const imageUrl = React.useMemo(() => {
      const result = captchaResult as ApiResult<{ image_data: string; image_header: string; save_time: number; code_length: number }>
      if (!result?.response) return ''

      const imageData = result.response.image_data || ''
      const imageHeader = result.response.image_header || 'image/png'

      if (imageData.startsWith('data:')) {
        return imageData
      } else {
        return `data:${imageHeader};base64,${imageData}`
      }
    }, [captchaResult])

    // 获取验证码长度
    const codeLength = React.useMemo(() => {
      const result = captchaResult as ApiResult<{ image_data: string; image_header: string; save_time: number; code_length: number }>
      return result?.response?.code_length || 4 // 默认长度为4
    }, [captchaResult])

    // 刷新验证码
    const refreshCaptcha = React.useCallback(() => {
      const newTag = generateRandomString(8)
      setCaptchaTag(newTag)
      setValidationResults([]) // 重置验证结果数组
      setHasImageLoadError(false) // 重置图片加载错误状态
    }, [])

    // 使用 useMutation 处理验证码验证
    const validationMutation = useMutation({
      mutationFn: async ({ code, signal }: { code: string; signal?: AbortSignal }) => {
        if (!captchaType || !captchaTag || !code) {
          throw new Error('Missing required parameters')
        }

        return await validCaptchaData({
          captcha_type: captchaType,
          captcha_tag: captchaTag,
          captcha_code: code
        }, { signal })
      },
      onSuccess: (result, { code }) => {
        // 将验证结果添加到数组中
        setValidationResults(prev => {
          const newResults = [...prev, { code, success: result.status }]

          if (result.status) {
            // 验证成功
            // 更新验证码数据，包含验证状态
            if (onChangeRef.current) {
              const newCaptchaData: CaptchaData = {
                code,
                key: captchaTag,
                validation: 'success'
              }
              onChangeRef.current(newCaptchaData)
            }
          } else {
            // 处理验证失败
            // 更新验证码数据，包含验证状态
            if (onChangeRef.current) {
              const newCaptchaData: CaptchaData = {
                code,
                key: captchaTag,
                validation: 'error'
              }
              onChangeRef.current(newCaptchaData)
            }

            // 检查是否需要自动刷新验证码
            // 情况1: 失败记录超过3次
            const failureCount = newResults.filter(r => !r.success).length
            // 情况2: 总记录超过4次且最后一次是失败
            const totalCount = newResults.length
            const lastResult = newResults[newResults.length - 1]
            const isLastFailure = lastResult && !lastResult.success

            const shouldRefresh = failureCount > 3 || (totalCount > 4 && isLastFailure)

            if (shouldRefresh) {
              setTimeout(() => {
                // 直接调用生成新标签的函数，避免依赖循环
                const newTag = generateRandomString(8)
                setCaptchaTag(newTag)
                setValidationResults([]) // 重置验证结果数组
                setHasImageLoadError(false) // 重置图片加载错误状态
              }, 100)
            }
          }

          return newResults
        })
      },
      onError: (error, { code }) => {
        // 将验证失败结果添加到数组中
        setValidationResults(prev => {
          const newResults = [...prev, { code, success: false }]

          // 更新验证码数据，包含验证状态
          if (onChangeRef.current) {
            const newCaptchaData: CaptchaData = {
              code,
              key: captchaTag,
              validation: 'error'
            }
            onChangeRef.current(newCaptchaData)
          }

          // 检查是否需要自动刷新验证码
          // 情况1: 失败记录超过3次
          const failureCount = newResults.filter(r => !r.success).length
          // 情况2: 总记录超过4次且最后一次是失败
          const totalCount = newResults.length
          const lastResult = newResults[newResults.length - 1]
          const isLastFailure = lastResult && !lastResult.success

          const shouldRefresh = failureCount > 3 || (totalCount > 4 && isLastFailure)

          if (shouldRefresh) {
            setTimeout(() => {
              // 直接调用生成新标签的函数，避免依赖循环
              const newTag = generateRandomString(8)
              setCaptchaTag(newTag)
              setValidationResults([]) // 重置验证结果数组
              setHasImageLoadError(false) // 重置图片加载错误状态
            }, 100)
          }

          return newResults
        })
      }
    })

    // 处理验证码输入
    const handleCodeChange = React.useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
      const code = e.target.value

      if (onChange) {
        const newCaptchaData: CaptchaData = {
          code,
          key: captchaTag || '',
          validation: code.length < codeLength ? null :
            (validationMutation.isPending ? 'pending' :
              validationMutation.isSuccess ? 'success' :
                validationMutation.isError ? 'error' : null)
        }
        onChange(newCaptchaData)
      }
    }, [captchaTag, onChange, codeLength, validationMutation.isPending, validationMutation.isSuccess, validationMutation.isError])

    // 使用 ref 来存储最新的 validationMutation.mutate 函数
    const validationMutateRef = React.useRef(validationMutation.mutate)
    validationMutateRef.current = validationMutation.mutate

    // 使用 useEffect 监听 deferred value 的变化来触发验证
    React.useEffect(() => {
      // 当 deferred input code 长度达到验证码长度时自动验证
      if (deferredInputCode.length === codeLength && deferredInputCode) {
        // 检查是否已经验证过相同的验证码，避免重复请求
        const alreadyValidated = validationResults.some(result => result.code === deferredInputCode)
        if (!alreadyValidated) {
          // 创建 AbortController 来支持取消请求
          const controller = new AbortController()
          validationMutateRef.current({ code: deferredInputCode, signal: controller.signal })
        }
      }
    }, [deferredInputCode, codeLength, validationResults])    // 处理输入框失去焦点时验证
    const handleCodeBlur = React.useCallback((e: React.FocusEvent<HTMLInputElement>) => {
      const code = e.target.value
      // 只有当输入长度等于验证码长度时才验证，使用当前输入值而不是 deferred value
      if (code && code.length === codeLength) {
        const alreadyValidated = validationResults.some(result => result.code === code)
        if (!alreadyValidated) {
          // 创建 AbortController 来支持取消请求
          const controller = new AbortController()
          validationMutateRef.current({ code, signal: controller.signal })
        }
      }
    }, [codeLength, validationResults])

    // 初始化验证码标签
    React.useEffect(() => {
      if (captchaType && !captchaTag) {
        const newTag = generateRandomString(8)
        setCaptchaTag(newTag)
        setValidationResults([]) // 重置验证结果数组
      }
    }, [captchaType, captchaTag])

    // 确定要显示的图片URL
    const displayImageUrl = imageUrl
    const hasError = queryError || hasImageLoadError // API错误或图片加载错误
    const showLoading = isLoading && !hasError

    return (
      <div className={cn("flex items-center gap-2", className)}>
        <div className="relative flex-1">
          <Input
            ref={ref}
            type="text"
            value={value?.code || ''}
            onChange={handleCodeChange}
            onBlur={handleCodeBlur}
            placeholder={placeholder}
            className={cn("h-9 pr-8")}
            autoComplete="off"
            {...props}
          />
          {/* 验证状态图标 */}
          <div className="absolute right-2 top-1/2 -translate-y-1/2">
            {validationMutation.isPending && (
              <Loader2 className={cn("h-4 w-4 animate-spin text-muted-foreground")} />
            )}
            {validationMutation.isSuccess && (
              <CheckCircle className={cn("h-4 w-4 text-green-500")} />
            )}
            {validationMutation.isError && (
              <XCircle className={cn("h-4 w-4 text-red-500")} />
            )}
          </div>
        </div>
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <div
                key={`captcha-${captchaType}-${captchaTag}`}
                className="h-9 w-24 border rounded-md overflow-hidden bg-muted flex items-center justify-center cursor-pointer hover:bg-muted/80 transition-colors"
                onClick={refreshCaptcha}
                title="点击刷新验证码"
              >
                {showLoading ? (
                  <Loader2 className={cn("h-4 w-4 animate-spin text-muted-foreground")} />
                ) : hasError ? (
                  <AlertCircle className={cn("h-5 w-5 text-destructive")} />
                ) : displayImageUrl ? (
                  <img
                    src={displayImageUrl}
                    alt="验证码"
                    className="w-full h-full object-contain"
                    onError={() => setHasImageLoadError(true)}
                  />
                ) : (
                  <Loader2 className={cn("h-4 w-4 animate-spin text-muted-foreground")} />
                )}
              </div>
            </TooltipTrigger>
            {hasError && (
              <TooltipContent side="top">
                {queryError?.message || '验证码加载失败，请点击重试'}
              </TooltipContent>
            )}
          </Tooltip>
        </TooltipProvider>
      </div>
    )
  }
)

CaptchaInput.displayName = 'CaptchaInput'

export { CaptchaInput }
export type { CaptchaInputProps }

