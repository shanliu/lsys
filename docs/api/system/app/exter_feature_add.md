### 添加外部扩展能力

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| feature_key | string | 是 | 扩展能力标识(如 "sms" / "mail" / "custom_x") |
| title | string | 是 | 扩展能力名称/标题 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | int | 新增记录ID |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/app/exter_feature_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
  "feature_key": "custom_x",
  "title": "自定义能力X"
}

```

```json
{
  "response": {
    "id": "123"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
