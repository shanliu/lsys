### 服务层实现

> 执行 sql_merge.[sh|cmd] 创建汇总各模块使用到的SQL语句:tables.sql 后导入到mysql中.

> 服务层依赖: mysql+redis 在 ./examples/lsys-actix-web/.env 配置

> 使用示例[examples]目前基于 actix-web 作为接入web框架，你可选择其他基于 tokio 的web框架

### 编译&&启动

1. 导入SQL到数据库
```
./sql_merge.[sh|cmd] #创建tables.sql
mysql -u your_username -p your_database < ./tables.sql #导入SQL到mysql
```

2. 修改你的REDIS,MYSQL,jwt token等配置
> [./examples/lsys-actix-web/.env 的配置会覆盖./examples/lsys-actix-web/config/下配置]

```
./examples/lsys-actix-web/.env 
```

3. 启动开发环境
```
cd ./examples/lsys-actix-web/ && cargo run #cargo run -r 
```
