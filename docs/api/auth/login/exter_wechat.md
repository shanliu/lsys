

### 微信扫码登录

> 请求参数

### 获取扫码登录二维码

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| login_state | string | 是 | 登录状态标识(自定义) |
| login_callback | string | 是 | 回调地址 |

### 检测扫码登录状态

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| login_state | string | 是 | 登录状态标识 |

### 扫码后完成登录页面

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| code | string | 是 | 平台回调 code |
| callback_state | string | 是 | 回调 state |

> 示例

```http
### 获取扫码登录二维码
POST /api/auth/exter_login_url/wechat
Content-Type: application/json

{
   "login_state":"sssss",
   "login_callback":"http://xxxx",
}

```


> 示例

```http
### 检测扫码登录状态
POST /api/auth/exter_state_check/wechat
Content-Type: application/json

{
   "login_state":"sssss"
}

```

```json
// 需要 MFA 时，接口可能返回：
// {
//   "response": { "mfa_token": "..." },
//   "result": { "code": "200", "message": "ok", "state": "ok" }
// }
```

```http
### 需要 MFA 时完成登录
POST /api/auth/login/mfa-verify
Content-Type: application/json

{
   "mfa_token": "...",
   "code": "123456"
}

```


> 示例

```http
### 扫码后,完成登录页面
POST /api/auth/exter_state_callback/wechat
Content-Type: application/json

{
   "code":"22222222222222",
   "callback_state":"sssss",
}
```
