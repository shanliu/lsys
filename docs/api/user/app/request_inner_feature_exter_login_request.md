### 申请外部登录功能

> 请求参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| app_id | int  | 应用ID |

> 响应参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app/request_inner_feature_exter_login_request
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
     "app_id": 1
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