### 条形码解析记录列表

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | string | 否 | 应用ID |
| barcode_type | string | 否 | 条码类型 |
| count_num | boolean | 否 | 是否返回总数 |
| page.page | int | 是 | 页码 |
| page.limit | int | 是 | 每页条数 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data | array | 数据列表 |
| response.data.app_id | string | 应用ID |
| response.data.bar_type | string | 条码类型 |
| response.data.create_time | int | 创建时间(秒) |
| response.data.error | string | 错误信息 |
| response.data.hash | string | 哈希值 |
| response.data.id | string | 记录ID |
| response.data.status | string | 状态 |
| response.data.text | string | 条码内容 |
| response.total | string | 总记录数 |
| result.code | string | 状态码 |
| result.message | string | 消息 |
| result.state | string | 状态 |
> 示例

```http
POST /api/user/app_barcode/parse_record_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
   "app_id":null,
   "barcode_type":null,
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
        "app_id": "16",
        "bar_type": "qrcode",
        "create_time": "1749866873",
        "error": "",
        "hash": "0585adee43445c28000a07f9aea0bab6f4649791298027a9217ba59931e61da2",
        "id": "2",
        "status": "1",
        "text": "DD12123DD"
      }
    ],
    "total": "1"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```