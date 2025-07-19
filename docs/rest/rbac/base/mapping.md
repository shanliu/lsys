### 获取基础映射数据

> payload参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.audit_is | array | 授权状态列表 |
| response.audit_is.key | string | 状态key |
| response.audit_is.val | string | 状态说明 |
| response.audit_result | array | 授权结果列表 |
| response.audit_result.key | string | 结果key |
| response.audit_result.val | string | 结果说明 |
| response.role_res_range | array | 角色资源范围列表 |
| response.role_res_range.key | string | 范围key |
| response.role_res_range.val | string | 范围说明 |
| response.role_user_range | array | 角色用户范围列表 |
| response.role_user_range.key | string | 范围key |
| response.role_user_range.val | string | 范围说明 |

> 示例

```http
POST /rest/rbac/base?method=mapping
Content-type:application/json

{
    
}
```

```json
{
  "response": {
    "audit_is": [
      {
        "key": "1",
        "val": "授权通过"
      },
      {
        "key": "0",
        "val": "授权失败"
      }
    ],
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
