## 二维码接口文档

> 接入方法及公共参数,可参考[接入文档](rest.md)


### 二维码创建

> payload参数说明:

| 参数         | 类型      | 是否必填   | 描述    |
|-------------|-----------|------------|-----------------------------------|
| contents     | String  | 是	       | 生成二维码内容|
| code_id     | int  | 是	       | 二维码应用ID|


> 示例:
```http
POST /rest/barcode?method=create
Content-type:application/json

{
    "contents": "xxx",
    "code_id":1
}
```


### 二维码解析


> payload参数说明:

| 参数         | 类型      | 是否必填   | 描述    |
|-------------|-----------|------------|--------|
| try_harder  | String  | 否       | 是否获取header信息|


> 示例:
```http
POST /rest/barcode?method=parse&&payload={"try_harder":true}
Content-type: multipart/form-data; boundary=WebKitFormBoundary
--WebKitFormBoundary
Content-Disposition: form-data; name="aa"; filename="aa.png"
Content-type: image/png

< ./data/a.png
--WebKitFormBoundary--
```
