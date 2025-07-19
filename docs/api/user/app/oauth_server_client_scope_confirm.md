### 子应用OAuth登录申请scope权限审核

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_req_id | int | 是 | 应用请求ID |
| confirm_status | int | 是 | 确认状态 |
| confirm_note | string | 是 | 确认备注 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app/oauth_server_client_scope_confirm
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "app_req_id": 20,
   "confirm_status": 2,
   "confirm_note": "ssss"
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