-- ----------- lsys-app-notify  ---------------
CREATE TABLE `yaf_notify_config` (
    `id` int unsigned NOT NULL AUTO_INCREMENT,
    `app_id` bigint unsigned NOT NULL DEFAULT '0' COMMENT '应用ID',
    `method` varchar(64) DEFAULT NULL COMMENT '回调类型',
    `call_url` varchar(512) NOT NULL COMMENT '请求URL',
    `user_id` bigint unsigned NOT NULL COMMENT '用户id',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后修改用户id',
    `change_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '最后更新时间',
    `create_time` bigint unsigned NOT NULL COMMENT '创建时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `yaf_notify_config_id_IDX` (`app_id`, `method`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '结果推送配置';
CREATE TABLE `yaf_notify_data` (
    `id` int unsigned NOT NULL AUTO_INCREMENT,
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID',
    `method` varchar(64) NOT NULL COMMENT '请求方法名',
    `payload` text NOT NULL COMMENT '请求JSON数据',
    `status` tinyint NOT NULL COMMENT '请求状态',
    `result` varchar(128) NOT NULL COMMENT '错误信息',
    `try_num` tinyint DEFAULT 0 COMMENT '请求次数',
    `publish_time` bigint unsigned NOT NULL COMMENT '最后推送时间',
    `next_time` bigint unsigned NOT NULL COMMENT '下次推送时间',
    `create_time` bigint unsigned NOT NULL COMMENT '创建时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '结果推送数据';
-- ----------- lsys-app-notify  ---------------