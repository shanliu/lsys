### 手机号完成注册

> 请求参数

| 参数名 | 类型 | 必填 | 描述 |
|--------|------|------|------|
| nikename | string | 是 | 用户昵称 |
| mobile | string | 是 | 手机号码 |
| area_code | string | 是 | 国际区号 |
| code | string | 是 | 验证码 |
| password | string | 是 | 密码 |

> 响应参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| response.id | string | 用户ID |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |


> 示例

```http
POST /api/auth/register/sms
Content-Type: application/json

{
    "nikename":"SHAN11",
    "mobile": "13800138001",
    "area_code": "86",
    "code":"179947",
    "password":"11323d1d"
}
```


```json
{
  "response": {
    "id": "6"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
