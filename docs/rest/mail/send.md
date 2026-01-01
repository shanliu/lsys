
### 发送邮件

> payload参数

| 参数         | 类型      | 是否必填   | 描述    |
|-------------|-----------|------------|--------|
| to     | []string  | 是       | 接收邮箱|
| tpl_id     | string  | 是       | 模板,在后台创建|
| data     | object |是  | 内容JSON数据  {"key":"val"}   |
| reply     | string  |    否 | 回复邮箱  |
| send_time     | string  | 否      | 发送时间 |
| max_try     | int  | 否     | 失败重试次数|



> 示例

```http
POST /rest/mail?method=send
Content-type:application/json

{
    "to":["rustlang@qq.com"],
    "tpl_key":"ddddd",
    "data":{"code":"11","aa":"111999999"},
    "reply":"9@qq.com",
    "send_time":"2024-12-11 10:00:00",
    "max_try":1
}
```

```json
{
  "response": {
    "detail": [
      {
        "mail": "rustlang@qq.com",
        "snid": "7338556632068214784"
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
