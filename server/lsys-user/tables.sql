-- ----------- lsys-user  ---------------
CREATE TABLE `yaf_user` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT '用户ID',
    `nickname` varchar(32) DEFAULT '' NOT NULL COMMENT '昵称',
    `status` tinyint NOT NULL DEFAULT 1 COMMENT '状态',
    `password_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '密码ID',
    `use_name` tinyint NOT NULL DEFAULT 0 COMMENT '是否启用用户名',
    `email_count` int unsigned NOT NULL DEFAULT 0 COMMENT '绑定邮箱数量 ',
    `mobile_count` int unsigned NOT NULL DEFAULT 0 COMMENT '绑定手机数量  ',
    `external_count` int unsigned NOT NULL DEFAULT 0 COMMENT '绑定外部账号数量 ',
    `address_count` int unsigned NOT NULL DEFAULT 0 COMMENT '收货地址数量 ',
    `add_time` bigint unsigned NOT NULL COMMENT '添加时间',
    `confirm_time` int unsigned NOT NULL DEFAULT 0 COMMENT '状态确认时间,激活时间',
    `change_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '最后更改时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '用户';
CREATE TABLE `yaf_user_address` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint unsigned NOT NULL COMMENT '用户ID',
    `country_code` varchar(6) NOT NULL COMMENT '国家代码',
    `address_code` varchar(21) NOT NULL COMMENT '地址代码',
    `address_info` varchar(64) NOT NULL COMMENT '地址信息,冗余,显示用',
    `address_detail` varchar(128) NOT NULL COMMENT '地址详细',
    `name` varchar(12) NOT NULL COMMENT '姓名',
    `mobile` varchar(13) NOT NULL COMMENT '电话',
    `status` tinyint NOT NULL DEFAULT 1 COMMENT '状态',
    `change_time` int unsigned NOT NULL DEFAULT 0 COMMENT '最后更改时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '用户收货地址';
CREATE TABLE `yaf_user_email` (
    `id` int unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint unsigned NOT NULL COMMENT '用户ID',
    `email` varchar(150) NOT NULL COMMENT '邮箱',
    `status` tinyint NOT NULL DEFAULT 1 COMMENT '绑定状态',
    `confirm_time` int unsigned NOT NULL DEFAULT 0 COMMENT '确认时间',
    `change_time` int unsigned NOT NULL DEFAULT 0 COMMENT '最后更改时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '用户关联邮箱';
CREATE TABLE `yaf_user_external` (
    `id` int unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint unsigned NOT NULL COMMENT '用户ID',
    `external_type` varchar(64) NOT NULL COMMENT '类型 wechat 微信 ',
    `external_id` varchar(125) NOT NULL COMMENT '其他网站用户表示',
    `external_name` varchar(255) NOT NULL DEFAULT '' COMMENT '其他网站用户名',
    `external_gender` varchar(4) NOT NULL DEFAULT '' COMMENT '性别',
    `external_link` varchar(256) NOT NULL DEFAULT '' COMMENT '其他网站用户链接',
    `external_pic` varchar(512) NOT NULL DEFAULT '' COMMENT '其他网站用户头像',
    `external_nikename` varchar(64) NOT NULL DEFAULT '' COMMENT '其他网站用户显示名',
    `status` tinyint NOT NULL DEFAULT 1 COMMENT '状态',
    `config_name` varchar(32) NOT NULL COMMENT '使用配置名',
    `change_time` int unsigned NOT NULL COMMENT '最后更改时间',
    `token_data` varchar(256) NOT NULL DEFAULT '' COMMENT '外部站点登录TOKEN数据',
    `token_timeout` int unsigned DEFAULT 0 COMMENT '外部站点登录超时',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '外部账号登录';
CREATE TABLE `yaf_user_info` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint unsigned NOT NULL COMMENT '用户ID',
    `gender` tinyint NOT NULL DEFAULT 0 COMMENT '性别 1 男 2 女',
    `headimg` varchar(512) NOT NULL DEFAULT '' COMMENT '头像地址',
    `birthday` varchar(10) NOT NULL DEFAULT '' COMMENT '生日',
    `reg_from` varchar(32) NOT NULL DEFAULT '' COMMENT '注册来源',
    `reg_ip` varchar(32) NOT NULL DEFAULT '' COMMENT '注册IP',
    `change_time` int unsigned NOT NULL DEFAULT 0 COMMENT '最后更改时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `yaf_user_user_id_IDX` (`user_id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '用户资料表';
CREATE TABLE `yaf_user_login` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `login_type` varchar(32) NOT NULL COMMENT '登录方式 账号密码登录 邮箱登录 手机登录 手机验证码登录 外部账号登录 链接登录',
    `auth_session` varchar(12) NOT NULL DEFAULT '' COMMENT '验证方式 ',
    `login_account` varchar(128) NOT NULL COMMENT '尝试登录账号',
    `login_ip` varchar(15) NOT NULL DEFAULT '' COMMENT '登陆者IP',
    `user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '尝试登录账号对应用户ID',
    `is_login` tinyint NOT NULL DEFAULT 0 COMMENT '是否登录成功',
    `add_time` bigint unsigned NOT NULL COMMENT '登录时间',
    `login_token` varchar(100) DEFAULT '' NOT NULL COMMENT '登陆token',
    `login_msg` varchar(100) DEFAULT '' NOT NULL COMMENT '登陆消息',
    `login_city` varchar(100) DEFAULT '' NOT NULL COMMENT '登陆城市',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '用户登录记录';
CREATE TABLE `yaf_user_mobile` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint unsigned NOT NULL COMMENT '用户ID',
    `area_code` char(4) NOT NULL COMMENT '电话区号',
    `mobile` char(13) NOT NULL COMMENT '手机号',
    `status` tinyint NOT NULL DEFAULT 1 COMMENT '绑定状态',
    `confirm_time` int unsigned NOT NULL DEFAULT 0 COMMENT '确认时间',
    `change_time` int unsigned NOT NULL DEFAULT 0 COMMENT '最后更改时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '用户登关联手机号';
CREATE TABLE `yaf_user_name` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint unsigned NOT NULL COMMENT '用户ID',
    `username` varchar(100) NOT NULL DEFAULT '' COMMENT '登录用户名',
    `change_time` int unsigned NOT NULL DEFAULT 0 COMMENT '最后更改时间',
    `status` tinyint NOT NULL DEFAULT 1 COMMENT '状态',
    PRIMARY KEY (`id`),
    UNIQUE KEY `yaf_user_user_id_IDX` (`user_id`) USING BTREE,
    UNIQUE KEY `yaf_user_username_IDX` (`username`, `status`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '用户登录账号';
CREATE TABLE `yaf_user_password` (
    `id` int unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint unsigned NOT NULL COMMENT '用户ID',
    `password` varchar(150) NOT NULL COMMENT '密码',
    `add_time` int unsigned NOT NULL COMMENT '绑定时间',
    `disable_time` int unsigned NOT NULL DEFAULT 0 COMMENT '停用时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '用户登录密码';
CREATE TABLE `yaf_user_index` (
    `id` int(10) unsigned NOT NULL AUTO_INCREMENT COMMENT '用户ID',
    `user_id` bigint(20) unsigned NOT NULL COMMENT '用户ID',
    `index_cat` tinyint(3) unsigned NOT NULL COMMENT '索引分类',
    `index_data` varchar(255) DEFAULT NULL COMMENT '索引分词',
    `status` tinyint(4) NOT NULL DEFAULT 1 COMMENT '索引状态 ',
    `change_time` int(10) unsigned NOT NULL DEFAULT 0 COMMENT '最后更改时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `yaf_user_index_user_id_IDX` (`user_id`, `index_cat`, `index_data`) USING BTREE,
    KEY `yaf_user_index_index_data_IDX` (`index_data`, `status`) USING BTREE,
    KEY `yaf_user_index_user_id_status_IDX` (`user_id`, `status`) USING BTREE
) ENGINE = InnoDB AUTO_INCREMENT = 18221001 DEFAULT CHARSET = utf8mb4 COMMENT = '用户数据索引,尝试用外部搜索引擎代替';
-- ----------- lsys-user  ---------------
-- ----------- 初始用户  ---------------
INSERT INTO yaf_user (
        id,
        nickname,
        status,
        password_id,
        use_name,
        add_time,
        confirm_time
    )
VALUES (
        1,
        'root',
        2,
        1,
        1,
        UNIX_TIMESTAMP(),
        UNIX_TIMESTAMP()
    );
-- ----------- 账号:aaaaa   ---------------
INSERT INTO yaf_user_name(user_id, username, change_time, status)
VALUES(1, 'aaaaa', UNIX_TIMESTAMP(), 1);
-- ----------- 密码:000000  ---------------
INSERT INTO yaf_user_password (user_id, password, add_time, disable_time)
VALUES (
        1,
        '670b14728ad9902aecba32e22fa4f6bd',
        UNIX_TIMESTAMP(),
        0
    );
-- ----------- 用户可搜索  ---------------
INSERT INTO yaf_user_index (
        user_id,
        index_cat,
        index_data,
        status,
        change_time
    )
VALUES (1, 5, 'root', 1, UNIX_TIMESTAMP()),
    (1, 6, 'aaaaa', 1, UNIX_TIMESTAMP()),
    (1, 7, '2', 1, UNIX_TIMESTAMP());
-- ----------- 初始用户  ---------------