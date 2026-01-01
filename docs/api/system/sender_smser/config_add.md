### 短信配置添加接口

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| priority | int | 是 | 优先级 |
| config_type | int | 是 | 配置类型 |
| config_data | int | 是 | 配置数据ID |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 配置ID |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/sender/smser/config_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "priority": 1,
    "config_type": 3,
    "config_data":2
}
```

```json
{
  "response": {
    "id": "9"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```