
### 获取短信验证码接口

> 请求参数

> 验证码 /captcha/register-sms/{captcha.key}

| 参数名           | 类型   | 必填 | 说明         |
|-----------------|--------|------|--------------|
| mobile         | string | 是   | 手机号       |
| area_code      | string | 是   | 国家区号     |
| captcha.code   | string | 是   | 验证码       |
| captcha.key    | string | 是   | 验证码key    |

> 响应参数

| 参数名          | 类型   | 说明     |
|----------------|--------|----------|
| result.code    | string | 状态码   |
| result.message | string | 返回消息 |
| result.state   | string | 返回状态 |

> 示例

```http
GET  /captcha/register-sms/8613800138000
```


> 示例

```http
POST /api/auth/register/sms-code
Content-Type: application/json

{
    "mobile": "13800138001",
    "area_code": "86",
    "captcha":  {
        "code":"wg5",
        "key":"8613800138000"
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




