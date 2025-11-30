### 登录相关字典

```http
POST /api/auth/login/mapping
Content-Type: application/json

{
   
}
```

```json
{
  "response": {
    "exter_type": [
      {
        "key": "wechat",
        "val": "dict-exter-type-{}:{msg:'wechat'}"
      }
    ],
    "login_type": [
      {
        "key": "email",
        "val": "dict-login-type-{}:{msg:'email'}",
        "validity": "259200"
      },
      {
        "key": "email-code",
        "val": "dict-login-type-{}:{msg:'email-code'}",
        "validity": "259200"
      },
      {
        "key": "name",
        "val": "dict-login-type-{}:{msg:'name'}",
        "validity": "259200"
      },
      {
        "key": "mobile",
        "val": "dict-login-type-{}:{msg:'mobile'}",
        "validity": "259200"
      },
      {
        "key": "mobile-code",
        "val": "dict-login-type-{}:{msg:'mobile-code'}",
        "validity": "259200"
      },
      {
        "key": "external",
        "val": "dict-login-type-{}:{msg:'external'}",
        "validity": "259200"
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
