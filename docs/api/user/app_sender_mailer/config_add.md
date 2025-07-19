### 添加邮件发送配置

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int | 是 | 应用ID |
| priority | int | 是 | 优先级 |
| config_type | int | 是 | 配置类型 |
| config_data | int | 是 | 配置数据ID |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 配置ID |
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态 |

> 示例

```http
POST /api/user/app_sender/mailer/config_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "app_id": 10,
    "priority": 1,
    "config_type": 3,
    "config_data":11
}
```

```json
{
  "response": {
    "id": "11"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```