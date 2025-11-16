CREATE TABLE `yaf_app` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `parent_app_id` bigint unsigned NOT NULL COMMENT '父级APP应用ID,0为系统内置用户应用',
    `name` varchar(32) NOT NULL COMMENT '应用名称',
    `client_id` varchar(32) NOT NULL COMMENT '应用key,因为REST接口使用一个参数确定应用,必须全局唯一',
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
    `callback_domain` varchar(255) NOT NULL DEFAULT '' COMMENT '回调域名',
    `scope_data` varchar(255) NOT NULL DEFAULT '' COMMENT '申请的SCOPE列表',
    `change_user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '更改用户',
    `change_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '更改时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `app_id` (`app_id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = 'OAUTH登录数据';

CREATE TABLE `yaf_app_oauth_client_refresh_token` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `app_id` bigint unsigned NOT NULL COMMENT 'app的id',
    `refresh_token_data` varchar(64) NOT NULL COMMENT '生成记录的CODE',
    `time_out` bigint unsigned NOT NULL COMMENT '过期时间',
    `code_data` text NOT NULL COMMENT '登录数据',
    `source_code` varchar(64) NOT NULL  COMMENT '生成记录的CODE',
    `status` tinyint NOT NULL COMMENT '状态',
    `delete_time` bigint unsigned NOT NULL DEFAULT 0  COMMENT '删除时间',
    `add_time` bigint unsigned NOT NULL COMMENT '添加时间',
    PRIMARY KEY (`id`),
    KEY `app_refresh_token_data` (`app_id`,`refresh_token_data`,`time_out`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = 'OAUTH登录刷新token';

CREATE TABLE `yaf_app_oauth_client_access` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `app_id` bigint unsigned NOT NULL COMMENT 'app的id',
    `access_token_data` varchar(64) NOT NULL COMMENT 'token data',
    `refresh_token_data` varchar(64) NOT NULL COMMENT '生成记录的CODE',
    `add_time` bigint unsigned NOT NULL  COMMENT '添加时间',
    PRIMARY KEY (`id`),
    KEY `token_index` (`app_id`,`refresh_token_data`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = 'OAUTH登录token';

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


CREATE TABLE `yaf_app_secret` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `app_id` bigint unsigned NOT NULL COMMENT 'app的id',
    `secret_type` tinyint NOT NULL COMMENT '密钥类型',
    `secret_data` varchar(64) NOT NULL COMMENT '应用秘钥',
    `time_out` bigint unsigned NOT NULL COMMENT '过期时间',
    `status` tinyint NOT NULL COMMENT '状态 正常 已删除',
    `add_user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '添加用户',
    `change_user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '更改用户',
    `change_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '更改时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `app_secret` (`app_id`,`secret_type`,`secret_data`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '应用密钥数据';

CREATE TABLE `yaf_app_request` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `parent_app_id` bigint unsigned NOT NULL COMMENT '父级APP应用ID,0为系统内置用户应用',
    `app_id` bigint unsigned NOT NULL COMMENT '应用ID',
    `request_type` tinyint NOT NULL COMMENT '请求类型:申请应用 更改应用 申请功能',
    `status` tinyint NOT NULL COMMENT '状态:待审核,已审核,已删除',
    `request_user_id` bigint unsigned NOT NULL COMMENT '请求用户',
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



CREATE TABLE `yaf_app_notify_config` (
    `id` int unsigned NOT NULL AUTO_INCREMENT,
    `app_id` bigint unsigned NOT NULL DEFAULT '0' COMMENT '应用ID',
    `app_user_id` bigint unsigned NOT NULL DEFAULT '0' COMMENT '冗余应用用户ID',
    `notify_method` varchar(64) DEFAULT NULL COMMENT '回调类型',
    `call_url` varchar(512) NOT NULL COMMENT '请求URL',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后修改用户id',
    `change_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '最后更新时间',
    `create_time` bigint unsigned NOT NULL COMMENT '创建时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `notify_config_id_IDX` (`app_id`, `notify_method`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '结果推送配置';
CREATE TABLE `yaf_app_notify_data` (
    `id` int unsigned NOT NULL AUTO_INCREMENT,
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID',
    `notify_method` varchar(64) NOT NULL COMMENT '请求方法名',
    `notify_type`tinyint unsigned  COMMENT '请求类型',
    `notify_key` varchar(64) NOT NULL COMMENT '请求标识',
    `notify_payload` text NOT NULL COMMENT '请求JSON数据',
    `status` tinyint NOT NULL COMMENT '请求状态',
    `try_num` tinyint unsigned DEFAULT 0 COMMENT '请求次数',
    `try_mode` tinyint  COMMENT '重试类型',
    `try_max` tinyint unsigned  COMMENT '重试类型',
    `try_delay` smallint unsigned COMMENT '重试基础秒数',
    `publish_time` bigint unsigned NOT NULL DEFAULT 0  COMMENT '最后推送时间',
    `next_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '下次推送时间',
    `delete_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '移除推送时间',
    `create_time` bigint unsigned NOT NULL COMMENT '创建时间',
    `result` varchar(512) NOT NULL DEFAULT '' COMMENT '错误信息',
    PRIMARY KEY (`id`),
    KEY idx_query ( `app_id`, `notify_method`,`notify_key`,`status`, `try_num`, `try_max`, `next_time`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '结果推送数据';