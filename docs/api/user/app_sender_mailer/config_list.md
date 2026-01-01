### 获取邮件发送配置列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| id | int | 否 | 配置ID |
| app_id | int | 是 | 应用ID |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data.add_time | string | 添加时间(秒) |
| response.data.app_id | string | 应用ID |
| response.data.config_data | string | 配置数据ID |
| response.data.config_type | string | 配置类型 |
| response.data.id | string | 配置ID |
| response.data.priority | string | 优先级 |
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态 |

> 示例

```http
POST /api/user/app_sender/mailer/config_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "id":null,
   "app_id":10
}
```

```json
{
  "response": {
    "data": [
      {
        "add_time": "1749871815",
        "app_id": "10",
        "config_data": "11",
        "config_type": "3",
        "id": "11",
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