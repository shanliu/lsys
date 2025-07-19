### 获取用户动态资源数据

> 请求参数

| 参数名         | 类型    | 必填 | 说明                |
| -------------- | ------- | ---- | ------------------- |
| user_any       | 任意    | 否   | 用户任意条件        |
| count_num      | bool    | 否   | 是否统计数量        |
| limit.forward  | bool    | 否   | 是否正向分页        |
| limit.page     | int     | 否   | 页码                |
| limit.limit    | int     | 否   | 每页条数            |

> 响应参数

| 参数名                                 | 类型    | 说明                       |
| -------------------------------------- | ------- | -------------------------- |
| response.next_id                       | 任意    | 下一页ID                   |
| response.total                         | string  | 总数                       |
| response.tpl_data                      | array   | 资源数据列表               |
| response.tpl_data[].op_data            | array   | 操作数据列表               |
| response.tpl_data[].op_data[].op_id    | string  | 操作ID                     |
| response.tpl_data[].op_data[].op_key   | string  | 操作标识                   |
| response.tpl_data[].op_data[].op_name  | string  | 操作名称                   |
| response.tpl_data[].res_id             | string  | 资源ID                     |
| response.tpl_data[].res_type           | string  | 资源类型                   |
| response.tpl_data[].user_data          | string  | 用户数据ID                 |
| response.tpl_data[].user_info          | object  | 用户信息                   |
| response.tpl_data[].user_info.app_id   | string  | 应用ID                     |
| response.tpl_data[].user_info.change_time | string | 变更时间(秒)               |
| response.tpl_data[].user_info.id       | string  | 用户ID                     |
| response.tpl_data[].user_info.user_account | string | 用户账号                  |
| response.tpl_data[].user_info.user_data | string  | 用户数据                   |
| response.tpl_data[].user_info.user_nickname | string | 用户昵称                 |
| result.code                            | string  | 结果码                     |
| result.message                         | string  | 结果信息                   |
| result.state                           | string  | 状态                       |

> 示例

```http
POST /api/system/rbac/res/dynamic_res_data_global_user
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "user_any":null,
    "count_num":true,
    "limit":{
      "forward":true,
      "page":1,
      "limit":10
   }
}
```

```json
{
  "response": {
    "next_id": null,
    "total": "6",
    "tpl_data": [
      {
        "op_data": [
          {
            "op_id": "2",
            "op_key": "address-edit",
            "op_name": "用户收货地址编辑"
          }
        ],
        "res_id": "3",
        "res_type": "global-user",
        "user_data": "27",
        "user_info": {
          "app_id": "1",
          "change_time": "1751453136",
          "id": "27",
          "user_account": "",
          "user_data": "ccc",
          "user_nickname": "ccc"
        }
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