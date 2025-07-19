### 地区搜索

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| key_word | string | 是 | 搜索关键词 |
| limit | int | 否 | 限制返回数量 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.area | array | 地区信息数组 |
| response.area[].code | string | 地区编码 |
| response.area[].leaf | string | 是否叶子节点(0:否,1:是) |
| response.area[].name | string | 地区名称 |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/area/search
Content-Type: application/json

{
    "key_word":"meizhou",
    "limit":"1"
}
```

```json
{
  "response": {
    "area": [
      [
        {
          "code": "44",
          "leaf": "0",
          "name": "广东省"
        },
        {
          "code": "4414",
          "leaf": "0",
          "name": "梅州市"
        }
      ]
    ]
  },
  "result": {
    "code": "200",
    "message": "ok",
    "state": "ok"
  }
}
```