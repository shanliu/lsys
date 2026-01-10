import {
  roleUserData,
  roleUserAdd,
  roleUserDelete,
  roleAvailableUser,
  type RoleItemType,
  type UserItemType,
  type RoleUserDataItemType,
} from '@shared/apis/admin/rbac-role'
import { ConfirmDialog } from '@shared/components/custom/dialog/confirm-dialog'
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
} from '@apps/main/components/local/drawer'
import {
  DEFAULT_PAGE_SIZE,
  PagePagination,
  useCountNumManager,
} from '@apps/main/lib/pagination-utils'
import { Badge } from '@shared/components/ui/badge'
import { Button } from '@shared/components/ui/button'
import { Card, CardContent } from '@shared/components/ui/card'
import { useIsMobile } from '@shared/hooks/use-mobile'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@shared/components/ui/command'
import { TimeoutInput } from '@shared/components/custom/input/timeout-input'
import { Label } from '@shared/components/ui/label'
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@shared/components/ui/popover'
import { useToast } from '@shared/contexts/toast-context'
import { formatServerError, getQueryResponseData, formatTime, TIME_STYLE, formatSeconds } from '@shared/lib/utils'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import { CenteredError } from '@shared/components/custom/page-placeholder/centered-error'
import { CenteredLoading } from '@shared/components/custom/page-placeholder/centered-loading'
import { Plus, Trash2, Users } from 'lucide-react'
import { Loader2 } from 'lucide-react'
import React, { useState, useDeferredValue } from 'react'


// 移动端/PC端自适应列表视图组件
interface RoleUsersListViewProps {
  roleUsers: RoleUserDataItemType[]
  onDelete: (userId: number, userName: string) => Promise<void>
}

function RoleUsersListView({ roleUsers, onDelete }: RoleUsersListViewProps) {
  const isMobile = useIsMobile()

  return (
    <div className={isMobile ? "space-y-3" : "grid grid-cols-2 gap-3"}>
      {roleUsers.map((item) => (
        <Card key={item.id} className="p-0">
          <CardContent className={isMobile ? "p-4 space-y-3" : "p-3 space-y-2"}>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <span className="text-xs text-muted-foreground">ID:</span>
                <span className="text-sm font-mono">{item.user_data.id}</span>
              </div>
              <ConfirmDialog
                title="移除用户"
                description={`确定要从角色中移除用户「${item.user_data.user_nickname || item.user_data.user_account}」吗？`}
                onConfirm={async () => {
                  await onDelete(item.user_id, item.user_data.user_nickname || item.user_data.user_account)
                }}
              >
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-7 px-2"
                >
                  <Trash2 className="h-4 w-4" />
                </Button>
              </ConfirmDialog>
            </div>
            <div className="flex items-start gap-3">
              <div className="text-xs font-medium text-muted-foreground whitespace-nowrap min-w-[50px] shrink-0 pt-0.5">
                用户名
              </div>
              <div className="text-sm">
                <Badge variant="outline">{item.user_data.user_account}</Badge>
                <span className="text-xs text-muted-foreground ml-1">
                  #{item.user_data.id}
                </span>
              </div>
            </div>
            <div className="flex items-start gap-3">
              <div className="text-xs font-medium text-muted-foreground whitespace-nowrap min-w-[50px] shrink-0 pt-0.5">
                昵称
              </div>
              <div className="text-sm">{item.user_data.user_nickname || '-'}</div>
            </div>
            <div className="flex items-start gap-3">
              <div className="text-xs font-medium text-muted-foreground whitespace-nowrap min-w-[50px] shrink-0 pt-0.5">
                有效期
              </div>
              <div className="text-sm">{formatSeconds(item.timeout || 0)}</div>
            </div>
            <div className="flex items-start gap-3">
              <div className="text-xs font-medium text-muted-foreground whitespace-nowrap min-w-[50px] shrink-0 pt-0.5">
                更改于
              </div>
              <div className="text-sm text-muted-foreground">
                {formatTime(item.change_time || null, TIME_STYLE.RELATIVE_ELEMENT)}
              </div>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  )
}

interface RoleUsersDrawerProps {
  /** 角色数据 */
  role: RoleItemType
  /** 是否打开 */
  open: boolean
  /** 打开状态变化回调 */
  onOpenChange: (open: boolean) => void
}

export function RoleUsersDrawer({
  role,
  open,
  onOpenChange,
}: RoleUsersDrawerProps) {
  const toast = useToast()
  const queryClient = useQueryClient()
  const [addUserOpen, setAddUserOpen] = React.useState(false)
  const [searchText, setSearchText] = React.useState('')
  const [timeout, setTimeout] = React.useState(0)
  const [selectedUserId, setSelectedUserId] = React.useState<number | null>(null)
  const [selectedUser, setSelectedUser] = React.useState<UserItemType | null>(null)

  // 分页状态
  const [pagination, setPagination] = useState({
    page: 1,
    limit: DEFAULT_PAGE_SIZE,
  })

  // count_num 优化管理器
  const countNumManager = useCountNumManager()

  // 获取角色用户列表
  const { data: usersData, isSuccess, isLoading, isError, error, refetch } = useQuery({
    queryKey: ['admin-rbac-role-users', role.id, pagination.page, pagination.limit],
    queryFn: async ({ signal }) => {
      const result = await roleUserData(
        {
          role_id: role.id,
          all: false,
          page: {
            page: pagination.page,
            limit: pagination.limit,
          },
          count_num: countNumManager.getCountNum(),
        },
        { signal }
      )
      return result
    },
    placeholderData: (previousData) => previousData,
    enabled: open,
  })

  // 处理分页查询结果
  if (isSuccess) {
    countNumManager.handlePageQueryResult(usersData)
  }

  // 使用 deferred value 减少请求频率
  const debouncedSearchText = useDeferredValue(searchText)

  // 获取可用用户列表
  const { data: availableUsersData, isLoading: availableLoading } = useQuery({
    queryKey: ['admin-rbac-available-users', role.id, debouncedSearchText],
    queryFn: async ({ signal }) => {
      const result = await roleAvailableUser(
        {
          user_data: debouncedSearchText || undefined,
          limit: {
            limit: 20,
            forward: true
          },
          count_num: false,
        },
        { signal }
      )
      return result
    },
    enabled: open,
  })

  const roleUsers = getQueryResponseData<RoleUserDataItemType[]>(usersData, [])
  const availableUsers = getQueryResponseData<UserItemType[]>(availableUsersData, [])

  // 添加用户
  const addUserMutation = useMutation({
    mutationFn: (userId: number) =>
      roleUserAdd({
        role_id: role.id,
        user_data: [
          {
            user_id: userId,
            timeout: timeout,
          },
        ],
      }),
    onSuccess: () => {
      toast.success('用户添加成功')
      countNumManager.reset()
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-role-users', role.id] })
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-available-users', role.id] })
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-role-list'], refetchType: 'all' })
      setAddUserOpen(false)
      setSelectedUserId(null)
      setSelectedUser(null)
      setTimeout(0)
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  // 删除用户
  const deleteUserMutation = useMutation({
    mutationFn: (userId: number) =>
      roleUserDelete({
        role_id: role.id,
        user_data: [userId],
      }),
    onSuccess: () => {
      toast.success('用户移除成功')
      countNumManager.reset()
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-role-users', role.id] })
      queryClient.invalidateQueries({ queryKey: ['admin-rbac-role-list'], refetchType: 'all' })
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  const handleAddUser = () => {
    if (!selectedUserId) {
      toast.error('请选择用户')
      return
    }
    addUserMutation.mutate(selectedUserId)
  }

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent className="w-[95%] md:w-[680px]">
        <DrawerHeader>
          <DrawerTitle className="flex items-center gap-2">
            <Users className="h-5 w-5" />
            角色用户管理
          </DrawerTitle>
          <DrawerDescription>
            管理角色「{role.role_name}」的关联用户
          </DrawerDescription>
        </DrawerHeader>

        <div className="mt-6 space-y-6">
          {/* 添加用户区域 */}
          <div className="space-y-3 p-4 border rounded-lg bg-muted/50">
            <h4 className="font-medium">添加用户</h4>
            <div className="flex flex-wrap items-end gap-3">
              <div className="flex-1 min-w-[200px]">
                <Label className="text-sm text-muted-foreground mb-1.5 block">选择用户</Label>
                <Popover open={addUserOpen} onOpenChange={setAddUserOpen}>
                  <PopoverTrigger asChild>
                    <Button
                      variant="outline"
                      role="combobox"
                      aria-expanded={addUserOpen}
                      className="w-full justify-start"
                    >
                      {selectedUser
                        ? selectedUser.user_nickname || selectedUser.user_account
                        : '搜索并选择用户...'}
                    </Button>
                  </PopoverTrigger>
                  <PopoverContent className="w-[300px] p-0" align="start">
                    <Command shouldFilter={false}>
                      <CommandInput
                        placeholder="搜索用户..."
                        value={searchText}
                        onValueChange={setSearchText}
                      />
                      <CommandList>
                        <CommandEmpty>
                          {availableLoading ? '加载中...' : '未找到用户'}
                        </CommandEmpty>
                        <CommandGroup>
                          {availableUsers.map((user) => (
                            <CommandItem
                              key={user.id}
                              value={`${user.user_account} ${user.user_nickname}`}
                              onSelect={() => {
                                setSelectedUserId(user.id)
                                setSelectedUser(user)
                                setAddUserOpen(false)
                              }}
                            >
                              <div className="flex flex-col">
                                <span className="font-medium">{user.user_nickname || user.user_account}</span>
                                <span className="text-xs text-muted-foreground">
                                  {user.user_account}
                                </span>
                              </div>
                            </CommandItem>
                          ))}
                        </CommandGroup>
                      </CommandList>
                    </Command>
                  </PopoverContent>
                </Popover>
              </div>

              <div className="w-full sm:w-[280px]">
                <Label className="text-sm text-muted-foreground mb-1.5 block">超时时间</Label>
                <TimeoutInput
                  value={timeout}
                  onChange={setTimeout}
                />
              </div>

              <Button
                onClick={handleAddUser}
                disabled={!selectedUserId || addUserMutation.isPending}
                className="w-full sm:w-auto shrink-0"
              >
                {addUserMutation.isPending ? (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                ) : (
                  <Plus className="mr-2 h-4 w-4" />
                )}
                添加
              </Button>
            </div>
          </div>

          {/* 用户列表 */}
          <div className="space-y-3">
            <h4 className="font-medium">已关联用户 ({countNumManager.getTotal() ?? roleUsers.length})</h4>
            {isLoading ? (
              <CenteredLoading className="py-8" />
            ) : isError ? (
              <CenteredError error={error} onReset={() => refetch()} className="py-8" />
            ) : roleUsers.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                暂无关联用户
              </div>
            ) : (
              <RoleUsersListView
                roleUsers={roleUsers}
                onDelete={async (userId, userName) => {
                  await deleteUserMutation.mutateAsync(userId)
                }}
              />
            )}

            {/* 分页 */}
            {(countNumManager.getTotal() ?? 0) > 0 && (
              <PagePagination
                currentPage={pagination.page}
                pageSize={pagination.limit}
                total={countNumManager.getTotal() ?? 0}
                loading={isLoading}
                onChange={(page: number) => {
                  setPagination((prev) => ({ ...prev, page }))
                }}
                onPageSizeChange={(limit: number) => {
                  setPagination({ page: 1, limit })
                }}
              />
            )}
          </div>
        </div>
      </DrawerContent>
    </Drawer>
  )
}
