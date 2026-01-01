import {
  appRbacRoleAvailableUser,
  appRbacRoleUserAdd,
  appRbacRoleUserData,
  appRbacRoleUserDelete,
  type AppRbacRoleAvailableUserDataItemType,
  type AppRbacRoleDataItemType,
  type AppRbacRoleUserDataResItemType,
} from '@shared/apis/user/rbac'
import { ConfirmDialog } from '@shared/components/custom/dialog/confirm-dialog'
import { Badge } from '@shared/components/ui/badge'
import { Button } from '@shared/components/ui/button'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@shared/components/ui/command'
import { Input } from '@shared/components/ui/input'
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@shared/components/ui/popover'
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
} from '@apps/main/components/local/drawer'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@shared/components/ui/table'
import { useToast } from '@shared/contexts/toast-context'
import { cn, formatServerError, getQueryResponseData } from '@shared/lib/utils'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Loader2, Plus, Trash2, Users } from 'lucide-react'
import React from 'react'

interface RoleUsersDrawerProps {
  /** 角色数据 */
  role: AppRbacRoleDataItemType
  /** 是否打开 */
  open: boolean
  /** 打开状态变化回调 */
  onOpenChange: (open: boolean) => void
  /** 应用ID */
  appId: number
}

export function RoleUsersDrawer({
  role,
  open,
  onOpenChange,
  appId,
}: RoleUsersDrawerProps) {
  const toast = useToast()
  const queryClient = useQueryClient()
  const [addUserOpen, setAddUserOpen] = React.useState(false)
  const [searchText, setSearchText] = React.useState('')
  const [customUserParam, setCustomUserParam] = React.useState('')

  // 获取角色用户列表
  const { data: usersData, isLoading } = useQuery({
    queryKey: ['rbac-role-users', role.id],
    queryFn: async ({ signal }) => {
      const result = await appRbacRoleUserData(
        {
          role_id: role.id,
          page: { page: 1, limit: 100 },
          count_num: false,
        },
        { signal }
      )
      return result
    },
    enabled: open,
  })

  // 获取可用用户列表
  const { data: availableUsersData, isLoading: availableLoading } = useQuery({
    queryKey: ['rbac-available-users', appId, searchText],
    queryFn: async ({ signal }) => {
      const result = await appRbacRoleAvailableUser(
        {
          app_id: appId,
          user_any: searchText || null,
          limit: { limit: 20 },
          count_num: false,
        },
        { signal }
      )
      return result
    },
    enabled: open && addUserOpen,
  })

  const users = getQueryResponseData<AppRbacRoleUserDataResItemType[]>(usersData, [])
  const availableUsers = getQueryResponseData<AppRbacRoleAvailableUserDataItemType[]>(
    availableUsersData,
    []
  )

  // 添加用户
  const addUserMutation = useMutation({
    mutationFn: (userData: { use_app_user: boolean; user_param: string }) =>
      appRbacRoleUserAdd({
        role_id: role.id,
        user_data: [
          {
            use_app_user: userData.use_app_user,
            user_param: userData.user_param,
            timeout: 0,
          },
        ],
      }),
    onSuccess: () => {
      toast.success('用户添加成功')
      queryClient.invalidateQueries({ queryKey: ['rbac-role-users', role.id] })
      setAddUserOpen(false)
      setCustomUserParam('')
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  // 删除用户
  const deleteUserMutation = useMutation({
    mutationFn: (user: AppRbacRoleUserDataResItemType) =>
      appRbacRoleUserDelete({
        role_id: role.id,
        use_app_user: user.use_app_user,
        user_data: [user.user_param],
      }),
    onSuccess: () => {
      toast.success('用户移除成功')
      queryClient.invalidateQueries({ queryKey: ['rbac-role-users', role.id] })
    },
    onError: (error: any) => {
      toast.error(formatServerError(error))
    },
  })

  const handleAddAppUser = (user: AppRbacRoleAvailableUserDataItemType) => {
    addUserMutation.mutate({
      use_app_user: true,
      user_param: String(user.id),
    })
  }

  const handleAddCustomUser = () => {
    if (!customUserParam.trim()) {
      toast.error('请输入用户参数')
      return
    }
    addUserMutation.mutate({
      use_app_user: false,
      user_param: customUserParam.trim(),
    })
  }

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent>
        <DrawerHeader>
          <DrawerTitle className={cn("flex items-center gap-2")}>
            <Users className="h-5 w-5" />
            角色用户管理
          </DrawerTitle>
          <DrawerDescription>
            管理角色「{role.role_name}」的关联用户
          </DrawerDescription>
        </DrawerHeader>

        <div className="mt-6 space-y-4">
          {/* 添加用户区域 */}
          <div className="flex gap-2">
            <Popover open={addUserOpen} onOpenChange={setAddUserOpen}>
              <PopoverTrigger asChild>
                <Button variant="outline" size="sm">
                  <Plus className=" h-4 w-4" />
                  <span className="ml-2">添加应用用户</span>
                </Button>
              </PopoverTrigger>
              <PopoverContent className="w-80 p-0" align="start">
                <Command>
                  <CommandInput
                    placeholder="搜索用户..."
                    value={searchText}
                    onValueChange={setSearchText}
                  />
                  <CommandList>
                    {availableLoading ? (
                      <div className="flex items-center justify-center py-6">
                        <Loader2 className="h-4 w-4 animate-spin" />
                      </div>
                    ) : (
                      <>
                        <CommandEmpty>未找到用户</CommandEmpty>
                        <CommandGroup>
                          {availableUsers.map((user) => (
                            <CommandItem
                              key={user.id}
                              value={String(user.id)}
                              onSelect={() => handleAddAppUser(user)}
                            >
                              <div className="flex flex-col">
                                <span>{user.nickname || user.username}</span>
                                <span className="text-xs text-muted-foreground">
                                  {user.username}
                                </span>
                              </div>
                            </CommandItem>
                          ))}
                        </CommandGroup>
                      </>
                    )}
                  </CommandList>
                </Command>
              </PopoverContent>
            </Popover>

            <div className="flex gap-2 flex-1">
              <Input
                placeholder="输入自定义用户参数"
                value={customUserParam}
                onChange={(e) => setCustomUserParam(e.target.value)}
                className="flex-1"
              />
              <Button
                variant="outline"
                size="sm"
                onClick={handleAddCustomUser}
                disabled={addUserMutation.isPending}
              >
                {addUserMutation.isPending ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  '添加'
                )}
              </Button>
            </div>
          </div>

          {/* 用户列表 */}
          <div className="border rounded-lg">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>用户</TableHead>
                  <TableHead>类型</TableHead>
                  <TableHead>参数</TableHead>
                  <TableHead className="w-[80px] text-center">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {isLoading ? (
                  <TableRow>
                    <TableCell colSpan={4} className="text-center py-8">
                      <Loader2 className="h-6 w-6 animate-spin mx-auto" />
                    </TableCell>
                  </TableRow>
                ) : users.length === 0 ? (
                  <TableRow>
                    <TableCell
                      colSpan={4}
                      className="text-center py-8 text-muted-foreground"
                    >
                      暂无关联用户
                    </TableCell>
                  </TableRow>
                ) : (
                  users.map((user) => (
                    <TableRow key={user.id}>
                      <TableCell>
                        {user.nickname || user.username || '-'}
                      </TableCell>
                      <TableCell>
                        <Badge variant={user.use_app_user ? 'default' : 'secondary'}>
                          {user.use_app_user ? '应用用户' : '自定义'}
                        </Badge>
                      </TableCell>
                      <TableCell className="font-mono text-sm">
                        {user.user_param}
                      </TableCell>
                      <TableCell className="text-center">
                        <ConfirmDialog
                          title="移除用户"
                          description={`确定要将用户「${user.nickname || user.username || user.user_param}」从该角色中移除吗？`}
                          onConfirm={async () => {
                            await deleteUserMutation.mutateAsync(user)
                          }}
                        >
                          <Button
                            variant="ghost"
                            size="sm"
                            className="h-8 w-8 p-0 "
                          >
                            <Trash2 className="h-4 w-4" />
                          </Button>
                        </ConfirmDialog>
                      </TableCell>
                    </TableRow>
                  ))
                )}
              </TableBody>
            </Table>
          </div>
        </div>
      </DrawerContent>
    </Drawer>
  )
}
