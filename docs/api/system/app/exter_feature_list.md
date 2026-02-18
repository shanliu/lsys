### 外部扩展能力列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| page.page | int | 否 | 页码 |
| page.limit | int | 否 | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data | array | 数据列表 |
| response.data.id | int | 记录ID |
| response.data.key | string | 扩展能力标识 |
| response.data.title | string | 扩展能力名称/标题 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/app/exter_feature_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
  "page": {
    "page": 1,
    "limit": 50
  }
}

```

```json
{
  "response": {
    "data": [
      {
        "id": "123",
        "key": "custom_x",
        "title": "自定义能力X"
      }
    ]
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
