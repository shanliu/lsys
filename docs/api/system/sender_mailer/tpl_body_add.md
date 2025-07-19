
### 添加邮件模板内容

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| tpl_id | string | 是 | 模板ID |
| tpl_data | string | 是 | 模板内容 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 模板ID |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/sender/mailer/tpl_body_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "tpl_id":"reg_code_body",
   "tpl_data":"注册验证码是 \{\{code\}\} ,请在 \{\{ttl|second_format(i=\"分钟\")\}\} 内使用"
}
```

```json
{
  "response": {
    "id": "7"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
