
### 添加华为云短信模板配置

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int | 是 | 应用ID |
| name | string | 是 | 配置名称 |
| hw_config_id | int | 是 | 华为云配置ID |
| tpl_key | string | 是 | 模板标识 |
| signature | string | 是 | 短信签名 |
| sender | string | 是 | 发送者 |
| template_id | string | 是 | 模板ID |
| template_map | string | 是 | 模板参数映射 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 配置ID |
| result.code | string | 状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app_sender/smser/hw_app_config_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "app_id":16,
    "name": "xxddx",
    "hw_config_id":10,
    "tpl_key": "adfad",
    "signature": "adfad",
    "sender": "adfad",
     "template_id": "adfad",
    "template_map": "adfad"
}
```

```json
{
  "response": {
    "id": "27"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```