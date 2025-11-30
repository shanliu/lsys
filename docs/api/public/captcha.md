### 验证码数据


```http
OPTIONS /api/captcha/
```

> 示例

```http
POST /api/captcha/
Content-Type: application/json

{
    "captcha_type":"login",
    "captcha_tag": "xxxx"
}
```


```json
{
  "response": {
    "image_data": "data:image/png;base64,xxx",
    "image_header": "image/png",
    "save_time": "60"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```