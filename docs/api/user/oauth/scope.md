### 获取授权Scope信息

> 请求参数

| 参数名      | 类型   | 必填 | 说明         |
| ----------- | ------ | ---- | ------------ |
| client_id   | string | 是   | 应用ID       |
| scope       | string | 是   | 授权范围key  |

> 响应参数

| 参数名                  | 类型   | 说明           |
| ----------------------- | ------ | -------------- |
| response.scope          | array  | 授权范围列表   |
| response.scope.desc     | string | 授权范围描述   |
| response.scope.key      | string | 授权范围key    |
| response.scope.name     | string | 授权范围名称   |
| result.code             | string | 结果码         |
| result.message          | string | 结果信息       |
| result.state            | string | 状态           |


> 示例

```http
POST /api/oauth/scope
Content-Type: application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "client_id": "subapp112",
    "scope": "user_address"
}
```

```json
{
  "response": {
    "scope": [
      {
        "desc": "用户收货地址",
        "key": "user_address",
        "name": "用户收货地址"
      }
    ]
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```