### rust实现的`面向内部应用的应用管理中心`实现

> 适用场景：
> 1. 当你使用rust开发web应用时，使用本项目二次开发可以减少重复工作量。
> 2. 当你需要一个包含oauth的轻量级开放平台服务时,可基于本项目快速实现鉴权，应用管理等.
> 3. 当进行分模块独立开发（如：公司内分团队的各自独立开发）时，可用本项目作为一个内部应用管理中心。
> 4. 当你希望把一些常用服务（如：发送短信，邮件 等）抽离出来公用时,可用本项目实现.

> 在线示例: http://175.178.90.181/ui/#

##### [后端服务](server) 安装

1. 编译 【中途有失败再次执行】
```
cd server && cargo build -r && cd ..
```

2. 导入SQL到数据库
```
导入 ./server/assets/tables.sql 到MYSQL
```

3. 修改你的监听端口,REDIS,MYSQL等
> [./server/examples/lsys-actix-web/.env 会覆盖./server/examples/lsys-actix-web/config/]

```
./server/examples/lsys-actix-web/.env 
```

4. 执行 启动服务
```
cd ./server/examples/lsys-actix-web/ && ../../target/release/lsys-actix-web
```


##### [管理UI](ui) 安装 

1. 修改配置,设置服务地址跟jwt的token等
```
./ui/web/config.json
./ui/wap/config.json
```

2. 编译
```
cd ./ui/web && npm i && npm run build && cd ../..
cd ./ui/wap && npm i && npm run build && cd ../..
cd ..
```

3. 访问 http://127.0.0.1/ui/

#### 二次开发参考时序:

##### 基于rust二次开发

> [基于rust二次开发示例](server/examples/lsys-web-subapp-demo/)

```mermaid
sequenceDiagram
    par 管理ui实现
    前端->>lsys服务: 前端请求到 lsys-actix-web[actix-web] 服务
    lsys服务-->>前端: 返回结果
    end
    par 自行开发RUST内部应用实现
    前端->>lsys服务: 前端请求到 lsys-actix-web[actix-web] 服务
    lsys服务->>内部rust实现: 参考示例应用[lsys-web-subapp-dome]
    内部rust实现-->>lsys服务: 可以库调用访问lsys服务[lsys-rbac,lsys-user等]
    lsys服务-->>前端: 返回结果
    end
```


##### 基于HTTP接口方式二次开发

> [通过HTTP接口接入文档](sdk/) 

```mermaid
sequenceDiagram
    par 普通应用实现[接入层处理业务逻辑]
    前端->>外部应用接入层: 自行选择语言,实现方式等
    外部应用接入层->>lsys服务: 通过以上协议 访问lsys提供的服务来实现部分你需要的功能
    lsys服务-->>外部应用接入层: 处理结果返回到外部应用
    外部应用接入层-->>前端: 返回结果
    end
    par 拆分接入层跟服务层的实现[服务层处理业务逻辑]
    前端->>外部应用接入层: 自行选择语言,实现方式等
    外部应用接入层->>外部应用服务层:自行选择语言,内部访问协议等,最好接入层 APPID 并传入
    外部应用服务层->>lsys服务: 根据接入层传入的APPID,查询对应的APP信息
    lsys服务-->>外部应用服务层: 可根据查询到的 APP信息 
    外部应用服务层->>外部应用服务层: 根据查询app信息及接入层传入APPID及其他参数 进行签名校验
    外部应用服务层->>外部应用服务层: 校验请求通过后，继续完成内部服务层业务逻辑
    外部应用服务层-->>外部应用接入层: 自行实现协议返回数据到接入层
    外部应用接入层-->>前端: 返回结果
    end
```
