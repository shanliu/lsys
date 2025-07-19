
### 获取RBAC映射信息

> 响应参数说明

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.audit_is.key | string | 授权状态键值 |
| response.audit_is.val | string | 授权状态描述 |
| response.audit_result.key | string | 审核结果键值 |
| response.audit_result.val | string | 审核结果描述 |
| response.role_res_range.key | string | 角色资源范围键值 |
| response.role_res_range.val | string | 角色资源范围描述 |
| response.role_user_range.key | string | 角色用户范围键值 |
| response.role_user_range.val | string | 角色用户范围描述 |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app_rbac/base/mapping
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

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