> 示例

```http
### oauth bind
POST /api/user/profile/exter/bind_url
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "login_type": "wechat",
    "login_state":"343444eee",
    "callback_url":"http://www.lsys.cc/app.html#/user/info/oauth"
}
```