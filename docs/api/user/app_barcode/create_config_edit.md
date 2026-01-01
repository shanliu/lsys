### 修改条形码配置

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| id | int | 是 | 配置ID |
| barcode_type | string | 是 | 码类型(qrcode) |
| status | int | 是 | 状态(1启用,0禁用) |
| image_format | string | 是 | 图片格式(jpg) |
| image_width | int | 是 | 图片宽度 |
| image_height | int | 是 | 图片高度 |
| margin | int | 是 | 边距 |
| image_color | string | 是 | 前景色(十六进制) |
| image_background | string | 是 | 背景色(十六进制) |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/user/app_barcode/create_config_edit
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "id": 2,
    "barcode_type": "qrcode",
    "status":1,
    "image_format": "jpg",
    "image_width": 200,
    "image_height": 200,
    "margin": 1,
    "image_color":"#663300",
    "image_background":"#003300"
}
```

```json
{
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```