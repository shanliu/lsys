### 添加短信服务配置

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int | 是 | 应用ID |
| priority | int | 是 | 优先级 |
| config_type | int | 是 | 配置类型 |
| config_data | int | 是 | 配置数据ID |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 配置ID |
| result.code | string | 状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app_sender/smser/config_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "app_id": 16,
    "priority": 1,
    "config_type": 10,
    "config_data":{
      "area": "86", 
      "mobile": "13800138000"
    }
}
```

```json
{
  "response": {
    "id": "12"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```

