# 新增条形码配置

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| app_id | int | 是 | 应用ID |
| barcode_type | string | 是 | 条码类型(qrcode) |
| status | int | 是 | 状态(1-启用) |
| image_format | string | 是 | 图片格式(png) |
| image_width | int | 是 | 图片宽度(像素) |
| image_height | int | 是 | 图片高度(像素) |
| margin | int | 是 | 边距 |
| image_color | string | 是 | 前景色(十六进制颜色值) |
| image_background | string | 是 | 背景色(十六进制颜色值) |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.id | string | 配置ID |
| result.code | string | 状态码 |
| result.message | string | 状态信息 |
| result.state | string | 状态标识 |

> 示例

```http
POST /api/user/app_barcode/create_config_add
Content-Type:application/json
Authorization:Bearer {{APP_BEARER_TEST_ACCOUNT}}

{
    "app_id": 1,
    "barcode_type": "qrcode",
    "status":2,
    "image_format": "png",
    "image_width": 100,
    "image_height": 100,
    "margin": 1,
    "image_color":"#000000",
    "image_background":"#ffffff"
}
```

```json
{
  "response": {
    "id": "1"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```