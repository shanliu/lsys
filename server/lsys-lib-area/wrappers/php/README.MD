
#### 编译前准备动态库

> 扩展兼容PHP >=7.0


#### linux or mac 下编译

> 可选编译参数

```
--with-lib_area_dir=./lib 自定义上面编译的so文件路径
--with-lib_area_use_mysql 启用mysql数据源支持,需开启cargo.toml中的 features data-mysql 
--with-lib_area_use_sqlite 启用sqlite数据源支持,需开启cargo.toml中的 features data-sqlite
```

> 执行编译,具体内容参见以下脚本:

```shell
./build.sh 
```


#### windows 下使用


> 使用PHP>8.0,可下载编译版本[Tag:v*.*.*-php-lib-area-ext lsys_lib_area_php_ext.zip](https://github.com/shanliu/lsys/releases/)

> !!!注意!!!: 需要把下载文件中的lsys_lib_area.dll 拷贝到:apache 的bin目录或php.exe(fastcgi)的目录

> 根据版本拷贝php_lib_area_\*.dll 到php的扩展`ext/`目录

> 编译参数说明
```
--enable-lib_area_use_mysql 启用mysql数据源支持,需开启cargo.toml中的 features data-mysql 
--enable-lib_area_use_sqlite 启用sqlite数据源支持,需开启cargo.toml中的 features data-sqlite
```

> 如果已经安装VS,可通过以下脚本编译

```shell
./build.cmd
```


#### 测试示例

> 数据资源下载：`https://github.com/shanliu/lsys/releases/tag/v0.0.0`

> [测试示例](./lib_area.api.php) [PHP类定义](./lib_area.stub.php)
