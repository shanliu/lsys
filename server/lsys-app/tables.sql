CREATE TABLE `yaf_app` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `parent_app_id` bigint unsigned NOT NULL COMMENT '父级APP应用ID,0为系统内置用户应用',
    `name` varchar(32) NOT NULL COMMENT '应用名称',
    `client_id` varchar(32) NOT NULL COMMENT '应用key,因为REST接口使用一个参数确定应用,必须全局唯一',
    `client_secret` varchar(32) NOT NULL COMMENT '应用秘钥',
    `status` tinyint NOT NULL COMMENT '状态 待审核 正常 已禁用',
    `user_id` bigint unsigned NOT NULL COMMENT '添加用户ID',
    `user_app_id` bigint unsigned NOT NULL COMMENT '冗余user表的app_id,>0时为外部账号',
    `change_user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '更改用户',
    `change_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '更改时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `app_client_id` (`client_id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '应用数据';

CREATE TABLE `yaf_app_feature` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `app_id` bigint unsigned NOT NULL COMMENT 'app的id',
    `feature_key` varchar(32) NOT NULL COMMENT '功能KEY',
    `status` tinyint NOT NULL COMMENT '状态 已开通 已禁用',
    `change_user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '更改用户',
    `change_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '更改时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `app_key` (`app_id`,`feature_key`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '可用功能数据';

CREATE TABLE `yaf_app_oauth_client` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `app_id` bigint unsigned NOT NULL COMMENT 'app的id',
    `oauth_secret` varchar(32) NOT NULL COMMENT 'oauth秘钥',
    `callback_domain` varchar(255) NOT NULL DEFAULT '' COMMENT '回调域名',
    `scope_data` varchar(255) NOT NULL DEFAULT '' COMMENT '申请的SCOPE列表',
    `change_user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '更改用户',
    `change_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '更改时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `app_id` (`app_id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = 'OAUTH登录数据';


CREATE TABLE `yaf_app_oauth_server_scope` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `app_id` bigint unsigned NOT NULL COMMENT 'app的id',
    `scope_key` varchar(12) NOT NULL  COMMENT 'SCOPE KEY',
    `scope_name` varchar(12) NOT NULL  COMMENT 'SCOPE 显示名',
    `scope_desc` varchar(128) NOT NULL  COMMENT 'SCOPE 介绍',
    `status` tinyint NOT NULL COMMENT '状态',
    `change_user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '更改用户',
    `change_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '更改时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `app_key` (`app_id`,`scope_key`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '外部账号OAUTH登录服务SCOPE数据';


CREATE TABLE `yaf_app_request` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `parent_app_id` bigint unsigned NOT NULL COMMENT '父级APP应用ID,0为系统内置用户应用',
    `app_id` bigint unsigned NOT NULL COMMENT '应用ID',
    `request_type` tinyint NOT NULL COMMENT '请求类型:申请应用 更改应用 申请功能',
    `status` tinyint NOT NULL COMMENT '状态:待审核,已审核,已删除',
    `request_user_id` bigint unsigned NOT NULL COMMENT '最后更新用户',
    `request_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    `confirm_user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '确认用户',
    `confirm_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '确认时间',
    `confirm_note` varchar(32) NOT NULL DEFAULT '' COMMENT '确认消息',
    PRIMARY KEY (`id`),
    KEY `app_user_id` (`app_id`,`request_type`,`status`) USING BTREE,
    KEY `parent_app_user_id` (`parent_app_id`,`app_id`,`status`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '应用申请';

CREATE TABLE `yaf_app_request_feature` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `app_request_id` bigint unsigned NOT NULL COMMENT '应用ID',
    `feature_data` varchar(255) NOT NULL DEFAULT '' COMMENT '申请的feature列表',
    PRIMARY KEY (`id`),
    UNIQUE KEY `req_id` (`app_request_id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '应用申请OAUTH登录相关数据';


CREATE TABLE `yaf_app_request_oauth_client` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `app_request_id` bigint unsigned NOT NULL COMMENT '应用ID',
    `scope_data` varchar(255) NOT NULL DEFAULT '' COMMENT '申请的SCOPE列表',
    PRIMARY KEY (`id`),
    UNIQUE KEY `req_id` (`app_request_id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '应用申请OAUTH登录相关数据';

CREATE TABLE `yaf_app_request_set_info` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `app_request_id` bigint unsigned NOT NULL COMMENT '应用ID',
    `name` varchar(32) NOT NULL  DEFAULT '' COMMENT '应用名称',
    `client_id` varchar(32) NOT NULL DEFAULT '' COMMENT '应用key',
    PRIMARY KEY (`id`),
    UNIQUE  KEY `req_id` (`app_request_id`) USING BTREE,
    KEY `client_id` (`client_id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '应用申请更改相关数据';


