### rust实现的`内部应用开放平台`

> 目标：轻量级 统一用户登陆,站内通用应用管理(如：发送短信，邮件，地址库等) 的实现，减少功能重复开发量

> 第三方服务依赖为: mysql+redis 在 ./examples/lsys-actix-web/.env 配置

> 目前使用示例[examples]基于 actix-web 作为接入web框架，你可选择其他基于 tokio 的web框架

> 已实现对外oauth服务，对内用户对接方案[已存在系统快速对接]思考跟实现中...

##### 后端服务安装

1. 编译 【中途有失败再次执行】
```
cargo build -r
```

2. 导入SQL到数据库
```
导入 ./assets/tables.sql 到MYSQL
```

3. 修改你的REDIS,MYSQL等的配置
> [./examples/lsys-actix-web/.env 的配置覆盖./examples/lsys-actix-web/config/下配置]

```
./examples/lsys-actix-web/.env 
```

4. 执行 启动服务
```
cd ./examples/lsys-actix-web/ && ../../target/release/lsys-actix-web
```


##### 管理UI安装

> git: https://github.com/shanliu/lsys-ui

1. 编译
```
cd web && npm i && npm run build && cd ..
cd wap && npm i && npm run build && cd ..
```

2. 修改配置,设置服务域名等
```
./web/config.json
```

3. 拷贝编译文件到lsys-actix-web静态文件目录中,静态文件目录配置在文件：./examples/lsys-actix-web/app.toml : ui_dir 中
> 备注: UI部分可以自行部署如NGINX服务器中
```
cp ./public/* ../examples/lsys-actix-web/static/ui
```

4. 访问 http://127.0.0.1/ui/

5. 在线示例：http://175.178.90.181/ui/#

```
示例账号：aaaaa
示例密码：000000
```


##### 外部应用实现示例[GO实现]

> git: https://github.com/shanliu/lsys-subapp

> 具体示例参见 lsys-subapp/examples/sub_app && lsys-subapp/examples/basic


#### 二次开发参考时序:

##### 内部应用:[基于RUST库方式二次开发]

>  直接访问:[lsys-ui 管理节目 实现此方式]

> 使用协议为 examples/lsys-actix-web/src/handler/api 实现的JWT协议

> 可基于 基于RUST库方式 开发 内部应用 [可参考examples/lsys-web-subapp-dome实现]

```mermaid
sequenceDiagram
    par lsys-ui 管理实现
    前端->>lsys服务: 前端请求到 lsys-actix-web[actix-web] 服务
    lsys服务-->>前端: 返回结果
    end
    par 自行开发RUST内部应用实现
    前端->>lsys服务: 前端请求到 lsys-actix-web[actix-web] 服务
    lsys服务->>内部应用: 参考示例应用[lsys-web-subapp-dome]
    内部应用-->>lsys服务: 在 内部应用 中可以库方式访问lsys服务
    lsys服务-->>前端: 返回结果
    end
```


##### 外部应用:[基于HTTP接口方式]

> 外部应用服务层 [通过HTTP接口应用,自行选择开发语言]

> 外部应用访问lsys服务 使用协议为 examples/lsys-actix-web/src/handler/rest 实现接口协议

> 应用可在 lsys-ui 后台实现管理

> lsys服务对应用进行了本地 LruCache ,所以 外部应用服务层 查询APP信息 自行判断是否在加缓存

> GO实现外部应用示例: https://github.com/shanliu/lsys-subapp

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
