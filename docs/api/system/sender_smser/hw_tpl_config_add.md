### 华为云短信模板配置添加接口

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| name | string | 是 | 配置名称 |
| hw_config_id | int | 是 | 华为云配置ID |
| tpl_key | string | 是 | 模板键名 |
| signature | string | 是 | 短信签名 |
| sender | string | 是 | 发送者 |
| template_id | string | 是 | 模板ID |
| template_map | string | 是 | 模板映射 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 配置ID |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

### huawei短信配置关联短信发送

> 示例

```http
POST /api/system/sender/smser/hw_tpl_config_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "name": "xxxssss",
    "hw_config_id":10,
    "tpl_key": "valid_code_account_mobile",
    "signature": "adfadadfad",
    "sender": "adfaadfadd",
     "template_id": "adfadadfad",
    "template_map": "adadfadfad"
}

```

```json
{
  "response": {
    "id": "25"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```