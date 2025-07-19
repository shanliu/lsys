
### 完成OAuth登陆授权

> 请求参数

| 参数名         | 类型   | 必填 | 说明                |
| -------------- | ------ | ---- | ------------------- |
| client_id      | string | 是   | 客户端ID            |
| scope          | string | 是   | 授权范围            |
| redirect_uri   | string | 是   | 授权回调地址        |

> 响应参数

| 参数名                | 类型   | 说明               |
| --------------------- | ------ | ------------------ |
| response.code         | string | 授权码             |
| result.code           | string | 用于完成`/oauth/token` 接口的code      |
| result.message        | string | 业务响应信息       |
| result.state          | string | 业务状态           |


> 示例

```http
POST /api/oauth/do
Content-Type: application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "client_id": "subapp112",
    "scope": "user_info,user_email,user_mobile,user_address",
    "redirect_uri":"http://xxx.com"
}
```

```json
{
  "response": {
    "code": "ef687e40753914047f835d6c59013010"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```