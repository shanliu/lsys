## 文档模块

> clone git 到本地，按规则删除非文档文件 在根据目录文件配置 生产站点文档页面

### 目录文件格式说明

> 目录文件可以多个，在后台添加绑定

> 目录文件根据相同节点id相同时进行子节点合并,id不同时保留两份

> path 路径 为相对 目录文件 的路径 使用/路径时为该源的根目录。`当markdown文档存在连接或图片时需要注意`

```json
{
    "//":"//为注释，兼容json写法，可以删除,以下为各字段说明:",
    "//":"id[可选] 当多个目录在相同节点位置id相同时会进行合并",
    "id": "v1",
    "//":"name[必须] 用于显示文档目录或文件名称",
    "name": "Oauth接入文档(v1)",
    "//":"title[可选] 目录或文件名提示信息",
    "title": "Oauth接入文档及示例",
    "//":"children 或 path 必须存在一个, 存在 children 时为目录 存在 path 时为文档",
    "children": [
        {
            "//":"同上说明，参见上面的说明",
            "name": "接入说明",
            "//":"同上说明，参见上面的说明",
            "path": "README.md"
        },
        {
            "//":"同上说明，参见上面的说明",
            "name": "参数说明",
            "//":"同上说明，参见上面的说明",
            "path": "pa/README1.md"
        }
    ]
}
```






