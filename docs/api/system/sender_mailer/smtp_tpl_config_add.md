
### 添加SMTP模板配置

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| smtp_config_id | int | 是 | SMTP配置ID |
| name | string | 是 | 配置名称 |
| tpl_key | string | 是 | 模板键值 |
| from_email | string | 是 | 发件人邮箱 |
| reply_email | string | 是 | 回复邮箱 |
| subject_tpl_id | string | 是 | 主题模板ID |
| body_tpl_id | string | 是 | 正文模板ID |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 配置ID |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/sender/mailer/smtp_tpl_config_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "smtp_config_id": 4,
   "name": "邮件xxx",
   "tpl_key": "valid_code_register_email",
   "from_email": "rustlang@qq.com",
   "reply_email": "rustlang@qq.com",
   "subject_tpl_id": "reg_code_title",
   "body_tpl_id": "reg_code_body"
}
```

```json
{
  "response": {
    "id": "19"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```