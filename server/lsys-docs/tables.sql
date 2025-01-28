CREATE TABLE `yaf_doc_git` (
    `id` int unsigned NOT NULL AUTO_INCREMENT,
    `name` varchar(64) NOT NULL COMMENT '源名称',
    `url` varchar(254) NOT NULL COMMENT 'GIT地址,包含用户名',
    `status` tinyint NOT NULL COMMENT '状态:删除 正常',
    `max_try` tinyint unsigned NOT NULL COMMENT '尝试克隆次数',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后修改用户',
    `change_time` bigint unsigned NOT NULL COMMENT '最后修改时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '文档GIT源';
CREATE TABLE `yaf_doc_tag` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `doc_git_id` int unsigned NOT NULL COMMENT '文档GIT来源ID',
    `tag` varchar(64) NOT NULL COMMENT 'Tag',
    `build_version` varchar(64) NOT NULL COMMENT '当前使用构建版本',
    `clear_rule` varchar(512) NOT NULL DEFAULT "" COMMENT '清理路径表达式',
    `status` tinyint NOT NULL COMMENT '状态 删除 未启用 已启用',
    `add_user_id` bigint unsigned NOT NULL COMMENT '最后修改用户',
    `add_time` bigint unsigned NOT NULL COMMENT '最后修改时间',
    PRIMARY KEY (`id`),
    KEY `user_tag_IDX` (`doc_git_id`) USING BTREE,
    KEY `tag_IDX` (`tag`) USING BTREE,
    KEY `ver_IDX` (`build_version`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '文档GIT-TAG';
CREATE TABLE `yaf_doc_clone` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `doc_tag_id` bigint unsigned NOT NULL COMMENT '文档GIT tag ID',
    `host` varchar(255) NOT NULL COMMENT '克隆主机',
    `start_time` bigint unsigned NOT NULL COMMENT '最后CLONE开始时间',
    `finish_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '克隆完成时间',
    `status` tinyint NOT NULL COMMENT '状态:待克隆 已克隆 克隆失败 已删除',
    PRIMARY KEY (`id`),
    KEY `user_clone_IDX` (`doc_tag_id`, `host`, `status`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '文档GIT CLONE记录';
CREATE TABLE `yaf_doc_menu` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `doc_tag_id` bigint unsigned NOT NULL COMMENT '文档GIT tag ID',
    `menu_path` varchar(254) NOT NULL COMMENT '目录文件路径',
    `menu_check_host` varchar(255) NOT NULL COMMENT '天机是检测通过的机器',
    `status` tinyint NOT NULL COMMENT '状态 正常 删除 ',
    `add_user_id` bigint unsigned NOT NULL COMMENT '修改用户',
    `add_time` bigint unsigned NOT NULL COMMENT '修改时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '文档目录文件配置';
CREATE TABLE `yaf_doc_logs` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `doc_tag_id` bigint unsigned NOT NULL COMMENT '文档GIT tag ID',
    `doc_clone_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '文档GIT来源ID',
    `host` varchar(255) NOT NULL COMMENT '执行时主机名',
    `message` varchar(255) NOT NULL COMMENT '消息内容',
    `add_time` bigint unsigned NOT NULL COMMENT '最后修改时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '文档日志';