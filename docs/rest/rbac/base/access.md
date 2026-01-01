### 检查权限

> payload参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| user_param | string | 用户参数 |
| token_data | string | token数据 |
| request_ip | string | 请求IP |
| access.role_key | array | 角色key数组 |
| access.role_key[].role_key | string | 角色key |
| access.role_key[].use_app_user | boolean | 是否使用app用户 |
| access.role_key[].user_param | string | 用户参数 |
| access.check_res | array | 检查资源数组 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.check_detail | object | 失败详情 |

> 示例

```http
POST /rest/rbac/base?method=access
Content-type:application/json

{
    "user_param":"ccc",
    "token_data": null,
    "request_ip": "1.1.1.1",
    "access":{
            "role_key":[
               {
                    "role_key":"xxxp",
                    "use_app_user":false,
                    "user_param":"account_11"
                }
            ],
            "check_res":[
               [
                 {
                    "res_type":"xx1",
                    "res_data":"",
                    "use_app_user":false,
                    "user_param":"account_11",
                    "ops":[{"op_key":"xx5","req_auth":"1"}]
                }
               ]
            ]
    }
}
```

```json
# 成功
{
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```


```json
# 失败
{
  "response": {
    "check_detail": {
      "xx1": [
        "资源 xx3 对应的操作 xx5 被禁用 (用户ID:86)"
      ]
    }
  },
  "result": {
    "code": "403",
    "message": "权限校验失败",
    "state": "check_fail"
  }
}
```

