doc-git-url-error = 提交的URL异常:{$msg}
doc-git-tmpdir-error = 建立临时目录异常:{$msg}
doc-task-error = 获取GIT数据异常:{$msg}
doc-git-name-empty = 提供GIT源名称
doc-git-version-not-find =  GIT源中未发现指定版本:{$tag}[${version}]
doc-git-not-find = GIT源已被删除
doc-git-rule-error = 清理规则[{$rule}]错误:{$msg}
doc-git-submit-version-error = 提交的版本号[{$version}]异常
doc-git-tag-empty = 请提供TAG名
doc-git-status-wrong = 提供的TAG[{$tag}]已被删除
doc-git-menu-empty = 指定TAG[{$tag}]未配置目录
doc-git-dir-access = 路径必须为[{$prefix}]子目录[{$host_name}]
doc-git-dir-error = 读取目录[{$host_name}:{$dir}]异常:{$msg}
doc-git-menu-read-not-yet = 指定主机[{$host_name}]的GIT未完成克隆
doc-git-menu-read-access = 目录[{$menu_file}]中路径[{$file_path}]非安全路径,触发主机[{$host_name}]
doc-git-menu-read-notfile = 目录[{$menu_file}]中路径[{$file_path}]非文件,触发主机[{$host_name}]
doc-git-menu-name-empty = 请提供目录名
doc-git-menu-file-error = 在{$tag}中读取目录{$file_path}异常:{$msg}
doc-git-menu-file-empty = 在{$tag}中读取目录{$file_path}内容为空
doc-git-menu-file-parse-error = 在{$tag}中读取目录{$file_path}内容无法解析为JSON:{$msg}
doc-git-menu-path-isfind =  在{$tag}中已存在目录:{$menu_path}  
doc-git-rule-encode-error=清理规则地址异常：{$msg}
doc-git-error=git异常:{$msg}
doc-menu-file-path-access = 无法访问读取路径:{$path}
doc-notify-channel-close = 发送同步任务时发生异常:{$msg}
doc-notify-call-fail = 通知节点继续删除时异常:{$msg}





# 状态

status-DocGitStatus-Enable=  启用
status-DocGitStatus-Delete= 删除




status-DocGitTagStatus-Publish= 已发布
status-DocGitTagStatus-Build=   已添加
status-DocGitTagStatus-Delete= 删除





status-DocGitCloneStatus-Init=    待克隆
status-DocGitCloneStatus-Cloned=  已克隆
status-DocGitCloneStatus-Fail=    克隆失败
status-DocGitCloneStatus-Delete= 删除





status-DocMenuStatus-Enable=  启用
status-DocMenuStatus-Delete= 删除



#校验名称

valid-rule-name-git_url = GIT文档地址
valid-rule-name-name = 文档名称
valid-rule-name-max_try = 最大尝试下载次数
valid-rule-name-git_status = GIT源状态
valid-rule-name-git_version = GIT版本编号
valid-rule-name-menu_path = 菜单路径