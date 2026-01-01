### 二维码创建

> payload参数

| 参数      | 类型     | 是否必填 | 描述         |
|-----------|---------|----------|--------------|
| contents  | string  | 是      | 生成二维码内容 |
| code_id   | int     | 是      | 二维码应用ID  |

> 响应参数

| 参数                 | 类型    | 描述          |
|---------------------|---------|---------------|
| response.data       | string  | 图片JSON64数据 |
| response.type       | string  | 图片类型       |
| result.code         | string  | 状态码        |
| result.message      | string  | 响应消息      |
| result.state        | string  | 响应状态      |

> 示例

```http
POST /rest/barcode?method=create
Content-type:application/json

{
    "contents": "xxx",
    "code_id":2
}
```

```json
{
  "response": {
    "data": "图片JSON64数据",
    "type": "image/jpeg"
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```