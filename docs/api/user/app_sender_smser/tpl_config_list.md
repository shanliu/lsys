### 获取短信模板配置列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| id | int | 否 | 配置ID |
| app_id | int | 否 | 应用ID |
| tpl | string | 否 | 模板标识 |
| app_info | boolean | 否 | 是否返回应用信息 |
| count_num | boolean | 是 | 是否统计总数 |
| page.page | int | 是 | 页码 |
| page.limit | int | 是 | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data[].app_client_id | string | 应用客户端ID |
| response.data[].app_id | string | 应用ID |
| response.data[].app_name | string | 应用名称 |
| response.data[].change_time | string | 修改时间(秒) |
| response.data[].change_user_id | string | 修改人ID |
| response.data[].config_data.sign_name | string | 短信签名 |
| response.data[].config_data.template_id | string | 模板ID |
| response.data[].config_data.template_map | string | 模板参数映射 |
| response.data[].id | string | 配置ID |
| response.data[].name | string | 配置名称 |
| response.data[].setting_id | string | 设置ID |
| response.data[].setting_key | string | 设置类型 |
| response.data[].setting_name | string | 设置名称 |
| response.data[].tpl_key | string | 模板标识 |
| response.data[].user_id | string | 用户ID |
| response.total | string | 总数量 |
| result.code | string | 状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app_sender/smser/tpl_config_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "id": null,
   "app_id":null,
   "tpl": null,
   "app_info":true,
   "count_num":true,
   "page":{
      "page":1,
      "limit":10
   }
}
```

```json
{
  "response": {
    "data": [
      {
        "app_client_id": "app001",
        "app_id": "16",
        "app_name": "测试aPP",
        "change_time": "1749896072",
        "change_user_id": "7",
        "config_data": {
          "sign_name": "xfffxddx",
          "template_id": "adfad",
          "template_map": "adfad"
        },
        "id": "32",
        "name": "xdddddddx",
        "setting_id": "18",
        "setting_key": "tenyun-sms-config",
        "setting_name": "bb121212bb",
        "tpl_key": "adfad",
        "user_id": "7"
      }
    ],
    "total": "0"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```