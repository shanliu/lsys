### 发送短信

> payload参数

| 参数         | 类型      | 是否必填   | 描述    |
|-------------|-----------|------------|--------|
| mobile     | []string  | 是       | 接收手机号|
| tpl_id     | string  | 是       | 模板,在后台创建|
| data     | object |是 | 内容JSON数据   {"key":"val"}  |
| send_time     | string  | 否      | 发送时间 |
| max_try     | int  | 否     | 失败重试次数|


> 示例

```http
POST /rest/sms?method=send
Content-type:application/json

{
    "mobile":["13800138000"],
    "tpl_key":"dddd",
    "data":{"code":"11","aa":"111"},
    "send_time":"2024-12-11 10:00:00",
    "max_try":1
}
```

```json
{
  "response": {
    "detail": [
      {
        "mobile": "13800138000",
        "snid": "7338560936988594176"
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