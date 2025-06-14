rbac-access-unauth = 资源 {$res_name} 对应的操作 {$op_name} 未授权 (用户ID:{$user_id})
rbac-access-block = 资源 {$res_name} 对应的操作 {$op_name} 被禁用 (用户ID:{$user_id})
rbac-parse-res-str-fail = 解析权限字符串失败:{$token}
rbac-res-exits = 资源 {$res_type}({$res_data}) 已经存在,名称为:{$old_name}
rbac-role-exist = 存在角色( {$name} : {$key} )  跟当前操作冲突,请检查key或名是否重复
rbac-res-perm-wrong = 此角色({$name}:{$role_id})不能关联权限({$range})
rbac-res-op-user-wrong = 此角色({$name}:{$role_id})不能关联用户({$range})
rbac-role-bad-perm-user = 非系统角色不能加非本角色用户资源,资源未:{$res}-{$op},用户为:{$user_id}
rbac-res-add-bad-op = 资源({$res_name}不能添加操作:{$op_name}),用户不匹配:{$user_id}
rbac-role-bad-res-op = 资源 {$res} 未关联操作:{$op}
rbac-role-bad-op-user = 操作数据 {$op} 非不属于对应资源 {$res} 用户:{$op_user_id}
rbac-check-fail = 权限校验失败
rbac-parse-op-str-fail = 解析清理缓存{$token}字符串失败:{$msg}
rbac-op-exits = 操作 {$op_type} 已经存在,名称为:{{$old_name}}




# 状态
status-RbacRoleStatus-Enable = 启用
status-RbacRoleStatus-Delete = 删除

status-RbacRoleResRange-Exclude = 禁止指定授权
status-RbacRoleResRange-Any = 访问任意资源
status-RbacRoleResRange-Include = 包含指定授权



status-RbacRoleUserRange-Custom = 指定用户
status-RbacRoleUserRange-Session = 会话角色

status-RbacResStatus-Enable = 启用
status-RbacResStatus-Delete = 删除


status-RbacOpStatus-Enable = 启用
status-RbacOpStatus-Delete = 删除


status-RbacOpResStatus-Enable = 启用
status-RbacOpResStatus-Delete = 删除

status-RbacPermStatus-Enable = 启用
status-RbacPermStatus-Delete = 删除





status-RbacRoleUserStatus-Enable = 启用
status-RbacRoleUserStatus-Delete = 删除

status-RbacAuditResult-Succ = 授权失败
status-RbacAuditResult-Fail = 授权通过

status-RbacAuditIs-Yes = 授权通过
status-RbacAuditIs-No = 授权失败

#校验名称


valid-rule-name-login_token_data  = 登陆标识符
valid-rule-name-res_type = 资源类型
valid-rule-name-res_data = 资源标识
valid-rule-name-res_name = 资源名称
valid-rule-name-op_key_data = 操作标识
valid-rule-name-op_key = 操作标识
valid-rule-name-op_name = 操作名称
valid-rule-name-role_key = 角色标识
valid-rule-name-role_name = 角色名称