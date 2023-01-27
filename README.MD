### 一个基于actix-web 的用户中心



##### 后端服务安装

1. 编译 【中途有失败再次执行】
```
cargo build -r
```

2. 导入SQL到数据库
```
导入 ./assets/tables.sql 到MYSQL
```

3. 修改配置,设置你的REDIS,MYSQL等
> [./lsys-actix-web/.env 的配置覆盖/lsys-actix-web/config/下配置]

```
./lsys-actix-web/.env 
```

4. 执行 启动服务
```
cd ./lsys-actix-web/ && ../target/release/lsys-actix-web
```


##### 管理UI安装

> git: https://github.com/shanliu/lsys-ui

1. 编译
```
cd web && npm i && npm run build
```

2. 修改配置,设置服务域名等
```
./web/config.json
```

3. 拷贝编译文件到lsys-actix-web静态文件目录中,也可以单独部署，具体配置为：app.toml/ui_dir
```
cp ./public/* ../lsys-actix-web/static/ui
```

4. 访问 http://127.0.0.1/ui/


##### 在线示例：
```
http://175.178.90.181/ui/#
示例账号：aaaaa
示例密码：000000
```

