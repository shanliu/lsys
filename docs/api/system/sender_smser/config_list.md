### 短信配置列表接口

> 请求参数

无需请求参数

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data[].id | string | 配置ID |
| response.data[].priority | string | 优先级 |
| response.data[].config_type | string | 配置类型 |
| response.data[].config_data | string | 配置数据ID |
| response.data[].add_time | string | 添加时间 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/sender/smser/config_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
}
```

```json
{
  "response": {
    "data": [
      {
        "add_time": "1749871733",
        "config_data": "2",
        "config_type": "3",
        "id": "9",
        "priority": "1"
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