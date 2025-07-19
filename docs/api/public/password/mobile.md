### 通过手机号验证码重置密码

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| mobile | string | 是 | 手机号码 |
| area_code | string | 是 | 国际区号 |
| code | string | 是 | 验证码 |
| new_password | string | 是 | 新密码 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/auth/password/mobile
Content-Type: application/json

{
    "mobile": "13800138001",
    "area_code":"86",
    "code": "542706",
    "new_password": "bb13579"
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
