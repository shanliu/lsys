import {
  resourceTypeOpData,
  resourceTypeOpAdd,
  resourceTypeOpDel,
  type ResourceOpItemType,
  type StaticResTplDataItemType,
} from '@shared/apis/admin/rbac-res'
import {
  opList,
  opAdd,
  type OpItemType,
  type OpAddParamType,
} from '@shared/apis/admin/rbac-op'
import { ConfirmDialog } from '@shared/components/custom/dialog/confirm-dialog'
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
} from '@apps/main/components/local/drawer'
import { Button } from '@shared/components/ui/button'
import { Checkbox } from '@shared/components/ui/checkbox'
import { Card, CardContent, CardHeader, CardTitle } from '@shared/components/ui/card'
import { Badge } from '@shared/components/ui/badge'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@shared/components/ui/select'
import { AutocompleteInput } from '@shared/components/custom/input/autocomplete-input'
import { Label } from '@shared/components/ui/label'
import { PagePagination, DEFAULT_PAGE_SIZE, useCountNumManager } from '@apps/main/lib/pagination-utils'
import { useToast } from '@shared/contexts/toast-context'
import { cn, formatServerError, getQueryResponseData } from '@shared/lib/utils'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { CenteredLoading } from '@shared/components/custom/page-placeholder/centered-loading'
import { useIsMobile } from '@shared/hooks/use-mobile'
import { Loader2, Plus, Settings, Trash2, Search, X, FileEdit } from 'lucide-react'
import React from 'react'

interface ResTypeOpsDrawerProps {
  /** 资源类型 */
  resType: string
  /** 是否打开 */
  open: boolean
  /** 打开状态变化回调 */
  onOpenChange: (open: boolean) => void
  /** 静态资源模板数据 */
  staticResources?: StaticResTplDataItemType[]
}

export function ResTypeOpsDrawer({
  resType,
  open,
  onOpenChange,
  staticResources = [],
}: ResTypeOpsDrawerProps) {
  const toast = useToast()
  const queryClient = useQueryClient()
  const isMobile = useIsMobile()
  const [selectedOpIds, setSelectedOpIds] = React.useState<number[]>([])

  // 根据当前资源类型从静态资源中获取推荐的操作
  const recommendedOps = React.useMemo(() => {
    // 从静态资源模板数据中查找匹配当前资源类型的操作数据
    const matchedTemplate = staticResources.find(tpl => tpl.res_type === resType)
    
    const opsMap = new Map<string, string>()
    
    if (matchedTemplate && matchedTemplate.op_data) {
      // 使用模板中的操作数据
      matchedTemplate.op_data.forEach(op => {
        opsMap.set(op.key, op.name)
      })
    }
    
    return {
      keys: Array.from(opsMap.keys()),
      names: Array.from(opsMap.values()),
      keyToName: opsMap,
      nameToKey: new Map(Array.from(opsMap.entries()).map(([k, v]) => [v, k])),
    }
  }, [resType, staticResources])

  // 搜索和创建状态
  const [searchMode, setSearchMode] = React.useState(false)
  const [createMode, setCreateMode] = React.useState(false)
  const [searchKeyword, setSearchKeyword] = React.useState('')
  const [searchType, setSearchType] = React.useState<'key' | 'name'>('key')

  // 新操作表单状态
  const [newOpKey, setNewOpKey] = React.useState('')
  const [newOpName, setNewOpName] = React.useState('')
  const [shouldCheckOpKey, setShouldCheckOpKey] = React.useState(false)
  const [shouldAssociateAfterCreate, setShouldAssociateAfterCreate] = React.useState(true)

  // 分页状态
  const [page, setPage] = React.useState(1)
  const [limit, setLimit] = React.useState(DEFAULT_PAGE_SIZE)

  // 计数管理器
  const countNumManager = useCountNumManager({ resType })

  // 使用 useQuery 检查操作标识是否已存在
  const { data: opKeyCheckData } = useQuery({
    queryKey: ['admin-rbac-op-check-exists', newOpKey],
    queryFn: async ({ signal }) => {
      const result = await opList(
        {
          page: { page: 1, limit: 1 },
          op_key: newOpKey,
          count_num: false,
        },
        { signal }
      )
      return result
    },
    enabled: shouldCheckOpKey && !!newOpKey.trim(),
    staleTime: 30 * 1000, // 30秒缓存
  })

  // 根据查询结果判断操作标识是否存在
  const opKeyExists = React.useMemo(() => {
    if (!shouldCheckOpKey || !newOpKey.trim()) return false
    const items = getQueryResponseData<OpItemType[]>(opKeyCheckData, [])
    return items.length > 0
  }, [opKeyCheckData, shouldCheckOpKey, newOpKey])

  // 获取资源类型已关联的操作列表（分页）
  const {
    data: resOpsData,
    isLoading: resOpsLoading,
    isSuccess: resOpsSuccess,
    isError: isResOpsError,
    error: resOpsError,
    refetch: refetchResOps,
  } = useQuery({
    queryKey: ['admin-rbac-res-type-ops', resType, page, limit],
    queryFn: async ({ signal }) => {
      const result = await resourceTypeOpData(
        {
          res_type: resType,
          page: { page, limit },
          count_num: countNumManager.getCountNum(),
        },
        { signal }
      )
      return result
    },
    enabled: open && !!resType,
  })

  // 处理分页计数
  if (resOpsSuccess) {
    countNumManager.handlePageQueryResult(resOpsData)
  }

  // 获取所有已关联的操作ID（用于过滤搜索结果）- 不分页，取前1000个
  const { data: allAssociatedIdsData } = useQuery({
    queryKey: ['admin-rbac-res-type-ops-ids', resType],
    queryFn: async ({ signal }) => {
      const result = await resourceTypeOpData(
        {
          res_type: resType,
          page: { page: 1, limit: 1000 },
          count_num: false,
        },
        { signal }
      )
      return result
    },
    enabled: open && !!resType,
  })

  // 搜索可用的操作列表（仅在搜索模式下查询）
  const {
    data: searchOpsData,
    isLoading: searchOpsLoading,
    isError: isSearchOpsError,
    error: searchOpsError,
    refetch: refetchSearchOps,
  } = useQuery({
    queryKey: ['admin-rbac-op-search', searchKeyword, searchType],
    queryFn: async ({ signal }) => {
      const result = await opList(
        {
          page: { page: 1, limit: 50 }, // 只展示前50个匹配项
          op_name: searchType === 'name' ? (searchKeyword || undefined) : undefined,
          op_key: searchType === 'key' ? (searchKeyword || undefined) : undefined,
          count_num: false,
        },
        { signal }
      )
      return result
    },
    enabled: open && searchMode && searchKeyword.length > 0,
  })

  // 创建新操作
  const createOpMutation = useMutation({
    mutationFn: (param: OpAddParamType & { shouldAssociate: boolean }) => opAdd({
      op_key: param.op_key,
      op_name: param.op_name,
    }),
    onSuccess: (result, variables) => {
      // API 返回格式: { response: { id: "33" }, result: {...} }
      // 注意: id 是字符串类型，需要转换为数字
      const newOpId = result?.response?.id ? Number(result.response.id) : 0
      toast.success('操作创建成功')
      
      // 重置创建表单
      setNewOpKey('')
      setNewOpName('')
      setShouldCheckOpKey(false)
      setCreateMode(false)
      
      // 如果选择了创建后立即关联，则自动关联到当前资源类型
      if (variables.shouldAssociate && newOpId) {
        addOpMutation.mutate([newOpId])
      }
      
      // 刷新操作列表
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-op-list'] })
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-op-search'] })
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-op-check-exists'] })
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  // 添加操作关联
  const addOpMutation = useMutation({
    mutationFn: (opIds?: number[]) =>
      resourceTypeOpAdd({
        res_type: resType,
        op_ids: opIds || selectedOpIds,
      }),
    onSuccess: () => {
      toast.success('操作关联成功')
      countNumManager.reset()
      queryClient.invalidateQueries({
        queryKey: ['admin-rbac-res-type-ops', resType],
      })
      queryClient.invalidateQueries({
        queryKey: ['admin-rbac-res-type-ops-ids', resType],
      })
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-op-list'] })
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-res-list'], refetchType: 'all' })
      setSelectedOpIds([])
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  // 删除操作关联
  const deleteOpMutation = useMutation({
    mutationFn: (opId: number) =>
      resourceTypeOpDel({
        res_type: resType,
        op_ids: [opId],
      }),
    onSuccess: () => {
      toast.success('操作移除成功')
      countNumManager.reset()
      queryClient.invalidateQueries({
        queryKey: ['admin-rbac-res-type-ops', resType],
      })
      queryClient.invalidateQueries({
        queryKey: ['admin-rbac-res-type-ops-ids', resType],
      })
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-op-list'] })
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-op-list-all'] })
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-res-list'], refetchType: 'all' })
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  const handleToggleOp = (opId: number) => {
    setSelectedOpIds((prev) =>
      prev.includes(opId) ? prev.filter((id) => id !== opId) : [...prev, opId]
    )
  }

  const resOps = getQueryResponseData<ResourceOpItemType[]>(resOpsData, [])
  const searchOps = getQueryResponseData<OpItemType[]>(searchOpsData, [])
  const allAssociatedOps = getQueryResponseData<ResourceOpItemType[]>(allAssociatedIdsData, [])

  // 过滤出未关联的操作
  const associatedOpIds = new Set(allAssociatedOps.map((op) => op.op_data.id))
  const availableOps = searchOps.filter((op) => !associatedOpIds.has(op.id))

  // 重置所有搜索和创建状态
  const resetModes = () => {
    setSearchMode(false)
    setCreateMode(false)
    setSearchKeyword('')
    setSelectedOpIds([])
    setNewOpKey('')
    setNewOpName('')
    setShouldCheckOpKey(false)
    setShouldAssociateAfterCreate(true)
  }

  // 当抽屉关闭时重置状态
  React.useEffect(() => {
    if (!open) {
      resetModes()
      setPage(1)
    }
  }, [open])

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent
        className={cn('md:w-[700px]')}
        showCloseButton={false}
        contentClassName="p-4 md:p-6"
      >
        <DrawerHeader>
          <DrawerTitle className="flex items-center justify-center sm:justify-start gap-2">
            <Settings className="h-5 w-5" />
            资源类型操作管理
          </DrawerTitle>
          <DrawerDescription>
            管理资源类型「{resType}」的操作关联
          </DrawerDescription>
        </DrawerHeader>

        <div className="mt-6 space-y-4">
          {/* 操作按钮组 */}
          <div className="flex flex-wrap gap-2">
            <Button
              variant={searchMode ? 'default' : 'outline'}
              size="sm"
              onClick={() => {
                if (searchMode) {
                  resetModes()
                } else {
                  setCreateMode(false)
                  setSearchMode(true)
                }
              }}
            >
              {searchMode ? <X className="h-4 w-4 mr-2" /> : <Search className="h-4 w-4 mr-2" />}
              {searchMode ? '取消搜索' : '搜索关联操作'}
            </Button>
            
            <Button
              variant={createMode ? 'default' : 'outline'}
              size="sm"
              onClick={() => {
                if (createMode) {
                  resetModes()
                } else {
                  setSearchMode(false)
                  setCreateMode(true)
                }
              }}
            >
              {createMode ? <X className="h-4 w-4 mr-2" /> : <Plus className="h-4 w-4 mr-2" />}
              {createMode ? '取消创建' : '创建新操作'}
            </Button>
          </div>

          {/* 创建新操作表单 */}
          {createMode && (
            <Card className="border-primary/50">
              <CardHeader className="pb-3">
                <CardTitle className="text-base flex items-center gap-2">
                  <FileEdit className="h-4 w-4" />
                  创建新操作
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="space-y-2">
                  <Label htmlFor="new-op-key">操作标识 *</Label>
                  <AutocompleteInput
                    id="new-op-key"
                    placeholder="例如: create, update, delete"
                    value={newOpKey}
                    onChange={(value) => {
                      setNewOpKey(value)
                      // 重置存在性检查状态
                      setShouldCheckOpKey(false)
                      // 当操作标识选择后，自动填充对应的操作名称
                      if (recommendedOps.keyToName.has(value)) {
                        setNewOpName(recommendedOps.keyToName.get(value)!)
                      }
                    }}
                    onBlur={() => setShouldCheckOpKey(true)}
                    options={recommendedOps.keys}
                    filterOnInput={false}
                    className={cn("h-9", opKeyExists && "border-destructive focus-visible:ring-destructive")}
                  />
                  {opKeyExists && (
                    <p className="text-sm text-destructive flex items-center gap-1">
                      <span className="inline-block w-1 h-1 rounded-full bg-destructive"></span>
                      该操作标识已存在，请使用其他标识或直接关联已有操作
                    </p>
                  )}
                </div>
                <div className="space-y-2">
                  <Label htmlFor="new-op-name">操作名称 *</Label>
                  <AutocompleteInput
                    id="new-op-name"
                    placeholder="例如: 创建, 更新, 删除"
                    value={newOpName}
                    filterOnInput={false}
                    onChange={(value) => {
                      setNewOpName(value)
                      // 当操作名称选择后，自动填充对应的操作标识
                      if (recommendedOps.nameToKey.has(value)) {
                        setNewOpKey(recommendedOps.nameToKey.get(value)!)
                      }
                    }}
                    options={recommendedOps.names}
                
                    className="h-9"
                  />
                </div>
                <div className="flex items-center gap-2">
                  <Checkbox
                    id="should-associate"
                    checked={shouldAssociateAfterCreate}
                    onCheckedChange={(checked) => setShouldAssociateAfterCreate(checked === true)}
                  />
                  <label
                    htmlFor="should-associate"
                    className="text-sm cursor-pointer select-none"
                  >
                    创建后立即关联到此资源类型
                  </label>
                </div>
                <Button
                  onClick={() => {
                    if (!newOpKey.trim() || !newOpName.trim()) {
                      toast.error('请填写完整信息')
                      return
                    }
                    if (opKeyExists) {
                      toast.error('操作标识已存在，请使用其他标识')
                      return
                    }
                    createOpMutation.mutate({
                      op_key: newOpKey.trim(),
                      op_name: newOpName.trim(),
                      shouldAssociate: shouldAssociateAfterCreate,
                    })
                  }}
                  disabled={createOpMutation.isPending || !newOpKey.trim() || !newOpName.trim() || opKeyExists}
                  size="sm"
                  className="w-full"
                >
                  {createOpMutation.isPending && (
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  )}
                  <Plus className="mr-2 h-4 w-4" />
                  {shouldAssociateAfterCreate ? '创建并关联到此资源类型' : '仅创建操作'}
                </Button>
              </CardContent>
            </Card>
          )}

          {/* 搜索关联操作区域 */}
          {searchMode && (
            <Card className="border-primary/50">
              <CardHeader className="pb-3">
                <CardTitle className="text-base flex items-center gap-2">
                  <Search className="h-4 w-4" />
                  搜索并关联操作
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="flex gap-2">
                  <Select
                    value={searchType}
                    onValueChange={(v: 'key' | 'name') => setSearchType(v)}
                  >
                    <SelectTrigger className="w-[110px] h-9">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="key">操作标识</SelectItem>
                      <SelectItem value="name">操作名称</SelectItem>
                    </SelectContent>
                  </Select>
                  <AutocompleteInput
                    placeholder={searchType === 'key' ? "输入操作标识..." : "输入操作名称..."}
                    value={searchKeyword}
                    onChange={setSearchKeyword}
                    options={searchType === 'key' ? recommendedOps.keys : recommendedOps.names}
                    filterOnInput={false}
                    className="h-9 flex-1"

                  />
                </div>
                
                {searchKeyword && (
                  <div className="space-y-2 max-h-[240px] overflow-y-auto">
                    {searchOpsLoading ? (
                      <CenteredLoading className="h-[100px]" iconSize="md" />
                    ) : isSearchOpsError ? (
                      <CenteredError
                        error={searchOpsError}
                        variant="content"
                        className="h-[100px]"
                        onReset={refetchSearchOps}
                      />
                    ) : availableOps.length > 0 ? (
                      <>
                        {availableOps.map((op) => (
                          <div
                            key={op.id}
                            className={cn(
                              "flex items-center gap-3 p-3 rounded-lg border transition-colors",
                              selectedOpIds.includes(op.id)
                                ? "bg-primary/10 border-primary"
                                : "hover:bg-muted/50"
                            )}
                          >
                            <Checkbox
                              id={`op-${op.id}`}
                              checked={selectedOpIds.includes(op.id)}
                              onCheckedChange={() => handleToggleOp(op.id)}
                            />
                            <label
                              htmlFor={`op-${op.id}`}
                              className="flex-1 cursor-pointer"
                            >
                              <div className="font-medium text-sm">
                                {op.op_name || op.op_key}
                              </div>
                              <div className="text-xs text-muted-foreground mt-0.5">
                                标识: <code className="bg-muted px-1 py-0.5 rounded">{op.op_key}</code>
                              </div>
                            </label>
                          </div>
                        ))}
                        <Button
                          onClick={() => addOpMutation.mutate(undefined)}
                          disabled={selectedOpIds.length === 0 || addOpMutation.isPending}
                          size="sm"
                          className="w-full"
                        >
                          {addOpMutation.isPending && (
                            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                          )}
                          <Plus className="mr-2 h-4 w-4" />
                          关联选中操作 ({selectedOpIds.length})
                        </Button>
                      </>
                    ) : searchOps.length > 0 ? (
                      // 搜索到了结果但都已关联
                      <div className="text-center py-6 space-y-3">
                        <div className="text-muted-foreground text-sm">
                          搜索到 {searchOps.length} 个操作，但均已关联到此资源类型
                        </div>
                      </div>
                    ) : (
                      // 真的没有搜索到结果
                      <div className="text-center py-6 space-y-3">
                        <div className="text-muted-foreground text-sm">
                          没有找到匹配的操作
                        </div>
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => {
                            // 切换到创建模式，并根据搜索类型自动填充
                            setSearchMode(false)
                            setCreateMode(true)
                            if (searchType === 'key') {
                              setNewOpKey(searchKeyword)
                              // 如果推荐操作中有对应的名称，自动填充
                              if (recommendedOps.keyToName.has(searchKeyword)) {
                                setNewOpName(recommendedOps.keyToName.get(searchKeyword)!)
                              }
                            } else {
                              setNewOpName(searchKeyword)
                              // 如果推荐操作中有对应的标识，自动填充
                              if (recommendedOps.nameToKey.has(searchKeyword)) {
                                setNewOpKey(recommendedOps.nameToKey.get(searchKeyword)!)
                              }
                            }
                            setSearchKeyword('')
                          }}
                        >
                          <Plus className="mr-2 h-4 w-4" />
                          创建「{searchKeyword}」操作
                        </Button>
                      </div>
                    )}
                  </div>
                )}
                
                {!searchKeyword && (
                  <div className="text-center py-6 text-muted-foreground text-sm">
                    输入关键词搜索操作
                  </div>
                )}
              </CardContent>
            </Card>
          )}

          {/* 已关联操作列表 */}
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <h4 className="font-medium text-sm">
                已关联操作
                {resOps.length > 0 && (
                  <Badge variant="secondary" className="ml-2">
                    {countNumManager.getTotal() ?? resOps.length}
                  </Badge>
                )}
              </h4>
            </div>
            
            {resOpsLoading ? (
              <CenteredLoading className="h-[200px]" />
            ) : isResOpsError ? (
              <CenteredError
                error={resOpsError}
                variant="content"
                className="h-[200px]"
                onReset={refetchResOps}
              />
            ) : resOps.length === 0 ? (
              <Card>
                <CardContent className="text-center py-12 text-muted-foreground">
                  暂无关联操作
                </CardContent>
              </Card>
            ) : (
              <div className="space-y-2">
                {resOps.map((op) => (
                  <Card key={op.op_res.id} className="transition-shadow hover:shadow-md">
                    <CardContent className="p-4">
                      <div className="flex items-start justify-between gap-3">
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2 flex-wrap">
                            <span className="font-medium text-sm">
                              {op.op_data.op_name || op.op_data.op_key}
                            </span>
                            <Badge variant="outline" className="text-xs font-mono">
                              {op.op_data.op_key}
                            </Badge>
                            {recommendedOps.keys.includes(op.op_data.op_key) && (
                              <Badge variant="secondary" className="text-xs">
                                系统
                              </Badge>
                            )}
                          </div>
                          {!isMobile && (
                            <div className="text-xs text-muted-foreground mt-1">
                              ID: {op.op_data.id}
                            </div>
                          )}
                        </div>
                        <ConfirmDialog
                          title="移除操作关联"
                          description={`确定要从资源类型中移除操作「${op.op_data.op_name}」吗？`}
                          onConfirm={async () => {
                            await deleteOpMutation.mutateAsync(op.op_data.id)
                          }}
                        >
                          <Button
                            variant="ghost"
                            size="sm"
                            className="h-8 px-2 text-destructive hover:text-destructive hover:bg-destructive/10"
                          >
                            <Trash2 className="h-4 w-4" />
                            {!isMobile && <span className="ml-1">移除</span>}
                          </Button>
                        </ConfirmDialog>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            )}

            {((countNumManager.getTotal() ?? 0) > 0) && (
              <PagePagination
                currentPage={page}
                pageSize={limit}
                total={countNumManager.getTotal() ?? 0}
                loading={resOpsLoading}
                onChange={setPage}
                onPageSizeChange={(l) => {
                  setPage(1)
                  setLimit(l)
                }}
                className="py-2"
              />
            )}
          </div>
        </div>
      </DrawerContent>
    </Drawer>
  )
}
