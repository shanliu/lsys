-- ----------- lsys-app-barcode  ---------------
CREATE TABLE `yaf_barcode_create` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `app_id` bigint unsigned NOT NULL COMMENT '应用ID',
    `user_id` bigint unsigned NOT NULL COMMENT '用户id',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后修改用户id',
    `change_time` bigint unsigned NOT NULL DEFAULT '0' COMMENT '最后修改时间',
    `create_time` bigint unsigned NOT NULL COMMENT '添加时间',
    `status` tinyint NOT NULL COMMENT '状态',
    `barcode_type` varchar(12) NOT NULL COMMENT '二维码类型',
    `image_format` varchar(6) NOT NULL COMMENT '图片输出格式',
    `image_width` int NOT NULL COMMENT '图片宽',
    `image_height` int NOT NULL COMMENT '图片高',
    `margin` int NOT NULL COMMENT '图片留白',
    `image_color` varchar(8) NOT NULL COMMENT '前景颜色',
    `image_background` varchar(8) NOT NULL COMMENT '背景颜色',
    PRIMARY KEY (`id`),
    KEY `appid` (`app_id`) USING BTREE,
    KEY `user_id` (`user_id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = 'barcode显示配置';
CREATE TABLE `yaf_barcode_parse` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint unsigned NOT NULL COMMENT '用户id',
    `app_id` bigint unsigned NOT NULL COMMENT '应用ID',
    `file_hash` varchar(64) NOT NULL COMMENT '文件hash',
    `barcode_type` varchar(12) NOT NULL COMMENT '文件',
    `record` text NOT NULL COMMENT '解析结果',
    `status` tinyint NOT NULL COMMENT '状态',
    `create_time` bigint unsigned NOT NULL COMMENT '解析时间',
    `change_time` bigint unsigned NOT NULL DEFAULT '0' COMMENT '最后修改时间',
    PRIMARY KEY (`id`),
    KEY `appid` (`app_id`) USING BTREE,
    KEY `file_hash` (`file_hash`) USING BTREE,
    KEY `user_id` (`user_id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = 'barcode 解析结果记录';
-- ----------- lsys-app-barcode  ---------------