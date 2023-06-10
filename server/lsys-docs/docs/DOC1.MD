
##### git文档模块




构建:
读取 doc_git , 通过 id+build_version+host 读取 doc_clone 状态 未完成
$build_version 目录不存在，clone 开始 超过，写错误信息,重试[clone_try+1]
$build_version 目录存在，pull，失败，写错误信息，删除目录，在按不存在处理继续处理
按配置切换分支。失败写错误消息，停止
完成,更新 doc_clone 状态,按条件更新 doc_git 的 finish_time

根据文档目录表，读目录文件,判断文件是否存在，有错误写日志，添加 doc_build 记录[存在时更新]
完成
	非全部目录都是错误:按条件更新 doc_menu 的 finish_time[跟上面的添加 doc_build 放在一个事务]
	目录构建完成时通知所有节点更新

以上完成后清理:
	查 doc_clone: doc_git_id+host+id<当前+status=[已克隆] order by id desc 得到 build_version
	更新 doc_build:doc_git_id+host+build_version 及 查doc_clone:doc_git_id+host+build_version 为的STATUS已删除
	根据 build_version 删除目录
	
git更新
固定构建版本，忽略
非固定，更新 doc_git 的 build_version,执行构建


启动:
遍历  doc_git 当 finish_time 为0 ,执行构建



查看目录：
读 doc_menu status=[正常]
	finish_time>0 查  id=doc_menu_id+build_version+host+status=[部分完成,完成] union all
	过滤掉结果 + finish_time=0 为条件查:id=doc_menu_id+host+status=[部分完成,完成] order by id desc  union all
	合并,菜单重复,取大ID值,输出build_data结果


查看文件：构建记录id，访问路径路径[去除?.*] [RAW跟MD文件,MD文件返回JSON]
找到对应的记录数据，得到文件基础目录，加访问路径，整理路径，判断是否在安全目录,判断扩展名，读取文件，返回内容。header加版本




