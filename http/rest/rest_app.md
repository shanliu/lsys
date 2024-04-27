## 应用接口文档

> 接入方法及公共参数,可参考[接入文档](rest.md)

> [go代码示例](https://github.com/shanliu/lsys/blob/main/sdk/go/examples/basic/app_test.go)

### 查询子应用信息

> payload参数说明:

| 参数         | 类型      | 是否必填   | 描述    |
|-------------|-----------|------------|-----------------------------------|
| client_id     | String  | 是	       | 应用KEY                           |

> 接口出参说明:

| 参数         | 类型      | 是否可选   | 描述               |
|-------------|-----------|-----------|-------------------|
| client_id   | String	 | 是	  | 应用ID                 |

> 示例:
```http
POST /rest/subapp?method=info
Content-type:application/json

{
    "client_id": "sub"
}
```

