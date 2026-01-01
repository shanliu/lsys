### 获取短信服务配置列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| id | int | 否 | 配置ID，为null时获取所有配置 |
| app_id | int | 是 | 应用ID |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data[].add_time | string | 添加时间(秒) |
| response.data[].app_id | string | 应用ID |
| response.data[].config_data | string | 配置数据ID |
| response.data[].config_type | string | 配置类型 |
| response.data[].id | string | 配置ID |
| response.data[].priority | string | 优先级 |
| result.code | string | 状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app_sender/smser/config_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "id":null,
   "app_id":16
}
```

```json
{
  "response": {
    "data": [
      {
        "add_time": "1749877412",
        "app_id": "16",
        "config_data": "100",
        "config_type": "3",
        "id": "12",
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

