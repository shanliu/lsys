-- ----------- lsys-app  ---------------
CREATE TABLE `yaf_app` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `name` varchar(32) NOT NULL COMMENT '应用名称',
    `client_id` varchar(32) NOT NULL COMMENT '应用key',
    `client_secret` varchar(32) NOT NULL COMMENT '应用秘钥',
    `oauth_secret` varchar(32) NOT NULL COMMENT 'oauth秘钥',
    `callback_domain` varchar(255) NOT NULL DEFAULT '' COMMENT '回调域名',
    `status` tinyint NOT NULL COMMENT '状态',
    `user_id` bigint unsigned NOT NULL COMMENT '添加用户ID',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更新用户',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    `confirm_user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '确认用户',
    `confirm_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '确认时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '应用数据';
CREATE TABLE `yaf_app_oauth_token` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `app_id` bigint unsigned NOT NULL COMMENT '应用ID',
    `access_user_id` bigint unsigned NOT NULL COMMENT '授权用户ID',
    `code` varchar(32) NOT NULL COMMENT '来源CODE',
    `token` varchar(32) NOT NULL COMMENT '授权TOKEN',
    `scope` varchar(512) NOT NULL COMMENT '授权范围',
    `status` tinyint NOT NULL COMMENT '状态',
    `token_time` bigint unsigned NOT NULL COMMENT '授权时间',
    `timeout` bigint unsigned NOT NULL COMMENT '超时',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '应用OAUTH登录数据';
CREATE TABLE `yaf_app_sub_user` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `app_id` bigint unsigned NOT NULL COMMENT '父应用ID',
    `user_id` bigint unsigned NOT NULL COMMENT '关联用户ID',
    `status` tinyint NOT NULL COMMENT '状态',
    `change_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '添加时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `app_user_id` (`app_id`, `user_id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '授权子用户';
CREATE TABLE `yaf_app_sub_app` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `app_id` bigint unsigned NOT NULL COMMENT '父应用ID',
    `user_id` bigint unsigned NOT NULL COMMENT '关联用户ID',
    `sub_app_id` bigint unsigned NOT NULL COMMENT '子应用ID',
    `sub_client_secret` varchar(32) NOT NULL COMMENT '子秘钥',
    `status` tinyint NOT NULL COMMENT '状态',
    `change_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '最后修改时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `app_user_id` (`app_id`, `user_id`, `sub_app_id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '子应用数据';
-- ----------- lsys-app  ---------------