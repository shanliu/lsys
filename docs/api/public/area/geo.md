### 根据经纬度获取地区信息

> 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| lat | float | 是 | 纬度 |
| lng | float | 是 | 经度 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.area | array | 地区信息数组 |
| response.area[].code | string | 地区编码 |
| response.area[].leaf | string | 是否叶子节点(0:否,1:是) |
| response.area[].name | string | 地区名称 |
| result.code | string | 状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/area/geo
Content-Type: application/json

{
    "lat": 26.61474,
    "lng": 114.13548
}
```

```json
{
  "response": {
    "area": [
      {
        "code": "36",
        "leaf": "0",
        "name": "江西省"
      },
      {
        "code": "3608",
        "leaf": "0",
        "name": "吉安市"
      },
      {
        "code": "360881",
        "leaf": "0",
        "name": "井冈山市"
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