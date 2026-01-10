"use client"

import type { AreaDataSchema } from '@shared/apis/public/area'
import {
  areaFind,
  areaGeo,
  areaList,
  areaRelated,
  areaSearch
} from '@shared/apis/public/area'
import { Button } from '@shared/components/ui/button'
import { Input } from '@shared/components/ui/input'
import { Popover, PopoverContent, PopoverTrigger } from '@shared/components/ui/popover'
import { ScrollArea } from '@shared/components/ui/scroll-area'
import { Drawer, DrawerContent, DrawerHeader, DrawerTitle } from '@apps/main/components/local/drawer'
import { Skeleton } from '@shared/components/ui/skeleton'
import { useToast } from '@shared/contexts/toast-context'
import { useIsMobile } from '@shared/hooks/use-mobile'
import { cn } from '@shared/lib/utils'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { ChevronRight, MapPin, Navigation, Search, X } from 'lucide-react'
import { useCallback, useDeferredValue, useEffect, useRef, useState } from 'react'
import type { z } from 'zod'

// 基础地址数据类型
type AreaData = z.infer<typeof AreaDataSchema>

// 地址选择结果类型
export interface AddressSelection {
  code: string
  name: string
  displayText: string
  // 地理位置信息（仅在通过GEO获取时提供）
  geo?: {
    latitude: number
    longitude: number
  }
}

// 组件属性接口
export interface AddressSelectorProps {
  value?: string
  onChange?: (selection: AddressSelection | null) => void
  placeholder?: string
  disabled?: boolean
  className?: string
  enableGeolocation?: boolean // 是否启用地理位置获取功能，默认为 true
  enableSearch?: boolean // 是否启用搜索功能，默认为 true
  selectLevel?: number // 选择到哪个层级：1=省级，2=市级，3=区县级，默认为无限制（选择到最底层）
}

/**
 * 地址选择器组件
 */
export function AddressSelector({
  value = '',
  onChange,
  placeholder = '请选择地址',
  disabled = false,
  className,
  enableGeolocation = true, // 默认启用地理位置功能
  enableSearch = true, // 默认启用搜索功能
  selectLevel // 选择到哪个层级，undefined 表示选择到最底层
}: AddressSelectorProps) {
  // 状态管理
  const [open, setOpen] = useState(false)
  const [displayText, setDisplayText] = useState('')
  const [currentValue, setCurrentValue] = useState(value || '')
  const [searchKeyword, setSearchKeyword] = useState('')
  const [searchResults, setSearchResults] = useState<AreaData[][]>([])
  const [listData, setListData] = useState<AreaData[][]>([])
  const [currentPath, setCurrentPath] = useState<AreaData[]>([])
  const [popoverWidth, setPopoverWidth] = useState(400)
  const [isAreaLoading, setIsAreaLoading] = useState(false) // 区域加载状态
  const [loadingAreaCode, setLoadingAreaCode] = useState<string | null>(null) // 正在加载的区域编码
  const [isGeoLoading, setIsGeoLoading] = useState(false) // 获取位置加载状态
  const [isSearchSelecting, setIsSearchSelecting] = useState(false) // 搜索选择加载状态
  const [isManualSelection, setIsManualSelection] = useState(false) // 标记是否为手动选择
  const [currentAbortController, setCurrentAbortController] = useState<AbortController | null>(null) // 当前请求的取消控制器
  const [lastClickTime, setLastClickTime] = useState<number>(0) // 防止快速点击的时间戳
  // 新增：缓存当前地址的完整层级数据，用于重新打开时直接复用
  const [cachedAreaData, setCachedAreaData] = useState<{ [key: string]: { listData: AreaData[][], currentPath: AreaData[] } }>({})
  const [needsApiCall, setNeedsApiCall] = useState(false) // 标记是否需要 API 调用

  // 新增：备份原始状态，用于取消时恢复
  const [originalState, setOriginalState] = useState<{
    currentValue: string
    displayText: string
    currentPath: AreaData[]
    listData: AreaData[][]
  } | null>(null)

  // Toast 通知
  const { error: showError } = useToast()

  // Query client for managing queries
  const queryClient = useQueryClient()

  // Check if mobile device
  const isMobile = useIsMobile()

  // 引用
  const inputRef = useRef<HTMLInputElement>(null)
  const triggerRef = useRef<HTMLDivElement>(null)
  const scrollAreaRefs = useRef<(HTMLDivElement | null)[]>([]) // 用于存储每列的滚动区域引用
  const mobileSectionRefs = useRef<(HTMLDivElement | null)[]>([]) // 用于存储移动端每个区域的引用

  // 自定义防抖 hook
  const useDebounce = (value: string, delay: number) => {
    const [debouncedValue, setDebouncedValue] = useState(value)

    useEffect(() => {
      const handler = setTimeout(() => {
        setDebouncedValue(value)
      }, delay)

      return () => {
        clearTimeout(handler)
      }
    }, [value, delay])

    return debouncedValue
  }

  // 防抖搜索关键词 - 使用更明显的防抖延迟
  const debouncedSearchKeyword = useDebounce(searchKeyword, 500) // 500ms 防抖
  const deferredSearchKeyword = useDeferredValue(debouncedSearchKeyword) // 额外的 React 延迟优化
  const isSearchMode = enableSearch && deferredSearchKeyword.trim().length > 0

  // React Query hooks
  // 1. 无初始值时：加载省份列表
  const { data: initialData, isLoading: isInitialLoading } = useQuery({
    queryKey: ['area-list', ''],
    queryFn: ({ signal }) => areaList({ code: '' }, { signal }),
    enabled: !currentValue || currentValue === '', // 只在无初始值时启用
    staleTime: 5 * 60 * 1000,
  })

  // 2. 搜索功能
  const { data: searchData, isLoading: isSearchLoading } = useQuery({
    queryKey: ['area-search', deferredSearchKeyword],
    queryFn: ({ signal }) => areaSearch({ key_word: deferredSearchKeyword.trim() }, { signal }),
    enabled: enableSearch && isSearchMode && deferredSearchKeyword.trim().length > 0,
    staleTime: 2 * 60 * 1000,
  })

  // 3. 有初始值且用户打开选择框时：使用 areaRelated 加载完整层级数据
  // 优化：只有在缓存中没有数据时才发起网络请求
  const { data: relatedData, isLoading: isRelatedLoading } = useQuery({
    queryKey: ['area-related', currentValue],
    queryFn: async ({ signal }) => {
      return areaRelated({ code: currentValue }, { signal })
    },
    enabled: !!currentValue && currentValue !== '' && open && needsApiCall, // 只有需要API调用时才启用
    staleTime: 10 * 60 * 1000, // 10分钟内使用缓存
    gcTime: 15 * 60 * 1000, // 缓存15分钟后清理
    refetchOnWindowFocus: false, // 窗口重新获得焦点时不自动刷新
    refetchOnMount: false, // 组件重新挂载时不自动刷新，依赖缓存
  })

  // 4. 获取路径文本（仅在有初始值但无手动选择数据时调用，用于页面加载时的初始显示）
  const { data: pathData, isLoading: isPathLoading } = useQuery({
    queryKey: ['area-find', currentValue],
    queryFn: ({ signal }) => areaFind({ code: currentValue }, { signal }),
    enabled: !!currentValue && currentValue !== '' && !isManualSelection && currentPath.length === 0, // 只在有初始值且无路径数据时调用
    staleTime: 5 * 60 * 1000,
  })

  // 构建路径文本
  const buildPathText = useCallback((path: AreaData[]): string => {
    return path.map(item => item.name).join(' / ')
  }, [])

  // 保存地址数据到缓存
  const saveAreaDataToCache = useCallback((code: string, listData: AreaData[][], currentPath: AreaData[]) => {
    if (code && listData.length > 0 && currentPath.length > 0) {
      setCachedAreaData(prev => ({
        ...prev,
        [code]: {
          listData: JSON.parse(JSON.stringify(listData)), // 深拷贝避免引用问题
          currentPath: JSON.parse(JSON.stringify(currentPath))
        }
      }))
    }
  }, [])

  // 完成地址选择的统一函数
  const completeSelection = useCallback((selection: AddressSelection, newPath: AreaData[], currentListData: AreaData[][]) => {
    // 保存到缓存
    saveAreaDataToCache(selection.code, currentListData, newPath)

    // 更新状态
    setCurrentPath(newPath)
    setCurrentValue(selection.code)

    // 标记为已完成选择，防止关闭时恢复原状态
    setIsManualSelection(true)

    setOpen(false)

    // 通知外部
    onChange?.(selection)
  }, [saveAreaDataToCache, onChange])

  // Effects - 动态计算弹窗宽度
  useEffect(() => {
    if (triggerRef.current && open) {
      const rect = triggerRef.current.getBoundingClientRect()

      // 计算每列的实际内容宽度，使用与渲染时相同的计算公式
      let totalWidth = 0
      listData.forEach((areas) => {
        if (areas.length > 0) {
          // 计算这一列最长的文本宽度，优化列宽算法减少留白
          const maxLength = Math.max(...areas.map(area => area.name.length))
          const columnWidth = Math.max(120, Math.min(180, 60 + maxLength * 12))
          totalWidth += columnWidth
        }
      })

      // 加上搜索栏和按钮的最小宽度
      const searchBarMinWidth = 320
      const finalWidth = Math.max(totalWidth, searchBarMinWidth, rect.width)
      setPopoverWidth(finalWidth)
    }
  }, [open, listData])

  // 打开选择器时的缓存检查逻辑
  useEffect(() => {
    if (open && currentValue && currentValue !== '') {
      // 检查是否有缓存数据
      const cached = cachedAreaData[currentValue]
      if (cached && cached.listData.length > 0 && cached.currentPath.length > 0) {
        setListData(cached.listData)
        setCurrentPath(cached.currentPath)
        setNeedsApiCall(false) // 不需要 API 调用
      } else {
        setNeedsApiCall(true) // 需要 API 调用
      }
    } else if (!currentValue || currentValue === '') {
      setNeedsApiCall(false) // 无值时不需要 API 调用
    }
  }, [open, currentValue, cachedAreaData])

  // 数据加载优先级处理
  useEffect(() => {
    // 优先级1：用户打开选择框且有初始值时，使用 areaRelated 的完整层级数据（包含缓存）
    if (relatedData?.status && relatedData.response?.area && Array.isArray(relatedData.response.area) && open) {
      const areaLevels = relatedData.response.area

      // 过滤掉空的层级数据，确保只显示有数据的层级
      let filteredLevels = areaLevels.filter(level => level && level.length > 0)

      // 根据 selectLevel 限制显示的层级数据
      if (selectLevel && filteredLevels.length > selectLevel) {
        filteredLevels = filteredLevels.slice(0, selectLevel)
      }

      if (filteredLevels.length > 0) {
        setListData(filteredLevels)

        // 从 areaRelated 数据中提取选中路径
        // 优化算法：基于行政区划编码规律和API返回的selected字段
        const selectedPath: AreaData[] = []
        const targetCode = currentValue

        // 方法1: 优先使用API返回的selected字段
        for (let levelIndex = 0; levelIndex < filteredLevels.length; levelIndex++) {
          const level = filteredLevels[levelIndex]

          // 首先尝试找到selected="1"的项目
          let selectedItem = level.find(item => (item as any).selected === '1')

          // 如果没有selected字段或者找不到，回退到编码匹配算法
          if (!selectedItem) {
            // 中国行政区划编码规律匹配
            const codeLengths = [2, 4, 6, 9, 12]
            if (levelIndex < codeLengths.length) {
              const levelCodeLength = codeLengths[levelIndex]
              const levelCodePrefix = targetCode.substring(0, levelCodeLength)

              selectedItem = level.find(item =>
                item.code.startsWith(levelCodePrefix) ||
                item.code === targetCode
              )
            }
          }

          if (selectedItem) {
            selectedPath.push(selectedItem)
          } else {
            // 如果某一级找不到匹配项，停止构建路径
            break
          }
        }

        if (selectedPath.length > 0) {
          setCurrentPath(selectedPath)

          // 保存 areaRelated 数据到缓存，以便下次重新打开时使用
          saveAreaDataToCache(currentValue, filteredLevels, selectedPath)
        }
      }
      return
    }

    // 优先级2：无初始值时，使用 areaList 的省份列表
    if (initialData?.status && initialData.response?.area && (!currentValue || currentValue === '')) {
      setListData([initialData.response.area])
      return
    }
  }, [relatedData, initialData, currentValue, open, selectLevel, saveAreaDataToCache])

  useEffect(() => {
    if (searchData?.status && searchData.response?.area) {
      setSearchResults(searchData.response.area)
    } else {
      setSearchResults([])
    }
  }, [searchData])

  // 自动滚动到选中项
  useEffect(() => {
    if (open && currentPath.length > 0 && listData.length > 0) {
      // 延迟执行滚动，确保DOM已渲染
      const timer = setTimeout(() => {
        if (isMobile) {
          // 移动端：滚动到最新添加的区域（下一级选择区域）
          const targetIndex = currentPath.length // 下一级的索引
          const targetSection = mobileSectionRefs.current[targetIndex]
          if (targetSection) {
            targetSection.scrollIntoView({
              behavior: 'smooth',
              block: 'start'
            })
          } else if (currentPath.length === listData.length) {
            // 如果已经是最后一级，滚动到当前选中项
            const currentSection = mobileSectionRefs.current[currentPath.length - 1]
            if (currentSection) {
              const scrollArea = scrollAreaRefs.current[currentPath.length - 1]
              if (scrollArea) {
                const selectedButton = scrollArea.querySelector(`[data-area-code="${currentPath[currentPath.length - 1].code}"]`)
                if (selectedButton) {
                  selectedButton.scrollIntoView({
                    behavior: 'smooth',
                    block: 'center'
                  })
                }
              }
            }
          }
        } else {
          // 桌面端：滚动到每列的选中项
          currentPath.forEach((pathItem, columnIndex) => {
            const scrollArea = scrollAreaRefs.current[columnIndex]
            if (scrollArea) {
              const selectedButton = scrollArea.querySelector(`[data-area-code="${pathItem.code}"]`)
              if (selectedButton) {
                selectedButton.scrollIntoView({
                  behavior: 'smooth',
                  block: 'center'
                })
              }
            }
          })
        }
      }, 100)

      return () => clearTimeout(timer)
    }
  }, [open, currentPath, listData, isMobile])

  // 从当前路径更新显示文本
  useEffect(() => {
    if (currentPath.length > 0) {
      const newDisplayText = buildPathText(currentPath)
      setDisplayText(newDisplayText)
    } else if (!currentValue) {
      setDisplayText('')
    }
  }, [currentPath, currentValue, buildPathText])

  useEffect(() => {
    // areaFind 仅用于初始化时获取显示文本（当没有其他数据源时）
    // 如果是手动选择操作（包括清除），则不使用 pathData
    if (!isManualSelection && pathData?.status && pathData.response?.area && Array.isArray(pathData.response.area) && currentPath.length === 0) {
      let path = pathData.response.area

      // 根据 selectLevel 过滤超过层级的数据
      if (selectLevel && path.length > selectLevel) {
        path = path.slice(0, selectLevel)

        // 更新 currentValue 为过滤后路径的最后一个地区的 code
        if (path.length > 0) {
          const newCode = path[path.length - 1].code
          setCurrentValue(newCode)

          // 如果值发生了变化，通知外部组件
          if (newCode !== currentValue && onChange) {
            const selection: AddressSelection = {
              code: newCode,
              name: path[path.length - 1].name,
              displayText: buildPathText(path)
            }
            onChange(selection)
          }
        }
      }

      setCurrentPath(path)
    }
  }, [pathData, currentValue, buildPathText, selectLevel, onChange, currentPath.length, isManualSelection])

  useEffect(() => {
    setCurrentValue(value || '')
    // 外部设置值时，重置手动选择标志和路径数据
    setIsManualSelection(false)
    if (!value || value === '') {
      setCurrentPath([])
      setDisplayText('')
    } else if (value !== currentValue) {
      // 值变化时清空路径，让系统重新加载
      setCurrentPath([])
    }
  }, [value, currentValue])

  // 清理 AbortController
  useEffect(() => {
    return () => {
      if (currentAbortController) {
        currentAbortController.abort()
      }
    }
  }, [currentAbortController])

  // 取消当前请求的辅助函数
  const cancelCurrentRequest = useCallback(() => {
    if (currentAbortController) {
      currentAbortController.abort()
      setCurrentAbortController(null)
    }
  }, [currentAbortController])

  // 使用 TanStack Query 预取区域数据的辅助函数
  const prefetchAreaList = useCallback(async (code: string, signal?: AbortSignal) => {
    const queryKey = ['area-list', code]

    // 取消之前可能存在的同类查询
    await queryClient.cancelQueries({ queryKey })

    // 预取数据
    return queryClient.fetchQuery({
      queryKey,
      queryFn: async ({ signal }) => {
        const result = await areaList({ code }, { signal })
        if (!result.status || !result.response?.area) {
          throw new Error('Failed to fetch area data')
        }
        return result.response.area
      },
      staleTime: 5 * 60 * 1000, // 5分钟缓存
    })
  }, [queryClient])

  // 使用 TanStack Query 获取地理位置区域数据的辅助函数
  const fetchAreaGeo = useCallback(async (lat: number, lng: number) => {
    const queryKey = ['area-geo', lat, lng]

    return queryClient.fetchQuery({
      queryKey,
      queryFn: async ({ signal }) => {
        const result = await areaGeo({ lat, lng }, { signal })
        if (!result.status || !result.response?.area) {
          throw new Error('Failed to fetch geo area data')
        }
        return result.response.area
      },
      staleTime: 5 * 60 * 1000, // 5分钟缓存
    })
  }, [queryClient])

  // 处理地区选择 - 核心功能：逐级选择
  const handleAreaSelect = useCallback(async (area: AreaData, columnIndex: number) => {
    // 防止快速点击（300ms内的重复点击被忽略）
    const now = Date.now()
    if (now - lastClickTime < 300) {
      return
    }
    setLastClickTime(now)

    // 防止在加载过程中启动新的请求
    if (isAreaLoading) {
      return
    }

    // 检查是否达到了指定的选择层级
    if (selectLevel && (columnIndex + 1) >= selectLevel) {
      // 达到指定层级，直接完成选择
      const newPath = [...currentPath.slice(0, columnIndex), area]
      const selection: AddressSelection = {
        code: area.code,
        name: area.name,
        displayText: buildPathText(newPath)
      }

      // 使用统一的完成选择函数
      completeSelection(selection, newPath, listData)
      return
    }

    // 使用 leaf 字段判断是否为叶子节点，避免不必要的API调用
    if (area.leaf === '1') {
      // 叶子节点，完成选择
      const newPath = [...currentPath.slice(0, columnIndex), area]
      const selection: AddressSelection = {
        code: area.code,
        name: area.name,
        displayText: buildPathText(newPath)
      }

      // 使用统一的完成选择函数
      completeSelection(selection, newPath, listData)
      return
    }

    // 非叶子节点，检查是否需要加载下级数据
    const nextColumnIndex = columnIndex + 1

    // 关键修复：当用户选择的不是当前路径中的项目时，总是需要调用API获取新数据
    const isCurrentPathItem = currentPath[columnIndex]?.code === area.code
    const hasNextLevelData = listData[nextColumnIndex] && listData[nextColumnIndex].length > 0

    // 只有在选择的是当前路径中的项目且已有下级数据时，才复用现有数据
    if (isCurrentPathItem && hasNextLevelData) {
      // 选择的是当前路径项目且已有下级数据，无需API调用
      // 更新当前路径（实际不变，但确保状态一致）
      const newPath = [...currentPath.slice(0, columnIndex), area]
      setCurrentPath(newPath)

      // 清除当前级别之后的路径，保留到下一级
      const updatedListData = listData.slice(0, nextColumnIndex + 1)
      setListData(updatedListData)

      return
    }

    // 没有下级数据，需要API调用
    // 取消之前的请求
    cancelCurrentRequest()

    // 创建新的 AbortController
    const abortController = new AbortController()
    setCurrentAbortController(abortController)

    setIsAreaLoading(true) // 开始加载
    setLoadingAreaCode(area.code) // 记录正在加载的区域编码

    // 立即清除当前级别之后的历史数据，避免显示过时信息
    const newListData = listData.slice(0, columnIndex + 1)
    setListData(newListData)

    // 更新当前路径
    const newPath = [...currentPath.slice(0, columnIndex), area]
    setCurrentPath(newPath)

    try {
      // 使用优化的预取函数
      const areaData = await prefetchAreaList(area.code, abortController.signal)

      // 请求成功后清理 AbortController
      setCurrentAbortController(null)

      if (areaData && areaData.length > 0) {
        // 有下级区域，保持弹窗打开，添加新列
        // 检查是否超过层级限制
        const nextLevelIndex = columnIndex + 2 // +1 是当前选择的层级，+2 是下一级
        if (!selectLevel || nextLevelIndex <= selectLevel) {
          // 添加新的下级数据
          const updatedListData = [...newListData, areaData]
          setListData(updatedListData)
        } else {
          // 超过层级限制，完成选择
          const selection: AddressSelection = {
            code: area.code,
            name: area.name,
            displayText: buildPathText(newPath)
          }

          // 使用统一的完成选择函数（注意这里使用 newListData，因为没有添加下级数据）
          completeSelection(selection, newPath, newListData)
        }
      } else {
        // 虽然leaf不是1，但实际没有下级区域，完成选择
        const selection: AddressSelection = {
          code: area.code,
          name: area.name,
          displayText: buildPathText(newPath)
        }

        // 使用统一的完成选择函数
        completeSelection(selection, newPath, newListData)
      }
    } catch (err) {
      // 清理 AbortController
      setCurrentAbortController(null)

      // 如果是取消请求，不显示错误信息
      if (err instanceof Error && err.name === 'AbortError') {
        return
      }

      console.error('查询下级区域失败:', err)
      showError('查询地区信息失败')
    } finally {
      setIsAreaLoading(false) // 结束加载
      setLoadingAreaCode(null) // 清除加载区域编码
    }
  }, [listData, currentPath, buildPathText, selectLevel, cancelCurrentRequest, prefetchAreaList, lastClickTime, isAreaLoading, completeSelection, showError])

  // 其他处理函数
  const handleSearchSelect = useCallback(async (path: AreaData[]) => {
    if (path.length === 0) return

    const lastArea = path[path.length - 1]

    // 标记为手动选择
    setIsManualSelection(true)

    // 检查是否达到了指定的选择层级
    if (selectLevel && path.length >= selectLevel) {
      // 达到指定层级，截取路径到指定层级
      const truncatedPath = path.slice(0, selectLevel)
      const targetArea = truncatedPath[truncatedPath.length - 1]

      const selection: AddressSelection = {
        code: targetArea.code,
        name: targetArea.name,
        displayText: buildPathText(truncatedPath)
      }

      // 使用统一的完成选择函数（搜索选择暂时用空的listData，因为还没构建）
      completeSelection(selection, truncatedPath, [])
      setSearchKeyword('')
      return
    }

    // 开始加载，显示UI提示
    setIsSearchSelecting(true)

    // 构建层级列表数据以便显示选择路径
    try {
      const newListData: AreaData[][] = []

      // 构建每一级的数据
      for (let i = 0; i < path.length; i++) {
        if (i === 0) {
          // 第一级，获取顶级数据
          if (initialData?.status && initialData.response?.area) {
            newListData.push(initialData.response.area)
          }
        } else {
          // 后续级别，根据上一级的code获取
          const parentCode = path[i - 1].code
          try {
            const areaData = await prefetchAreaList(parentCode)
            if (areaData) {
              newListData.push(areaData)
            }
          } catch (err) {
            // 如果是取消请求，不处理错误
            if (err instanceof Error && err.name === 'AbortError') {
              return
            }
            console.error('获取区域数据失败:', err)
            throw err
          }
        }
      }

      setListData(newListData)
      setCurrentPath(path)

      // 如果是叶子节点，完成选择
      if (lastArea.leaf === '1') {
        const selection: AddressSelection = {
          code: lastArea.code,
          name: lastArea.name,
          displayText: buildPathText(path)
        }

        // 使用统一的完成选择函数
        completeSelection(selection, path, newListData)
        setSearchKeyword('')
      } else {
        // 非叶子节点，切换到地址选择器模式，清空搜索关键词
        setCurrentValue(lastArea.code)
        setSearchKeyword('')

        // 标记为手动选择状态
        setIsManualSelection(true)

        // 通知父组件当前选择（非叶子节点，但用户已选择到这一级）
        const selection: AddressSelection = {
          code: lastArea.code,
          name: lastArea.name,
          displayText: buildPathText(path)
        }
        onChange?.(selection)

        // 检查是否需要加载下级区域
        if (newListData.length === path.length) {
          // 检查是否超过层级限制
          if (!selectLevel || newListData.length < selectLevel) {
            try {
              const areaData = await prefetchAreaList(lastArea.code)
              if (areaData && areaData.length > 0) {
                setListData([...newListData, areaData])
              }
            } catch (err) {
              // 如果是取消请求，不处理错误
              if (err instanceof Error && err.name === 'AbortError') {
                return
              }
              console.error('加载下级区域失败:', err)
            }
          }
        }
      }
    } catch (err) {
      console.error('构建搜索选择路径失败:', err)
      showError('切换到地址选择器失败')
    } finally {
      // 结束加载状态
      setIsSearchSelecting(false)
    }
  }, [buildPathText, initialData, selectLevel, prefetchAreaList, completeSelection, showError, onChange])

  const handleClear = useCallback(async () => {
    // 标记为手动选择（清除也是用户操作）
    setIsManualSelection(true)

    // 清除所有状态
    setCurrentValue('')
    setDisplayText('')
    setCurrentPath([])
    setSearchKeyword('') // 清空搜索关键词
    setNeedsApiCall(true) // 重置为需要API调用（下次打开时）

    // 清理缓存数据
    setCachedAreaData({})

    // 重置列表数据
    setListData([])

    onChange?.(null)
  }, [onChange])

  const handleGetLocation = useCallback(async () => {
    if (!navigator.geolocation) {
      showError('您的浏览器不支持地理位置功能')
      return
    }

    // 标记为手动选择
    setIsManualSelection(true)

    setIsGeoLoading(true)

    try {
      const position = await new Promise<GeolocationPosition>((resolve, reject) => {
        const options: PositionOptions = {
          enableHighAccuracy: true, // 启用高精度
          timeout: 15000, // 15秒超时
          maximumAge: 300000 // 5分钟内的缓存位置可用
        }

        navigator.geolocation.getCurrentPosition(resolve, reject, options)
      })

      const { latitude, longitude } = position.coords
      const path = await fetchAreaGeo(latitude, longitude)

      if (path && Array.isArray(path) && path.length > 0) {
        const area = path[path.length - 1]

        // 构建层级列表数据以便显示选择路径
        let newListData: AreaData[][] = []
        try {
          // 构建每一级的数据
          for (let i = 0; i < path.length; i++) {
            if (i === 0) {
              // 第一级，获取顶级数据
              // 如果 initialData 可用就使用，否则通过 API 获取
              if (initialData?.status && initialData.response?.area) {
                newListData.push(initialData.response.area)
              } else {
                // 当 initialData 不可用时（比如当前有 value），直接调用 API 获取省份数据
                try {
                  const topLevelData = await prefetchAreaList('')
                  if (topLevelData) {
                    newListData.push(topLevelData)
                  }
                } catch (err) {
                  if (err instanceof Error && err.name === 'AbortError') {
                    return
                  }
                  console.error('获取顶级区域数据失败:', err)
                  throw err
                }
              }
            } else {
              // 后续级别，根据上一级的code获取
              const parentCode = path[i - 1].code
              try {
                const areaData = await prefetchAreaList(parentCode)
                if (areaData) {
                  newListData.push(areaData)
                }
              } catch (err) {
                // 如果是取消请求，不处理错误
                if (err instanceof Error && err.name === 'AbortError') {
                  return
                }
                console.error('获取区域数据失败:', err)
                throw err
              }
            }
          }

          setListData(newListData)
        } catch (err) {
          console.error('构建地理位置选择路径失败:', err)
        }

        // 根据 selectLevel 截取路径和区域
        let finalPath = path
        let finalArea = area
        let finalListData = newListData
        if (selectLevel && path.length > selectLevel) {
          finalPath = path.slice(0, selectLevel)
          finalArea = finalPath[finalPath.length - 1]
          // 同时截取 listData 到对应层级
          finalListData = newListData.slice(0, selectLevel)
        }

        // 立即返回地理位置信息给父组件，包含经纬度坐标
        const selection: AddressSelection = {
          code: finalArea.code,
          name: finalArea.name,
          displayText: buildPathText(finalPath),
          geo: {
            latitude,
            longitude
          }
        }

        // 保存到缓存（地理位置选择也要缓存层级数据）
        saveAreaDataToCache(finalArea.code, finalListData, finalPath)

        setCurrentValue(finalArea.code)
        setCurrentPath(finalPath)
        setListData(finalListData)
        setSearchKeyword('')

        // 标记为已选择但不关闭选择器，让用户看到选择结果
        setIsManualSelection(true)

        // 通过 onChange 回调返回地理位置信息
        onChange?.(selection)
      } else {
        showError('定位失败，请手动选择地址')
      }
    } catch (err) {
      console.error('获取位置失败:', err)

      // 详细的错误处理
      if (err instanceof GeolocationPositionError) {
        switch (err.code) {
          case GeolocationPositionError.PERMISSION_DENIED:
            showError('位置访问被拒绝，请在浏览器设置中允许位置访问权限')
            break
          case GeolocationPositionError.POSITION_UNAVAILABLE:
            showError('无法获取位置信息，可能网络不可用或GPS信号不佳')
            break
          case GeolocationPositionError.TIMEOUT:
            showError('获取位置超时，请检查网络连接或手动选择地址')
            break
          default:
            showError('获取位置失败，请手动选择地址')
        }
      } else {
        showError('获取位置失败，请检查权限设置或手动选择地址')
      }
    } finally {
      setIsGeoLoading(false)
    }
  }, [buildPathText, initialData, prefetchAreaList, fetchAreaGeo, saveAreaDataToCache, onChange, showError, selectLevel])

  const handleOpenChange = useCallback((newOpen: boolean) => {
    if (newOpen) {
      // 打开时备份当前状态并重置手动选择标志
      setOriginalState({
        currentValue,
        displayText,
        currentPath: [...currentPath],
        listData: listData.map(level => [...level])
      })

      // 重置手动选择标志，为本次选择做准备
      setIsManualSelection(false)

      if (enableSearch) {
        setSearchKeyword('')
      }
    } else {
      // 关闭时检查是否需要恢复原状态
      // 如果用户没有完成选择就关闭了选择器，恢复到原始状态
      if (originalState && !isManualSelection) {
        setCurrentValue(originalState.currentValue)
        setDisplayText(originalState.displayText)
        setCurrentPath(originalState.currentPath)
        setListData(originalState.listData)
      }

      if (enableSearch) {
        setSearchResults([])
      }

      // 清除备份状态
      setOriginalState(null)
    }

    setOpen(newOpen)
  }, [enableSearch, currentValue, displayText, currentPath, listData, originalState, isManualSelection])

  // 渲染函数
  const renderAreaList = (areas: AreaData[], columnIndex: number) => (
    <ScrollArea
      ref={(el) => {
        if (scrollAreaRefs.current) {
          scrollAreaRefs.current[columnIndex] = el
        }
      }}
      className={cn(
        isMobile ? "h-full" : "h-64"
      )}
      data-vaul-no-drag
      onTouchMove={isMobile ? (e) => e.stopPropagation() : undefined}
    >
      <div className="space-y-1">
        {areas.map((area) => {
          const isSelected = currentPath[columnIndex]?.code === area.code
          const isLoading = isAreaLoading && loadingAreaCode === area.code
          const isAnyLoading = isAreaLoading // 任何加载状态都禁用按钮

          return (
            <Button
              key={area.code}
              data-area-code={area.code}
              variant={isSelected ? "secondary" : "ghost"}
              className={cn(
                "w-full justify-start h-auto py-2 px-3 text-left",
                isSelected && "bg-muted font-medium",
                isAnyLoading && !isLoading && "opacity-50" // 其他按钮半透明显示
              )}
              onClick={() => handleAreaSelect(area, columnIndex)}
              disabled={isAnyLoading} // 任何加载状态都禁用
            >
              <div className="flex items-center w-full">
                <span className="truncate mr-1">{area.name}</span>
                {isLoading ? (
                  <div className="h-3 w-3 animate-spin rounded-full border border-muted-foreground border-t-transparent flex-shrink-0" />
                ) : (
                  // 箭头显示逻辑：不是叶子节点且未达到层级限制时显示
                  area.leaf !== '1' && (!selectLevel || (columnIndex + 1) < selectLevel) ? (
                    <ChevronRight className={cn("h-3 w-3 text-muted-foreground flex-shrink-0")} />
                  ) : null
                )}
              </div>
            </Button>
          )
        })}
      </div>
    </ScrollArea>
  )

  const renderSearchResults = () => {
    // 检查是否正在等待防抖
    const isWaitingForDebounce = searchKeyword.trim() !== debouncedSearchKeyword.trim()

    return (
      <div className="relative">
        <ScrollArea 
          className={cn(
            isMobile ? "h-full" : "h-64"
          )}
          data-vaul-no-drag
          onTouchMove={isMobile ? (e) => e.stopPropagation() : undefined}
        >
          <div className="space-y-2">
            {isWaitingForDebounce ? (
              <div className="text-center text-muted-foreground py-8">
                <div className="flex items-center justify-center gap-2">
                  <div className="h-3 w-3 animate-spin rounded-full border border-muted-foreground border-t-transparent" />
                  <span>准备搜索...</span>
                </div>
              </div>
            ) : searchResults.length === 0 ? (
              <div className="text-center text-muted-foreground py-8">
                {isSearchLoading ? (
                  <div className="flex items-center justify-center gap-2">
                    <div className="h-3 w-3 animate-spin rounded-full border border-muted-foreground border-t-transparent" />
                    <span>搜索中...</span>
                  </div>
                ) : deferredSearchKeyword.trim().length > 0 ? (
                  '无搜索结果'
                ) : (
                  '请输入搜索关键词'
                )}
              </div>
            ) : (
              searchResults.map((path, index) => (
                <Button
                  key={index}
                  variant="ghost"
                  className={cn("w-full justify-start h-auto py-2 px-3 text-left")}
                  onClick={() => handleSearchSelect(path)}
                  disabled={isSearchSelecting}
                >
                  <span className="truncate">{buildPathText(path)}</span>
                </Button>
              ))
            )}
          </div>
        </ScrollArea>

        {/* 整体加载遮罩层 */}
        {isSearchSelecting && (
          <div className="absolute inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <div className="h-4 w-4 animate-spin rounded-full border-2 border-muted-foreground border-t-transparent" />
              <span>正在加载地址数据...</span>
            </div>
          </div>
        )}
      </div>
    )
  }

  const isLoading = isInitialLoading || isSearchLoading

  // 渲染搜索和定位工具栏
  const renderToolbar = () => (
    <div className="flex items-center gap-2 p-3 border-t bg-background">
      {enableSearch && (
        <div className="flex-1 flex items-center gap-2 border rounded-md px-2">
          <Search className={cn("h-4 w-4 text-muted-foreground flex-shrink-0")} />
          <Input
            ref={inputRef}
            placeholder="搜索地址..."
            value={searchKeyword}
            onChange={(e) => setSearchKeyword(e.target.value)}
            className={cn("border-0 shadow-none focus-visible:ring-0 h-9")}
          />
        </div>
      )}
      {enableGeolocation && (
        <Button
          variant="outline"
          size="sm"
          onClick={handleGetLocation}
          disabled={isGeoLoading}
          className={cn("flex-shrink-0")}
        >
          {isGeoLoading ? (
            <div className="h-4 w-4 animate-spin rounded-full border border-muted-foreground border-t-transparent" />
          ) : (
            <Navigation className={cn("h-4 w-4")} />
          )}
          <span className={cn("ml-1", isMobile && "hidden sm:inline")}>
            {isGeoLoading ? '定位中' : '定位'}
          </span>
        </Button>
      )}
    </div>
  )

  // 渲染内容区域
  const renderContent = () => (
    <>
      {isLoading ? (
        <div className="p-4 space-y-2">
          {Array.from({ length: 6 }).map((_, i) => (
            <Skeleton key={i} className={cn("h-8 w-full")} />
          ))}
        </div>
      ) : isSearchMode ? (
        renderSearchResults()
      ) : (
        <div className={cn("flex", isMobile ? "flex-col" : "overflow-hidden")}>
          {listData.map((areas, columnIndex) => {
            // 计算每列的动态宽度，优化列宽算法减少留白
            const maxLength = areas.length > 0 ? Math.max(...areas.map(area => area.name.length)) : 0
            const columnWidth = Math.max(120, Math.min(180, 60 + maxLength * 12))

            return (
              <div
                key={columnIndex}
                ref={(el) => {
                  if (isMobile && mobileSectionRefs.current) {
                    mobileSectionRefs.current[columnIndex] = el
                  }
                }}
                className={cn(
                  isMobile ? "border-b last:border-b-0 pb-2 mb-2" : "border-r last:border-r-0 flex-shrink-0"
                )}
                style={isMobile ? undefined : { width: `${columnWidth}px` }}
              >
                {isMobile && (
                  <div className="text-sm font-medium text-muted-foreground px-3 py-2 sticky top-0 bg-background z-10">
                    {columnIndex === 0 ? '省/直辖市' : columnIndex === 1 ? '市' : columnIndex === 2 ? '区/县' : columnIndex === 3 ? '乡镇' : columnIndex === 4 ? '村/街道' : `第${columnIndex + 1}级`}
                  </div>
                )}
                {renderAreaList(areas, columnIndex)}
              </div>
            )
          })}
        </div>
      )}
    </>
  )

  const trigger = (
    <div
      ref={triggerRef}
      className={cn(
        "flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors",
        "cursor-pointer",
        "file:border-0 file:bg-transparent file:text-sm file:font-medium",
        "placeholder:text-muted-foreground",
        "focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring",
        "disabled:cursor-not-allowed disabled:opacity-50",
        disabled && "cursor-not-allowed opacity-50"
      )}
    >
      <div className="flex items-center flex-1 gap-2 min-w-0 overflow-hidden">
        <MapPin className={cn("h-4 w-4 text-muted-foreground flex-shrink-0")} />
        <span className={cn(
          "flex-1 overflow-x-auto overflow-y-hidden whitespace-nowrap scrollbar-hide",
          (!displayText && !((isRelatedLoading || isPathLoading) && !isManualSelection)) && "text-muted-foreground"
        )}>
          {((isRelatedLoading || isPathLoading) && !isManualSelection) ? (
            <span className="flex items-center gap-2">
              <div className="h-3 w-3 animate-spin rounded-full border border-muted-foreground border-t-transparent" />
              <span className="text-muted-foreground">正在加载地址...</span>
            </span>
          ) : (
            displayText || placeholder
          )}
        </span>
        {currentValue && (
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className={cn("h-4 w-4 p-0 hover:bg-transparent")}
            onClick={(e) => {
              e.stopPropagation()
              handleClear()
            }}
          >
            <X className={cn("h-3 w-3")} />
          </Button>
        )}
      </div>
    </div>
  )

  if (isMobile) {
    return (
      <div className={cn("relative", className)}>
        <div onClick={() => !disabled && handleOpenChange(true)}>
          {trigger}
        </div>
        <Drawer open={open} onOpenChange={handleOpenChange}>
          <DrawerContent 
            className="w-[95%] sm:max-w-md"
            contentClassName="p-0 flex flex-col"
          >
            <DrawerHeader className="px-4 py-3 border-b">
              <DrawerTitle>选择地址</DrawerTitle>
            </DrawerHeader>
            <div className="flex-1 overflow-auto px-4 py-2">
              {renderContent()}
            </div>
            {(enableSearch || enableGeolocation) && renderToolbar()}
          </DrawerContent>
        </Drawer>
      </div>
    )
  }

  return (
    <div className={cn("relative", className)}>
      <Popover open={open} onOpenChange={handleOpenChange}>
        <PopoverTrigger asChild>
          {trigger}
        </PopoverTrigger>

        <PopoverContent
          className={cn("p-0")}
          style={{ width: popoverWidth }}
          align="start"
        >
          <div className="flex flex-col">
            {(enableSearch || enableGeolocation) && (
              <div className="flex items-center border-b p-2 gap-2">
                {enableSearch && (
                  <>
                    <Search className={cn("h-4 w-4 text-muted-foreground")} />
                    <Input
                      ref={inputRef}
                      placeholder="搜索地址..."
                      value={searchKeyword}
                      onChange={(e) => setSearchKeyword(e.target.value)}
                      className={cn("border-0 shadow-none focus-visible:ring-0")}
                    />
                  </>
                )}
                {enableGeolocation && (
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={handleGetLocation}
                    disabled={isGeoLoading}
                    className={cn("flex-shrink-0")}
                  >
                    {isGeoLoading ? (
                      <div className="h-4 w-4 animate-spin rounded-full border border-muted-foreground border-t-transparent" />
                    ) : (
                      <Navigation className={cn("h-4 w-4")} />
                    )}
                    <span className="ml-1 hidden sm:inline">
                      {isGeoLoading ? '定位中...' : '获取当前位置'}
                    </span>
                  </Button>
                )}
              </div>
            )}

            {renderContent()}
          </div>
        </PopoverContent>
      </Popover>
    </div>
  )
}
