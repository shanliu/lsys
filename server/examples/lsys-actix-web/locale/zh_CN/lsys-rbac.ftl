rbac-access-unauth = 资源 {$res_name} 对应的操作 {$op_name} 未授权 [用户ID:{$user_id}]
rbac-access-block = 资源 {$res_name} 对应的操作 {$op_name} 被禁用 [用户ID:{$user_id}]
rbac-parse-res-str-fail = 解析权限字符串失败:{$token}
rbac-res-exits =  资源[{$name}:{$res_type}:{$res_data}]已经存在
rbac-role-exist = 角色[{$name}]已经存在,对应KEY:{$key}
rbac-res-perm-wrong =此角色[{$name}:{$role_id}]不能关联权限[{$range}]
rbac-res-op-user-wrong = 此角色[{$name}:{$role_id}]不能关联用户[{$range}]
rbac-role-bad-perm-user = 非系统角色不能加非本角色用户资源,资源未:{$res}-{$op},用户为:{$user_id}
rbac-res-add-bad-op = 资源[{$res_name}不能添加操作:{$op_name}],用户不匹配:{$user_id}
rbac-role-bad-res-op =资源 {$res} 未关联操作:{$op}
rbac-role-bad-op-user = 操作数据 {$op} 非不属于对应资源 {$res} 用户:{$op_user_id}
rbac-check-fail = 权限校验失败
rbac-check-length= 字段[{$key}]长度异常:{$msg}
rbac-parse-op-str-fail= 解析清理缓存{$token}字符串失败:{$msg}
rbac-op-exits= 操作[{$name}:{$res_type}:{$res_data}]已经存在




# 状态

status-RbacRoleStatus-Enable=启用
status-RbacRoleStatus-Delete=删除





status-RbacRoleResRange-Exclude= 由RbacRoleModel决定,排除某些授权
status-RbacRoleResRange-Any=     任意资源
status-RbacRoleResRange-Include= 由RbacRoleModel决定,包含某些授权




status-RbacRoleUserRange-Custom=  自定义用户
status-RbacRoleUserRange-Session= 会话角色





status-RbacResStatus-Enable=启用
status-RbacResStatus-Delete=删除





status-RbacOpStatus-Enable=启用
status-RbacOpStatus-Delete=删除





status-RbacOpResStatus-Enable=启用
status-RbacOpResStatus-Delete=删除





status-RbacPermStatus-Enable=启用
status-RbacPermStatus-Delete=删除





status-RbacRoleUserStatus-Enable=启用
status-RbacRoleUserStatus-Delete=删除





status-RbacAuditResult-Succ=授权失败
status-RbacAuditResult-Fail=授权通过





status-RbacAuditIs-Yes=授权通过
status-RbacAuditIs-No=授权失败
