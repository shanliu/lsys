### 取消发送短信


> payload参数

| 参数         | 类型      | 是否必填   | 描述    |
|-------------|-----------|------------|--------|
| snid_data     | []string  | 是       | 消息ID,发送接口返回|


> 示例

```http
POST /rest/sms?method=cancel
Content-type:application/json

{
    "snid_data": ["7338560936988594176"]
}
```

```json
{
  "response": {
    "detail": [
      {
        "msg": "当前状态不可被取消",
        "snid": "7338560936988594176",
        "status": "0"
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