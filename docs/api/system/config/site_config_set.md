### 站点配置设置

> 请求参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| site_tips | string | 站点提示信息 |
| password_timeout | int | 密码超时时间(秒) |
| disable_old_password | string | 是否禁用旧密码 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/config/site_config/set
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "site_tips":"站点提示",
   "password_timeout":111,
   "disable_old_password":"1"
}

```

```json
{
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```



