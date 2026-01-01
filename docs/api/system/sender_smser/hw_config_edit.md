### 华为云短信配置编辑接口

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| id | int | 是 | 配置ID |
| name | string | 是 | 配置名称 |
| url | string | 是 | 接口地址 |
| app_key | string | 是 | 应用Key |
| app_secret | string | 是 | 应用密钥 |
| callback_key | string | 否 | 回调密钥 |
| limit | int | 否 | 限制数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.num | string | 更新数量 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/sender/smser/hw_config_edit
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "id": 10,
   "name": "bbbb",
    "url": "http://rage1.xxx.com/dddd",
    "app_key":"dddddddddddddd",
    "app_secret":"dddddddddddddd",
    "callback_key":"ddddddd",
    "limit":11
}
```

```json
{
  "response": {
    "num": "1"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```