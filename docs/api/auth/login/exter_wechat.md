

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
