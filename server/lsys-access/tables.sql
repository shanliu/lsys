CREATE TABLE `yaf_user` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `app_id` bigint unsigned NOT NULL COMMENT '应用ID,内置账号登录为0,为yaf_app的ID,>0时为外部账号',
    `user_data` varchar(32) NOT NULL COMMENT '用户唯一标识',
    `user_account` varchar(128) NOT NULL DEFAULT '' COMMENT '尝试登录账号',
    `user_name` varchar(32) NOT NULL  COMMENT '用户名称',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更新时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `user_index` (`app_id`, `user_data`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '用户登录记录';

CREATE TABLE `yaf_session` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint unsigned NOT NULL COMMENT '尝试登录账号对应用户ID',
    `user_app_id` bigint unsigned NOT NULL COMMENT '冗余yaf_user的app_id',
    `oauth_app_id` bigint unsigned NOT NULL COMMENT 'OAUTH登录时的app_id,非OAUTH登录为0,子应用或子应用的应用ID',
    `token_data` varchar(64) NOT NULL COMMENT '授权token',
    `source_token_data` varchar(64) NOT NULL DEFAULT '' COMMENT '源授权token',
    `login_type` varchar(12) NOT NULL DEFAULT '' COMMENT '登录类型',
    `login_ip` varchar(39) NOT NULL DEFAULT '' COMMENT '登陆者IP',
    `device_id` varchar(32) NOT NULL DEFAULT '' COMMENT '设备ID',
    `device_name` varchar(255) NOT NULL DEFAULT '' COMMENT '设备名',
    `status` tinyint NOT NULL COMMENT '状态',
    `add_time` bigint unsigned NOT NULL COMMENT '登录时间',
    `expire_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '超时时间',
    `logout_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '退出时间',
    PRIMARY KEY (`id`),
    KEY `user_index` (`user_id`) USING BTREE,
    UNIQUE KEY `session_index` (
        `user_app_id`,
        `token_data`,
        `status`
    ) USING BTREE,
    KEY `source_session_index` (
        `user_app_id`,
        `source_token_data`,
        `status`
    ) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '用户登录记录';

CREATE TABLE `yaf_session_data` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `session_id` bigint unsigned NOT NULL COMMENT '冗余SESSION id',
    `data_key` varchar(12) NOT NULL COMMENT '数据类型',
    `data_val` text NOT NULL COMMENT '授权数据',
    `change_time` int unsigned NOT NULL DEFAULT 0 COMMENT '最后更改时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `session_id_index` (`session_id`, `data_key`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '用户登录授权数据';