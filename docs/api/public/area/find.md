### 根据行政区域编码查询地区列表

> 请求参数

 | 参数名 | 类型 | 必填 | 说明 |
 |--------|------|------|------|
 | code | string | 是 | 行政区域编码 |

> 响应参数

 | 参数名 | 类型 | 说明 |
 |--------|------|------|
 | response.area | array | 地区列表 |
 | response.area[].code | string | 行政区域编码 |
 | response.area[].leaf | string | 是否叶子节点(0:否,1:是) |  
 | response.area[].name | string | 地区名称 |
 | result.code | string | 状态码 |
 | result.message | string | 状态信息 |
 | result.state | string | 状态(ok:成功) |

> 示例

```http
POST /api/area/find
Content-Type: application/json

{
    "code": "110101001"
}
```

```json
{
  "response": {
    "area": [
      {
        "code": "11",
        "leaf": "0",
        "name": "北京市"
      },
      {
        "code": "1101",
        "leaf": "0",
        "name": "市辖区"
      },
      {
        "code": "110101",
        "leaf": "0",
        "name": "东城区"
      },
      {
        "code": "110101001",
        "leaf": "0",
        "name": "东华门街道"
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