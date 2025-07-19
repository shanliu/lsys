### 阿里云短信模板配置添加接口

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| name | string | 是 | 配置名称 |
| ali_config_id | int | 是 | 阿里云配置ID |
| tpl_key | string | 是 | 模板键名 |
| aliyun_sms_tpl | string | 是 | 阿里云短信模板ID |
| aliyun_sign_name | string | 是 | 阿里云短信签名 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 配置ID |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

###  aliyun短信配置关联短信发送

> 示例

```http
POST /api/system/sender/smser/ali_tpl_config_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "name": "xxxee112",
    "ali_config_id":7,
    "tpl_key": "valid_code_register_mobile",
    "aliyun_sms_tpl": "sms_111",
    "aliyun_sign_name": "豆豆"
}
```

```json
{
  "response": {
    "id": "20"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```