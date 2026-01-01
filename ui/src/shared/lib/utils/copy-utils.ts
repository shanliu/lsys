import copy from 'copy-to-clipboard'

/**
 * 创建带有 Toast 提示的复制函数
 * @param showSuccess Toast 成功提示函数
 * @param showError Toast 错误提示函数
 * @returns 返回复制函数
 */
export function createCopyWithToast(
  showSuccess: (message: string) => void,
  showError: (message: string) => void
) {
  return async (text: string, successMessage: string = '已复制到剪贴板') => {
    try {
      const successful = copy(text, {
        debug: false,
        message: 'Press #{key} to copy',
        format: 'text/plain'
      })
      
      if (!successful) {
        throw new Error('复制失败')
      }
      
      showSuccess(successMessage)
      return true
    } catch (error) {
      const copyError = error instanceof Error ? error : new Error('复制失败')
      showError(copyError.message || '复制失败')
      return false
    }
  }
}
