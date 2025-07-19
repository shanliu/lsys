### 添加APP接口

> 请求参数

| 参数名 | 类型 | 描述 |
|--------|------|------|
| parent_app_id | int | 父应用ID |
| name | string | 应用名称 |
| client_id | string | 应用标识 |

> 响应参数 

| 参数名 | 类型 | 描述 |
|--------|------|------|
| response.id | string | 新增应用的ID |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |


> 示例

```http
POST /api/user/app/add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}


{
    "parent_app_id": null,
    "name": "测试aPP",
    "client_id": "app001"
}

```

```json
{
  "response": {
    "id": "7"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```