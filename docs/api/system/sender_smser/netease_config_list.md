### 网易云短信配置列表接口

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| ids | array | 否 | 配置ID列表 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data[].id | string | 配置ID |
| response.data[].name | string | 配置名称 |
| response.data[].access_key | string | 访问密钥 |
| response.data[].access_secret | string | 访问密钥密文 |
| response.data[].hide_access_key | string | 隐藏后的访问密钥 |
| response.data[].callback_url | string | 回调URL |
| response.data[].limit | string | 限制数量 |
| response.data[].change_time | string | 修改时间 |
| response.data[].change_user_id | string | 修改用户ID |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/sender/smser/netease_config_list
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
        "access_key": "cccccccccccc",
        "access_secret": "cccccccccccc",
        "callback_url": "http://127.0.0.1:8080/notify/sms/12/",
        "change_time": "1748006674",
        "change_user_id": "1",
        "hide_access_key": "cc**cc",
        "id": "12",
        "limit": "11",
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