-- ----------- lsys-rbac  ---------------
CREATE TABLE `yaf_rbac_res` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint unsigned NOT NULL COMMENT '用户ID',
    `name` varchar(32) NOT NULL COMMENT '资源名',
    `res_key` varchar(32) NOT NULL COMMENT '资源KEY',
    `status` tinyint NOT NULL COMMENT '状态',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更新用户',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '资源';
CREATE TABLE `yaf_rbac_res_op` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `name` varchar(32) NOT NULL COMMENT '资源操作名',
    `op_key` varchar(32) NOT NULL COMMENT '资源操作KEY',
    `res_id` bigint unsigned NOT NULL COMMENT '资源ID',
    `status` tinyint NOT NULL COMMENT '状态',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更新用户',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '资源可用操作';
CREATE TABLE `yaf_rbac_role` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint unsigned NOT NULL COMMENT '用户ID',
    `name` varchar(32) NOT NULL COMMENT '角色名',
    `relation_key` varchar(32) NOT NULL COMMENT '指定关系角色KEY,其他角色类型为空',
    `user_range` tinyint NOT NULL COMMENT '角色包含用户范围',
    `res_op_range` tinyint NOT NULL COMMENT '角色可操作资源范围',
    `status` tinyint NOT NULL COMMENT '状态',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更新用户',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    `priority` tinyint DEFAULT 99,
    PRIMARY KEY (`id`),
    KEY `yaf_rbac_role_user_id_IDX` (
        `user_id`,
        `status`,
        `priority`,
        `id`
    ) USING BTREE,
    KEY `yaf_rbac_role_user_range_IDX` (
        `user_range`,
        `res_op_range`,
        `status`,
        `priority`,
        `id`
    ) USING BTREE,
    KEY `yaf_rbac_role_relation_key_IDX` (
        `user_id`,
        `status`,
        `user_range`,
        `relation_key`
    ) USING BTREE,
    KEY `yaf_rbac_role_priority_IDX` (`priority`, `id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '角色';
CREATE TABLE `yaf_rbac_role_op` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `res_op_id` bigint unsigned NOT NULL COMMENT '角色关联资源操作ID',
    `role_id` bigint unsigned NOT NULL COMMENT '角色ID',
    `positivity` tinyint NOT NULL COMMENT '授权类型: 授权操作 禁止操作',
    `status` tinyint NOT NULL COMMENT '状态',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更新用户',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    PRIMARY KEY (`id`),
    KEY `yaf_rbac_role_op_IDX` (
        `role_id`,
        `res_op_id`,
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
    KEY `yaf_rbac_role_user_IDX` (`role_id`, `user_id`, `status`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '角色关联的用户';
CREATE TABLE `yaf_rbac_tags` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `from_id` bigint unsigned NOT NULL COMMENT 'TAG关联来源 ID',
    `from_source` tinyint NOT NULL COMMENT 'TAG关联来源 res role',
    `user_id` bigint unsigned NOT NULL COMMENT 'TAG建立用户ID',
    `name` varchar(32) NOT NULL COMMENT 'TAG名称',
    `status` tinyint NOT NULL COMMENT '状态',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更新用户',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = 'TAG,用于分组资源跟角色';
-- ----------- lsys-rbac  ---------------