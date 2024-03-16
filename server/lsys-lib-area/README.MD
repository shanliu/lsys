
### 5级行政区域数据

> Rust 实现 省市县镇村 数据查询 名称模糊搜索 及坐标对应城市搜索

> [c的ffi示例](wrappers/c_dome/main.c)，可以通过ffi转接到其他语言上,示例:[php扩展](wrappers/php)

#### 包含以下数据：

1. 五级行政区域数据库: [sqlite数据,需解压](https://github.com/shanliu/lsys/releases/download/v0.0.0/area-data.db.zip) [csv数据](https://github.com/shanliu/lsys/releases/download/v0.0.0/2023-7-area-code.csv.gz)

2. 县区级经纬度坐标数据 [sqlite数据,需解压](https://github.com/shanliu/lsys/releases/download/v0.0.0/area-data.db.zip) [csv数据](https://github.com/shanliu/lsys/releases/download/v0.0.0/2023-7-area-geo.csv.gz)


#### 基于 axum 的示例
```
cd examples/axum ; cargo run
#测试接口
curl 'http://127.0.0.1:8081/area/geo?lat=22.57729&lng=113.89409'
curl http://127.0.0.1:8081/area/list?code=1101
curl http://127.0.0.1:8081/area/related?code=4414
curl http://127.0.0.1:8081/area/find?code=4414
curl http://127.0.0.1:8081/area/search?key_word=%E6%B7%B1%E5%9C%B3%20%E5%B8%83%E5%90%89
```


##### 代码示例:

1. 请先下载CSV数据文件到你的项目：[城市数据示例](https://github.com/shanliu/lsys/releases/download/v0.0.0/2023-7-area-code.csv.gz) [城市坐标示例](https://github.com/shanliu/lsys/releases/download/v0.0.0/2023-7-area-geo.csv.gz) 

> csv数据源使用示例

```toml
#cargo.toml 默认仅引入:data-csv 
lsys-lib-area = { version = "~0.1.*"}
```

```rust
let code_path = PathBuf::from("data/2023-7-area-code.csv.gz");
let geo_path = PathBuf::from("data/2023-7-area-geo.csv.gz");
let data = lsys_lib_area::CsvAreaData::new(
    lsys_lib_area::CsvAreaCodeData::from_inner_path(code_path, true).unwrap(),
    Some(lsys_lib_area::CsvAreaGeoData::from_inner_path(geo_path, true).unwrap()),
);
let area = lsys_lib_area::AreaDao::from_csv_mem(data, AreaStoreMemory::default()).unwrap();
//使用文件索引，减少内存使用，接口速度比mem，但省内存
// let area = lsys_lib_area::AreaDao::from_csv_disk(data, AreaStoreDisk::new(PathBuf::from("./tmp"), None).unwrap()).unwrap();
```

2. 请先下载sqlite数据到你的项目 [内置的包含5级行政区域跟坐标的sqlite数据，需解压](https://github.com/shanliu/lsys/releases/download/v0.0.0/area-data.db.zip) 

> 如果你已经装了lib-sqlite的库，可用:`data-sqlite` 会使用系统的sqlite库. 具体参见crate:`rusqlite`实现

> 如果你未安装lib-sqlite的库，可先执行`sqlite_source.[cmd|sh]`下载sqlite源码后用:`data-sqlite-source` 编译.

> sqlite数据源使用示例

```toml
#cargo.toml
lsys-lib-area = { version = "~0.1.*", default-features=false,features=["data-sqlite-source"]}
```

```rust
//下载 area-data.db.zip 解压后得到 area-data.db
let conn = "data/area-data.db";
let sqlite = lsys_lib_area::SqliteAreaData::new(
    lsys_lib_area::SqliteAreaCodeData::from_path(PathBuf::from(&conn)),
    Some(lsys_lib_area::SqliteAreaGeoData::from_path(PathBuf::from(&conn))),
);

let area = lsys_lib_area::AreaDao::from_sqlite_mem(sqlite, AreaStoreMemory::default()).unwrap();
//使用文件索引，减少内存使用，接口速度比mem，但省内存
// let area = lsys_lib_area::AreaDao::from_sqlite_disk(sqlite, AreaStoreDisk::new(PathBuf::from("./tmp"), None).unwrap()).unwrap();
```

3. 使用mysql作为数据来源：[MYSQL数据](https://github.com/shanliu/lsys/releases/download/v0.0.0/mysql-table.sql.zip) 解压后导入数据库

> mysql数据库作为数据源的示例

```toml
#cargo.toml
lsys-lib-area = { version = "~0.1.*", default-features=false,features=["data-mysql"]}
```

```rust
let pool = "mysql://***:***@127.0.0.*:3306/***";
let mysql = lsys_lib_area::MysqlAreaData::new(
    lsys_lib_area::MysqlAreaCodeData::from_uri(pool),
    Some(lsys_lib_area::MysqlAreaGeoData::from_uri(pool)),
);
let area = lsys_lib_area::AreaDao::from_mysql_mem(mysql, AreaStoreMemory::default()).unwrap();
//使用文件索引，减少内存使用，接口速度比mem，但省内存
// let area = lsys_lib_area::AreaDao::from_mysql_disk(mysql, AreaStoreDisk::new(PathBuf::from("./tmp"), None).unwrap()).unwrap();
```

4. 数据更新时重新加载数据

```rust
let area = lsys_lib_area::AreaDao::from_*(...).unwrap();
area.geo_reload().unwrap();//重新加载GEO数据
area.code_reload().unwrap();//重新加载CODE数据
```

5. FFI导出

> 默认不引入 `lib-clib` ,需要FFI请开启
```
cargo build --features "lib-clib" 
```
```toml
#修改 cargo.toml 添加 lib-clib
lsys-lib-area = { version = "~0.1.*",features=["lib-clib"]}
```


##### 可用的查询方法

1. 查询 省市县镇村 可用在地址选择多级联动接口

```rust
let child="";//空列出省级，把省级的code转入列出市级..依次完成
let res = area.code_childs(child).unwrap();
println!("{}", &res);
```

2. 查询 指定code 的相关数据

```rust
let child="441403133";//地址code
let res = area.code_related(child).unwrap();
println!("{}", &res);
```

3. 查询 指定code 的详细地址信息

> 可用于系统内的code到地址转换

```rust
let child="441403133";//地址code
let res = area.code_find(child).unwrap();
println!("{}", &res);
```

4. 搜索任意地址，返回匹配的地址信息

```rust
let child="guang dong";//地址信息 在比如: 广东 布吉
let limit = 10;//返回匹配数量
let res = area.code_search(child, limit).unwrap();
println!("{}", &res);
```

5. 返回指定坐标的地址信息

> 可用手机根据geo信息自动填写当前位置收货地址

> 目前到市一级，因为只找到了市一级的坐标数据

```rust
let res = area.geo_search(  26.61474,  114.13548 , ).unwrap();
println!("{}", &res);
```

