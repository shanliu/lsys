### 应用管理 服务层实现

> 服务层依赖: mysql+redis 在 ./examples/lsys-actix-web/.env 配置

> 使用示例[examples]目前基于 actix-web 作为接入web框架，你可选择其他基于 tokio 的web框架

> 已实现统一登陆的oauth服务，更多的通用应用实现中。。。


### 开发or编译

1. 导入SQL到数据库
```
导入 ./assets/tables.sql 到MYSQL
```

2. 修改你的REDIS,MYSQL,jwt token等配置
> [./examples/lsys-actix-web/.env 的配置会覆盖./examples/lsys-actix-web/config/下配置]

```
./examples/lsys-actix-web/.env 
```

3. 启动开发环境
```
cd ./examples/lsys-actix-web/ && cargo run 
#cargo build -r && cd .. #or build
```
