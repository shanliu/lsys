### 添加邮箱

> 请求参数

| 参数名 | 类型 | 必填 | 描述 |
|--------|------|------|------|
| email | string | 是 | 邮箱地址 |

> 响应参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| response.id | string | 邮箱ID |
| result.code | string | 返回码 |
| result.message | string | 返回信息 |
| result.state | string | 返回状态 |

> 示例

```http
### add mail
POST /api/user/profile/email/add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "email": "ssss11121@qq.com"
}
```

```json
{
  "response": {
    "id": "2"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```