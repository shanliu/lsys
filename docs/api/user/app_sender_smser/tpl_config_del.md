### 删除短信模板配置

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| config_id | int | 是 | 配置ID |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app_sender/smser/tpl_config_del
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "config_id":32
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