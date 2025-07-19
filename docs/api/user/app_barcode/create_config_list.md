### 条码配置列表

> 请求参数 

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| id | int | 否 | 配置ID |
| app_id | int | 否 | 应用ID |
| barcode_type | string | 否 | 条码类型 |
| count_num | boolean | 否 | 是否统计数量 |
| page.page | int | 是 | 页码 |
| page.limit | int | 是 | 每页条数 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.data.app_id | string | 应用ID |
| response.data.barcode_type | string | 条码类型 |
| response.data.change_time | int | 修改时间(秒) |
| response.data.id | string | 配置ID |
| response.data.image_background | string | 图片背景色 |
| response.data.image_color | string | 图片颜色 |
| response.data.image_format | string | 图片格式 |
| response.data.image_height | string | 图片高度 |
| response.data.image_width | string | 图片宽度 |
| response.data.margin | string | 边距 |
| response.data.status | string | 状态 |
| response.data.url | string | 链接地址 |
| response.total | string | 总数量 |
| result.code | string | 状态码 |
| result.message | string | 消息 |
| result.state | string | 状态 |

> 示例

```http
POST /api/user/app_barcode/create_config_list
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "id": null,
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
        "app_id": "1",
        "barcode_type": "qrcode",
        "change_time": "1749452090",
        "id": "2",
        "image_background": "#003300",
        "image_color": "#663300",
        "image_format": "jpg",
        "image_height": "200",
        "image_width": "200",
        "margin": "1",
        "status": "1",
        "url": ""
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