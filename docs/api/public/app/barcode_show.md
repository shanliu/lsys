### 显示二维码

> 请求参数 `/barcode/{content_type}/{code_id}/{content_data}`

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| content_type | string | 是 | 二维码内容类型:text;base64 |
| code_id | int | 是 | 二维码配置ID,如示例:1 |  
| content_data | string | 是 | 二维码内容,内容长度参考app.toml配置:barcode_create_max |

> 响应内容 `二维码图片`

> 示例

```http
GET /barcode/text/2/DDDDD
```
