### rust实现的`面向内部应用的应用管理中心`实现

> 目标：轻量级 统一用户登陆,站内通用应用管理(如：发送短信，邮件 等) 的实现。

> 适用场景：
> 1. 当你需要一个包含oauth的轻量级开放平台服务时，可以用此项目快速搭建
> 2. 当你使用rust开发web应用时，使用本项目二次开发可以减少重复工作量，部份功能(如：发送短信，邮件 等)的开箱即用。
> 3. 当进行分模块独立开发（如：公司内分团队的各自独立开发）时，可用本项目作为一个内部应用管理中心，及常用功能(如：发送短信，邮件 等)的统一管理。


##### [后端服务](server) 安装

1. 编译 【中途有失败再次执行】
```
cd server && cargo build -r && cd ..
```

2. 导入SQL到数据库
```
导入 ./server/assets/tables.sql 到MYSQL
```

3. 修改你的REDIS,MYSQL等的配置
> [./server/examples/lsys-actix-web/.env 的配置覆盖./server/examples/lsys-actix-web/config/下配置]

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

4. 在线示例：http://175.178.90.181/ui/#

```
示例账号：aaaaa
示例密码：000000
```

#### 二次开发参考时序:

##### 基于rust二次开发

> [可参考示例:server/examples/lsys-web-subapp-demo](server/examples/lsys-web-subapp-demo/)

> 基础类功能建议用此方式

```mermaid
sequenceDiagram
    par 管理ui实现
    前端->>lsys服务: 前端请求到 lsys-actix-web[actix-web] 服务
    lsys服务-->>前端: 返回结果
    end
    par 自行开发RUST内部应用实现
    前端->>lsys服务: 前端请求到 lsys-actix-web[actix-web] 服务
    lsys服务->>内部rust实现: 参考示例应用[lsys-web-subapp-dome]
    内部rust实现-->>lsys服务: 在 内部应用 中可以库方式访问lsys服务
    lsys服务-->>前端: 返回结果
    end
```


##### 基于HTTP接口方式接入

> 通过HTTP接口接入 [go sdk](sdk/go/) 业务类功能建议用此方式

> GO实现外部应用示例 参考 [基本使用](sdk/go/examples/basic/)  && [外部应用服务层](sdk/go/examples/sub_app/)

> 备注:应用信息查询接口使用了本地 LruCache 缓存

> 备注:[rest服务层接口代码路径:server/examples/lsys-actix-web/src/handler/rest](server/examples/lsys-actix-web/src/handler/rest)

```mermaid
sequenceDiagram
    par 直接提供对外服务实现
    前端->>外部应用: 自行选择语言,实现方式等
    外部应用->>lsys服务: 通过以上协议 访问lsys提供的服务来实现部分你需要的功能
    lsys服务-->>外部应用: 处理结果返回到外部应用
    外部应用-->>前端: 返回结果
    end
    par 分隔接入层跟服务层实现
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
