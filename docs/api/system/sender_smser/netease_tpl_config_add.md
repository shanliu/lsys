### 网易云短信模板配置添加接口

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| name | string | 是 | 配置名称 |
| config_id | int | 是 | 网易云配置ID |
| tpl_key | string | 是 | 模板键名 |
| template_id | string | 是 | 模板ID |
| template_map | string | 是 | 模板映射 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 配置ID |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

### netease短信配置关联短信发送

> 示例

```http
POST /api/system/sender/smser/netease_tpl_config_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "name": "xxxdddddd",
    "config_id":12,
    "tpl_key": "valid_code_login_mobile",
     "template_id": "adfad",
    "template_map": "adfad"
}
```

```json
{
  "response": {
    "id": "23"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```