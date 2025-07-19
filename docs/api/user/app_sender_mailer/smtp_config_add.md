### 添加SMTP发送配置

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int | 是 | 应用ID |
| smtp_config_id | int | 是 | SMTP配置ID |
| name | string | 是 | 配置名称 |
| tpl_key | string | 是 | 模板标识 |
| from_email | string | 是 | 发件人邮箱 |
| reply_email | string | 否 | 回复邮箱 |
| subject_tpl_id | string | 是 | 主题模板ID |
| body_tpl_id | string | 是 | 内容模板ID |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 配置ID |
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态 |

> 示例

```http
POST /api/user/app_sender/mailer/smtp_config_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "app_id":16,
   "smtp_config_id": 4,
   "name": "test12",
   "tpl_key": "ddddd",
   "from_email": "rustlang@qq.com",
   "reply_email": "rustlang@qq.com",
   "subject_tpl_id": "111",
   "body_tpl_id": "test1"
}
```

```json
{
  "response": {
    "id": "18"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```