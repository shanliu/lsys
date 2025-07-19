### 添加邮件模板内容

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| tpl_id | string | 是 | 模板ID |
| tpl_data | string | 是 | 模板内容(支持变量: {{变量名}}) |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 内容ID |
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态 |

> 示例

```http
POST /api/user/app_sender/mailer/tpl_body_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "tpl_id":"test1",
   "tpl_data":"bad \{\{code\}\} aa is: \{\{aa\}\}"
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