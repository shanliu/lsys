-- ----------- lsys-setting  ---------------
CREATE TABLE `yaf_setting` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `setting_type` tinyint NOT NULL COMMENT '类型',
    `name` varchar(32) NOT NULL COMMENT '应用端配置显示名',
    `setting_key` varchar(32) NOT NULL COMMENT '应用端配置key',
    `setting_data` text NOT NULL COMMENT '内容',
    `user_id` bigint unsigned NOT NULL COMMENT '用户ID 系统为0',
    `status` tinyint NOT NULL COMMENT '状态',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后修改用户',
    `change_time` bigint unsigned NOT NULL COMMENT '最后修改时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '配置数据';
-- ----------- lsys-setting  ---------------