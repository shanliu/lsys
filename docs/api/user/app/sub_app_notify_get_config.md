### 子应用回调信息配置查询

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data[].app_id | int | 应用ID |
| response.data[].app_name | string | 应用名称 | 
| response.data[].call_url | string | 回调地址 |
| response.data[].change_time | int | 修改时间 |
| response.data[].change_user_id | int | 修改用户ID |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |


> 示例

```http
POST /api/user/app/sub_app_notify_get_config
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
        "app_id": "1",
        "app_name": "ddff237222",
        "call_url": "https://www.baidu.com/",
        "change_time": "1749450206",
        "change_user_id": "1"
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