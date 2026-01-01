### 发送手机验证码

> 请求参数

| 参数名 | 类型 | 必填 | 描述 |
|--------|------|------|------|
| captcha.code | string | 是 | 图形验证码 |
| captcha.key | string | 是 | 自定义随机字符串,跟验证时保持一致 |
| area_code | string | 是 | 区号 |
| mobile | string | 是 | 手机号 |

> 响应参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| result.code | string | 返回码 |
| result.message | string | 返回信息 |
| result.state | string | 返回状态 |


> 示例

```http
GET  /captcha/add-sms/fasdfa
```


```http
POST /api/user/profile/mobile/send_code
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "captcha":  {
        "code":"hyr",
        "key":"fasdfa"
    },
    "area_code":"86",
    "mobile": "13800138004"
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