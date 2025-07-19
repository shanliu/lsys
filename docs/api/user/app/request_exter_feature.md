### 应用外部功能申请

> 系统应用跟子应用公用

> 请求参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| app_id | int | 应用ID |
| featuer_data | array | 功能列表数组.mail:邮件发送;sms:短信发送;barcode:二维码应用;rbac:权限管理|

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app/request_exter_feature
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "app_id":16,
    "featuer_data":["mail","sms","barcode","rbac"]
}
```

```json
{
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```
