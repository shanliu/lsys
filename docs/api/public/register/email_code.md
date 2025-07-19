### 注册邮件发送

> 请求参数 

>   验证码 `/captcha/register-email/{captcha.key}` 注意 key 保存一致

| 参数名         | 类型   | 必填 | 说明           |
| -------------- | ------ | ---- | -------------- |
| email          | string | 是   | 邮箱地址       |
| captcha.code   | string | 是   | 验证码内容     |
| captcha.key | string | 是 | 自定义随机字符串,跟验证时保持一致 |

> 响应参数

| 参数名             | 类型   | 说明         |
| ------------------ | ------ | ------------ |
| result.code        | string | 响应码       |
| result.message     | string | 响应信息     |
| result.state       | string | 状态         |

> 示例

```http
GET  /captcha/register-email/shan.liu.msn.com
```

```http
POST /api/auth/register/email-code
Content-Type: application/json

{
    "email": "rustlang@qq.com",
     "captcha":  {
        "code":"E7k",
        "key":"shan.liu.msn.com"
    }
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


