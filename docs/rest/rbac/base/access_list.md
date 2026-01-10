### 批量检查权限

> payload参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| menu_res | array | 权限检查数组 |
| menu_res[].name | string | 检查项名称 |
| menu_res[].check_res.user_param | string | 用户参数 |
| menu_res[].check_res.token_data | string | token数据 |
| menu_res[].check_res.access.role_key | array | 角色key数组 |
| menu_res[].check_res.access.check_res | array | 检查资源数组 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.result | array | 检查结果数组 |
| response.result.name | string | 检查项名称 |
| response.result.status | string | 检查状态 0:失败 1:成功 |

> 示例

```http
POST /rest/rbac/base?method=access_list
Content-type:application/json

{
    "menu_res": [
        {
            "name":"xxx",
            "check_res":{
                "user_param": "account_11",
                "token_data": "sub",
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
        }
    ]
}
```

```json
{
  "response": {
    "result": [
      {
        "name": "xxx",
        "status": "0"
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