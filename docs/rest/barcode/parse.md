
### 二维码解析

> payload参数

| 参数 | 类型 | 是否必填 | 描述 |
|------|------|----------|------|
| try_harder | string | 否 | 是否获取header信息 |

> 响应参数

| 参数 | 类型 | 描述 |
|------|------|------|
| response.record.data.hash | string | 图片哈希值 |
| response.record.data.position.x | float | 二维码定位点X坐标 |
| response.record.data.position.y | float | 二维码定位点Y坐标 |
| response.record.data.text | string | 二维码内容 |
| response.record.data.type | string | 码类型 |
| response.record.status | string | 记录状态 |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |


> 解析示例图

![二维码解析示例图片](./parse_pic.png)

> 示例

```http
POST /rest/barcode?method=parse&&payload={"try_harder":true}
Content-type: multipart/form-data; boundary=WebKitFormBoundary
--WebKitFormBoundary
Content-Disposition: form-data; name="aa"; filename="parse_pic.png"
Content-type: image/png

< ./parse_pic.png
--WebKitFormBoundary--
```

```json
{
  "response": {
    "record": [
      {
        "data": {
          "hash": "0585adee43445c28000a07f9aea0bab6f4649791298027a9217ba59931e61da2",
          "position": [
            {
              "x": "8.5",
              "y": "8.5"
            },
            {
              "x": "92.5",
              "y": "8.5"
            },
            {
              "x": "92.5",
              "y": "92.5"
            },
            {
              "x": "8.5",
              "y": "92.5"
            }
          ],
          "text": "DD12123DD",
          "type": "qrcode"
        },
        "status": "1"
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