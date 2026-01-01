### 邮件配置列表


> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data | array | 配置数据列表 |
| response.data.add_time | int | 添加时间（秒） |
| response.data.config_data | string | 配置数据 |
| response.data.config_type | string | 配置类型 |
| response.data.id | string | 配置ID |
| response.data.priority | string | 优先级 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
### 配置列表
POST /api/system/sender/mailer/config_list
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
        "add_time": "1749869952",
        "config_data": "11",
        "config_type": "3",
        "id": "5",
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