### 根据行政区域编码下级区域列表

> 请求参数

| 参数名 | 类型 | 是否必须 | 说明 |
|--------|------|----------|------|
| code | string | 是 | 行政区域编码 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| response.area | array | 区域列表 |
| response.area[].code | string | 区域编码 |
| response.area[].leaf | string | 是否是叶子节点(0:否,1:是) |
| response.area[].name | string | 区域名称 |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/area/list
Content-Type: application/json

{
    "code": "11"
}
```

```json
{
  "response": {
    "area": [
      {
        "code": "1101",
        "leaf": "0",
        "name": "市辖区"
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