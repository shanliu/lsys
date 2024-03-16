-- ----------- lsys-app-barcode  ---------------
CREATE TABLE `yaf_barcode_output` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `app_id` bigint unsigned NOT NULL COMMENT '应用ID',
    `code_format` tinyint unsigned NOT NULL COMMENT '编码格式',
    `image_format` tinyint unsigned NOT NULL COMMENT '图片格式',
    `image_size` bigint unsigned NOT NULL COMMENT '图片大小',
    `image_color_front` int unsigned NOT NULL DEFAULT 0 COMMENT '编码颜色',
    `image_color_background` int unsigned NOT NULL DEFAULT 16777215 COMMENT '背景色',
    `image_background` text NOT NULL COMMENT '背景图,BASE64',
    `user_id` bigint unsigned NOT NULL COMMENT '用户id',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后修改用户id',
    `change_time` bigint unsigned NOT NULL COMMENT '最后修改时间',
    PRIMARY KEY (`id`),
    KEY `appid` (`app_id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = 'barcode 应用输出配置';
CREATE TABLE `yaf_barcode_parse` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `app_id` bigint unsigned NOT NULL COMMENT '应用ID',
    `file_path` varchar(255) NOT NULL COMMENT '文件',
    `file_hash` varchar(64) NOT NULL COMMENT '文件hash',
    `file_size` bigint unsigned NOT NULL COMMENT '文件大小',
    `record` text NOT NULL COMMENT '解析结果',
    `user_id` bigint unsigned NOT NULL COMMENT '用户id',
    `create_time` bigint unsigned NOT NULL COMMENT '解析时间',
    PRIMARY KEY (`id`),
    KEY `appid` (`app_id`) USING BTREE,
    KEY `file_hash` (`file_hash`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = 'barcode 解析结果记录';
-- ----------- lsys-app-barcode  ---------------