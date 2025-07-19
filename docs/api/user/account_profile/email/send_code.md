
### 发送邮箱验证码

> 请求参数

| 参数名 | 类型 | 必填 | 描述 |
|--------|------|------|------|
| captcha.code | string | 是 | 图形验证码 |
| captcha.key | string | 是 | 自定义随机字符串,跟验证时保持一致 |
| email | string | 是 | 邮箱地址 |

> 响应参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| result.code | string | 返回码 |
| result.message | string | 返回信息 |
| result.state | string | 返回状态 |


> 示例

```http
GET  /captcha/add-email/ddddddddd

```

```http
POST /api/user/profile/email/send_code
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
     "captcha":  {
        "code":"8lj",
        "key":"ddddddddd"
    },
    "email":"ssss11121@qq.com"
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
