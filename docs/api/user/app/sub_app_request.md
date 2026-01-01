### 子应用权限申请

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int  | 是   | 应用ID |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| code | string | 响应代码 |
| message | string | 响应消息 |
| state | string | 响应状态 |

> 示例

```http
POST /api/user/app/sub_app_request
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
     "app_id": 16
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