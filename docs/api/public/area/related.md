### 区域关联查询接口

> 请求参数

| 参数名 | 类型 | 是否必须 | 说明 |
|--------|------|----------|------|
| code   | string | 是 | 区域编码 |

> 响应参数

| 参数名 | 类型 | 说明 |
|--------|------|------|
| area | array | 区域数组，二维数组，第一层为省份列表，第二层为选中省份的城市列表 |
| area[].code | string | 区域编码 |
| area[].leaf | string | 是否是叶子节点，0-否，1-是 |
| area[].name | string | 区域名称 |
| area[].selected | string | 是否选中，0-否，1-是 |
| result.code | string | 响应状态码 |
| result.message | string | 响应消息 |
| result.state | string | 响应状态 |

> 示例

```http
POST /api/area/related
Content-Type: application/json

{
    "code": "44"
}
```

```json
{
  "response": {
    "area": [
      [
        {
          "code": "11",
          "leaf": "0",
          "name": "北京市",
          "selected": "0"
        },
        {
          "code": "44",
          "leaf": "0",
          "name": "广东省",
          "selected": "1"
        },
         {
          "code": "..",
          "leaf": "0",
          "name": "..其他省略",
          "selected": "0"
        }
      ],
      [
        {
          "code": "4401",
          "leaf": "0",
          "name": "广州市",
          "selected": "0"
        },
         {
          "code": "..",
          "leaf": "0",
          "name": "..其他省略",
          "selected": "0"
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