### 获取全局动态资源类型


> 响应参数

| 参数名                | 类型   | 说明         |
|----------------------|--------|--------------|
| response.res_type    | array  | 资源类型列表 |
| response.res_type[].res_name | string | 资源名称     |
| response.res_type[].res_type | string | 资源类型标识 |
| result.code          | string | 结果代码     |
| result.message       | string | 结果信息     |
| result.state         | string | 结果状态     |
> 示例

```http
POST /api/system/rbac/res/dynamic_res_type
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   
}
```

```json
{
  "response": {
    "res_type": [
      {
        "res_name": "用户全局权限",
        "res_type": "global-user"
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