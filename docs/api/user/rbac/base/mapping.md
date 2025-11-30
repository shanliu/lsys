### RBAC映射关系查询

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| audit_result.key | string | 审计结果键值 |
| audit_result.val | string | 审计结果描述 |
| role_res_range.key | string | 角色资源范围键值 |
| role_res_range.val | string | 角色资源范围描述 |
| role_user_range.key | string | 角色用户范围键值 |
| role_user_range.val | string | 角色用户范围描述 |


> 示例

```http
POST /api/user/rbac/base/mapping
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    
}
```

```json
{
  "response": {
    "audit_result": [
      {
        "key": "1",
        "val": "授权失败"
      },
      {
        "key": "2",
        "val": "授权通过"
      }
    ],
    "role_res_range": [
      {
        "key": "3",
        "val": "禁止指定授权"
      },
      {
        "key": "2",
        "val": "访问任意资源"
      },
      {
        "key": "1",
        "val": "包含指定授权"
      }
    ],
    "role_user_range": [
      {
        "key": "1",
        "val": "指定用户"
      },
      {
        "key": "2",
        "val": "会话角色"
      }
    ]
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
