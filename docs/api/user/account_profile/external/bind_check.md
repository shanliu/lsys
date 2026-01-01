> 示例

```http
### 检测是否完成绑定
POST /api/user/profile/exter/bind_check
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "login_type": "wechat",
    "login_state":"343444eee"
}
```

