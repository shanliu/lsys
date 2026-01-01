### 短信模板配置列表接口

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| id | int | 否 | 配置ID |
| tpl | string | 否 | 模板筛选 |
| app_info | boolean | 否 | 是否返回应用信息 |
| count_num | boolean | 否 | 是否返回总数 |
| page.page | int | 否 | 页码 |
| page.limit | int | 否 | 每页数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data[].id | string | 配置ID |
| response.data[].name | string | 配置名称 |
| response.data[].app_id | string | 应用ID |
| response.data[].app_name | string | 应用名称 |
| response.data[].app_client_id | string | 应用客户端ID |
| response.data[].user_id | string | 用户ID |
| response.data[].setting_id | string | 设置ID |
| response.data[].setting_key | string | 设置键名 |
| response.data[].setting_name | string | 设置名称 |
| response.data[].tpl_key | string | 模板键名 |
| response.data[].config_data.template_id | string | 模板ID |
| response.data[].config_data.template_map | string | 模板映射 |
| response.data[].change_time | string | 修改时间 |
| response.data[].change_user_id | string | 修改用户ID |
| response.total | string | 总记录数 |
| result.code | string | 响应代码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/system/sender/smser/tpl_config_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "id": null,
   "tpl":null,
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
        "app_client_id": "",
        "app_id": "0",
        "app_name": "",
        "change_time": "1749876525",
        "change_user_id": "7",
        "config_data": {
          "template_id": "adfad",
          "template_map": "adfad"
        },
        "id": "21",
        "name": "xeeexx",
        "setting_id": "9",
        "setting_key": "col-sms-config",
        "setting_name": "bbbbddddd",
        "tpl_key": "valid_code_reset_password_mobile",
        "user_id": "0"
      }
    ],
    "total": "11"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```