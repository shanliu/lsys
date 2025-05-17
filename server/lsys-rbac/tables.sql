CREATE TABLE `yaf_rbac_role` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint unsigned NOT NULL COMMENT '用户ID',
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID,当user_id时,对应关联的应用ID',
    `role_key` varchar(32) NOT NULL COMMENT '角色KEY,可用于会话角色鉴权时的标识',
    `user_range` tinyint NOT NULL COMMENT '角色包含用户范围',
    `res_range` tinyint NOT NULL COMMENT '角色可操作资源范围',
    `role_name` varchar(32) NOT NULL COMMENT '角色名,可为空',
    `status` tinyint NOT NULL COMMENT '状态',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更新用户',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    PRIMARY KEY (`id`),
    KEY `rbac_role_key_IDX` (
        `user_id`,
        `app_id`,
        `role_key`
    ) USING BTREE,
    KEY `rbac_role_user_range_IDX` (
        `user_range`,
        `res_range`,
        `status`
    ) USING BTREE,
    KEY `rbac_role_role_key_IDX` (
        `user_id`,
        `status`,
        `role_key`
    ) USING BTREE,
    KEY `rbac_role_app_IDX` (
        `app_id`
    ) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '角色';
CREATE TABLE `yaf_rbac_res` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint unsigned NOT NULL COMMENT '用户ID',
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID,当user_id时,对应关联的应用ID',
    `res_type` varchar(32) NOT NULL COMMENT '资源类型',
    `res_data` varchar(32) NOT NULL COMMENT '资源数据',
    `res_name` varchar(32) NOT NULL COMMENT '资源名,可为空',
    `status` tinyint NOT NULL COMMENT '状态',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更新用户',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    PRIMARY KEY (`id`),
    KEY `rbac_res_key_IDX` (
        `user_id`,
        `app_id`,
        `res_type`,
        `res_data`
    ) USING BTREE,
    KEY `rbac_res_type_IDX` (
        `res_type`,
        `user_id`,
        `status`
    ) USING BTREE,
    KEY `rbac_role_app_IDX` (
        `app_id`
    ) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '资源';
CREATE TABLE `yaf_rbac_op` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint unsigned NOT NULL COMMENT '用户ID',
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID,当user_id时,对应关联的应用ID',
    `op_key` varchar(32) NOT NULL COMMENT '资源操作KEY',
    `op_name` varchar(32) NOT NULL COMMENT '资源操作名,可为空',
    `status` tinyint NOT NULL COMMENT '状态',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更新用户',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    PRIMARY KEY (`id`),
    KEY `rbac_op_IDX` (
        `user_id`,
        `app_id`,
        `op_key`
    ) USING BTREE,
    KEY `rbac_role_app_IDX` (
        `app_id`
    ) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '资源操作';
CREATE TABLE `yaf_rbac_op_res` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `res_type` varchar(32) NOT NULL COMMENT '资源类型',
    `user_id` bigint unsigned NOT NULL COMMENT '用户ID',
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID,当user_id时,对应关联的应用ID',
    `op_id` bigint unsigned NOT NULL COMMENT '资源操作ID',
    `status` tinyint NOT NULL COMMENT '状态',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更新用户',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    PRIMARY KEY (`id`),
     KEY `rbac_res_type_IDX` (
        `res_type`,
        `user_id`,
        `app_id`,
        `op_id`,
        `status`
    ) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '操作跟资源关联';

CREATE TABLE `yaf_rbac_perm` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `role_id` bigint unsigned NOT NULL COMMENT '角色ID',
    `res_id` bigint unsigned NOT NULL COMMENT '资源ID',
    `op_id` bigint unsigned NOT NULL COMMENT '资源操作ID',
    `status` tinyint NOT NULL COMMENT '状态',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更新用户',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    PRIMARY KEY (`id`),
    KEY `rbac_role_op_IDX` (
        `role_id`,
        `res_id`,
        `op_id`,
        `status`
    ) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '角色关联的资源操作';

CREATE TABLE `yaf_rbac_role_user` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `role_id` bigint unsigned NOT NULL COMMENT '角色ID',
    `user_id` bigint unsigned NOT NULL COMMENT '用户ID',
    `timeout` bigint unsigned NOT NULL COMMENT '角色关联用户超时',
    `status` tinyint NOT NULL COMMENT '状态',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更新用户',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    PRIMARY KEY (`id`),
    KEY `rbac_role_user_IDX` (`role_id`, `user_id`, `status`,`timeout`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '角色关联的用户';


CREATE TABLE `yaf_rbac_audit` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint unsigned NOT NULL COMMENT '访问用户ID',
    `role_key_data` varchar(512) NOT NULL DEFAULT '' COMMENT '授权时指定角色列表',
    `check_result` tinyint NOT NULL  COMMENT '授权检查结果',
    `token_data` varchar(64) NOT NULL DEFAULT '' COMMENT '授权token',
    `user_ip` varchar(46) NOT NULL DEFAULT '' COMMENT '登陆者IP',
    `device_id` varchar(64) NOT NULL DEFAULT '' COMMENT '设备ID',
    `device_name` varchar(255) NOT NULL DEFAULT '' COMMENT '设备名',
    `request_id` varchar(64) NOT NULL DEFAULT '' COMMENT '请求id',
    `add_time` bigint unsigned NOT NULL COMMENT '检测时间',
    PRIMARY KEY (`id`),
    KEY `rbac_log_IDX` (
        `user_id`
    ) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '授权审计记录';


CREATE TABLE `yaf_rbac_audit_detail` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `rbac_audit_id` bigint unsigned NOT NULL COMMENT '审计ID',
    `res_type` varchar(32) NOT NULL COMMENT '资源类型',
    `res_data` varchar(32) NOT NULL COMMENT '资源数据',
    `res_user_id` bigint unsigned NOT NULL COMMENT '资源用户ID',
    `op_key` varchar(32) NOT NULL COMMENT '资源操作KEY',
    `res_id` bigint unsigned NOT NULL COMMENT '资源ID',
    `op_id` bigint unsigned NOT NULL COMMENT '操作ID',
    `check_result` tinyint NOT NULL  COMMENT '授权检查结果',
    `is_self` tinyint NOT NULL  COMMENT '是否自身角色',
    `is_root` tinyint NOT NULL  COMMENT '是否超级用户角色',
    `is_role_excluce` tinyint NOT NULL  COMMENT '是否被屏蔽',
    `is_role_include` tinyint NOT NULL  COMMENT '是否单独授权',
    `is_role_all` tinyint NOT NULL  COMMENT '是否全局授权结果',
    `role_data` varchar(1024) NOT NULL COMMENT '匹配的角色信息',
    `add_time` bigint unsigned NOT NULL COMMENT '检测时间',
    PRIMARY KEY (`id`),
    KEY `rbac_log_IDX` (
        `rbac_audit_id`
    ) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '授权审计详细';
