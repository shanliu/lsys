
### 获取网易云短信配置列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| ids | array | 否 | 配置ID列表，为null时获取所有配置 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data[].app_id | string | 应用ID |
| response.data[].id | string | 配置ID |
| response.data[].name | string | 配置名称 |
| result.code | string | 状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app_sender/smser/netease_config_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "ids":null
}
```

```json
{
  "response": {
    "data": [
      {
        "app_id": "cc**cc",
        "id": "12",
        "name": "bbbbcccc"
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
