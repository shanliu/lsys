
### 禁用指定应用

> 请求参数

| 参数名 | 类型 | 是否必填 | 描述 |
|--------|------|----------|------|
| app_id | int  | 是       | 应用ID |

> 响应参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| code   | string | 状态码 |
| message| string | 响应消息 |
| state  | string | 状态说明 |


> 示例

```http
POST /api/system/app/disable
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "app_id": 8
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