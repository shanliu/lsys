
import {
  differenceInDays,
  differenceInHours,
  differenceInMonths,
  differenceInYears,
  format,
  formatDistanceToNow,
  isValid,
  parseISO
} from "date-fns"
import { zhCN } from "date-fns/locale"
import React, { type JSX } from "react"


/**
 * 处理服务器返回的错误结果，转换为可读的错误字符串
 * @param error - 服务器返回的错误对象，可以是任意类型（支持数组）
 * @param defaultMessage - 默认错误消息，当无法提取具体错误信息时使用
 * @returns 错误字符串
 */
export function formatServerError(error: any, defaultMessage: string = '操作失败，请重试'): string {
  // 如果error为空或undefined
  if (!error) {
    return defaultMessage
  }

  // 如果error是数组，取第一个非空错误递归处理
  if (Array.isArray(error)) {
    const firstError = error.find(e => e != null)
    return firstError ? formatServerError(firstError, defaultMessage) : defaultMessage
  }

  // 优先从 error.data.message 获取（API错误格式）
  if (error.data && typeof error.data.message === 'string') {
    return error.data.message
  }

  // 从 error.message 获取（标准Error对象）
  if (typeof error.message === 'string') {
    return error.message
  }

  // 从 error.response.data.message 获取（axios错误格式）
  if (error.response?.data?.message && typeof error.response.data.message === 'string') {
    return error.response.data.message
  }

  // 从 error.response.data.result.message 获取（自定义API错误格式）
  if (error.response?.data?.result?.message && typeof error.response.data.result.message === 'string') {
    return error.response.data.result.message
  }

  // 如果error.response.data是字符串
  if (error.response?.data && typeof error.response.data === 'string') {
    return error.response.data
  }

  // 如果error本身是字符串
  if (typeof error === 'string') {
    return error
  }

  // 如果都无法提取，返回默认消息
  return defaultMessage
}


/**
 * formatTime 函数的样式常量
 */
export const TIME_STYLE = {
  /** 相对时间元素 - 返回 JSX.Element，带tooltip */
  RELATIVE_ELEMENT: 'relative-element',
  /** 相对时间文本 - 返回 string */
  RELATIVE_TEXT: 'relative-text',
  /** 绝对时间元素 - 返回 JSX.Element，带tooltip */
  ABSOLUTE_ELEMENT: 'absolute-element',
  /** 绝对时间文本 - 返回 string */
  ABSOLUTE_TEXT: 'absolute-text'
} as const

export type TimeStyle = typeof TIME_STYLE[keyof typeof TIME_STYLE]

/**
 * 格式化时间显示
 * @param date 时间对象或时间字符串，可以为 null
 * @param style 显示风格，使用 TIME_STYLE 常量（必传）：
 *   - TIME_STYLE.RELATIVE_ELEMENT: 相对时间元素，返回 JSX.Element
 *   - TIME_STYLE.RELATIVE_TEXT: 相对时间文本，返回 string
 *   - TIME_STYLE.ABSOLUTE_ELEMENT: 绝对时间元素，返回 JSX.Element
 *   - TIME_STYLE.ABSOLUTE_TEXT: 绝对时间文本，返回 string
 * @returns 根据style返回对应类型的时间显示
 */
export function formatTime(date: Date | string | number | null, style: TimeStyle): JSX.Element | string {
  if (!date) {
    if (style === TIME_STYLE.RELATIVE_TEXT || style === TIME_STYLE.ABSOLUTE_TEXT) {
      return '无'
    }
    return React.createElement('span', { title: '无时间信息' }, '无')
  }

  // 解析日期
  const dateObj = typeof date === 'string' ? parseISO(date) : new Date(date)
  const parsedDate = isValid(dateObj) ? dateObj : null

  if (!parsedDate) {
    if (style === TIME_STYLE.RELATIVE_TEXT || style === TIME_STYLE.ABSOLUTE_TEXT) {
      return '无'
    }
    return React.createElement('span', { title: '时间格式无效' }, '无')
  }

  const absoluteTime = format(parsedDate, 'yyyy年MM月dd日 HH:mm:ss')
  const displayTime = format(parsedDate, 'yyyy-MM-dd HH:mm:ss')

  // 绝对时间样式
  if (style === TIME_STYLE.ABSOLUTE_ELEMENT) {
    return React.createElement('span', { title: absoluteTime }, displayTime)
  }

  if (style === TIME_STYLE.ABSOLUTE_TEXT) {
    return displayTime
  }

  // 计算相对时间
  const now = new Date()
  const diffInHours = differenceInHours(now, parsedDate)
  const diffInDays = differenceInDays(now, parsedDate)
  const diffInMonths = differenceInMonths(now, parsedDate)
  const diffInYears = differenceInYears(now, parsedDate)

  // 智能相对时间显示
  let relativeText: string
  if (diffInHours < 1) {
    relativeText = formatDistanceToNow(parsedDate, { addSuffix: true, locale: zhCN })
  } else if (diffInHours < 24) {
    relativeText = `${diffInHours}小时前`
  } else if (diffInDays < 7) {
    relativeText = `${diffInDays}天前`
  } else if (diffInDays < 32) {
    relativeText = `${Math.floor(diffInDays / 7)}周前`
  } else if (diffInMonths < 12) {
    relativeText = `${diffInMonths}个月前`
  } else {
    relativeText = `${diffInYears}年前`
  }

  // 相对时间样式
  if (style === TIME_STYLE.RELATIVE_TEXT) {
    return relativeText
  }

  // 默认：相对时间元素（带title的span元素）
  return React.createElement('span', { title: absoluteTime }, relativeText)
}

/**
 * 格式化秒数为可读的时间字符串
 * @param seconds - 秒数（整数）
 * @returns 格式化后的时间字符串，如 "1分钟"、"2小时30分"、"3天5小时"、"永久"
 * @example
 * formatSeconds(0) // "永久"
 * formatSeconds(60) // "1分钟"
 * formatSeconds(3600) // "1小时"
 * formatSeconds(3661) // "1小时1分1秒"
 * formatSeconds(86400) // "1天"
 * formatSeconds(90061) // "1天1小时1分1秒"
 */
export function formatSeconds(seconds: number): string {
  if (seconds === 0) {
    return "永久"
  }

  if (seconds < 0) {
    return "无效时间"
  }

  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const secs = seconds % 60

  const parts: string[] = []
  if (days > 0) parts.push(`${days}天`)
  if (hours > 0) parts.push(`${hours}小时`)
  if (minutes > 0) parts.push(`${minutes}分`)
  if (secs > 0) parts.push(`${secs}秒`)

  return parts.length > 0 ? parts.join("") : "0秒"
}


