### OAuth登录可用的scope列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int | 是 | 父应用ID,系统传0 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.scope | array | 权限范围列表 |
| response.scope.desc | string | 权限描述 |
| response.scope.key | string | 权限标识 |
| response.scope.name | string | 权限名称 |
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态标识 |

> 示例

```http
POST /api/user/app/oauth_client_scope_data
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
     "app_id": 0
}
```


```json
{
  "response": {
    "scope": [
      {
        "desc": "用户资料",
        "key": "user_info",
        "name": "用户资料"
      },
      {
        "desc": "用户邮箱",
        "key": "user_email",
        "name": "用户邮箱"
      },
      {
        "desc": "用户手机号",
        "key": "user_mobile",
        "name": "用户手机号"
      },
      {
        "desc": "用户收货地址",
        "key": "user_address",
        "name": "用户收货地址"
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