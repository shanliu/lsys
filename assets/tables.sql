-- test.yaf_rbac_res definition
-- ----------- lsys-rbac  ---------------
CREATE TABLE `yaf_rbac_res` (
    `id` bigint(11) unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `user_id` bigint(11) unsigned NOT NULL,
    `name` varchar(32) NOT NULL,
    `res_key` varchar(32) NOT NULL,
    `status` tinyint(1) NOT NULL,
    `add_user_id` bigint(11) unsigned NOT NULL,
    `change_user_id` bigint(11) unsigned NOT NULL,
    `change_time` bigint(11) unsigned NOT NULL,
    PRIMARY KEY (`id`)
) ENGINE = InnoDB AUTO_INCREMENT = 6 DEFAULT CHARSET = utf8;
-- test.yaf_rbac_res_op definition
CREATE TABLE `yaf_rbac_res_op` (
    `id` bigint(11) unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `name` varchar(32) NOT NULL,
    `op_key` varchar(32) NOT NULL,
    `res_id` bigint(11) unsigned NOT NULL,
    `status` tinyint(1) NOT NULL,
    `change_user_id` bigint(11) unsigned NOT NULL,
    `change_time` bigint(11) unsigned NOT NULL,
    PRIMARY KEY (`id`)
) ENGINE = InnoDB AUTO_INCREMENT = 6 DEFAULT CHARSET = utf8;
-- test.yaf_rbac_role definition
CREATE TABLE `yaf_rbac_role` (
    `id` bigint(11) unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `user_id` bigint(11) unsigned NOT NULL,
    `name` varchar(32) NOT NULL,
    `relation_key` varchar(32) NOT NULL,
    `user_range` tinyint(1) NOT NULL,
    `res_op_range` tinyint(1) NOT NULL,
    `status` tinyint(1) NOT NULL,
    `change_user_id` bigint(11) unsigned NOT NULL,
    `change_time` bigint(11) unsigned NOT NULL,
    `priority` tinyint(3) DEFAULT 99,
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
    KEY `yaf_rbac_role_priority_IDX` (`priority`, `id`) USING BTREE
) ENGINE = InnoDB AUTO_INCREMENT = 768686 DEFAULT CHARSET = utf8;
-- test.yaf_rbac_role_op definition
CREATE TABLE `yaf_rbac_role_op` (
    `id` bigint(11) unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `res_op_id` bigint(11) unsigned NOT NULL,
    `role_id` bigint(11) unsigned NOT NULL,
    `positivity` tinyint(1) NOT NULL,
    `status` tinyint(1) NOT NULL,
    `change_user_id` bigint(11) unsigned NOT NULL,
    `change_time` bigint(11) unsigned NOT NULL,
    PRIMARY KEY (`id`),
    KEY `yaf_rbac_role_op_IDX` (
        `role_id`,
        `res_op_id`,
        `status`
    ) USING BTREE
) ENGINE = InnoDB AUTO_INCREMENT = 6 DEFAULT CHARSET = utf8;
-- test.yaf_rbac_role_user definition
CREATE TABLE `yaf_rbac_role_user` (
    `id` bigint(11) unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `role_id` bigint(11) unsigned NOT NULL,
    `user_id` bigint(11) unsigned NOT NULL,
    `timeout` bigint(11) unsigned NOT NULL,
    `status` tinyint(1) NOT NULL,
    `change_user_id` bigint(11) unsigned NOT NULL,
    `change_time` bigint(11) unsigned NOT NULL,
    PRIMARY KEY (`id`),
    KEY `yaf_rbac_role_user_IDX` (`role_id`, `user_id`, `status`) USING BTREE
) ENGINE = InnoDB AUTO_INCREMENT = 6 DEFAULT CHARSET = utf8;
-- test.yaf_rbac_tags definition
CREATE TABLE `yaf_rbac_tags` (
    `id` bigint(11) unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `from_id` bigint(11) unsigned NOT NULL,
    `from_source` tinyint(1) NOT NULL,
    `user_id` bigint(11) unsigned NOT NULL,
    `name` varchar(32) NOT NULL,
    `status` tinyint(1) NOT NULL,
    `change_user_id` bigint(11) unsigned NOT NULL,
    `change_time` bigint(11) unsigned NOT NULL,
    PRIMARY KEY (`id`)
) ENGINE = InnoDB AUTO_INCREMENT = 6 DEFAULT CHARSET = utf8;
-- ----------- lsys-rbac  ---------------
-- ----------- lsys-user  ---------------
-- test.yaf_user definition
CREATE TABLE `yaf_user` (
    `id` bigint(11) unsigned NOT NULL AUTO_INCREMENT COMMENT '用户ID',
    `nickname` varchar(32) DEFAULT NULL COMMENT '昵称',
    `status` tinyint(11) NOT NULL DEFAULT 1 COMMENT '1bit 是否激活 2bit 是否屏蔽 default:  1',
    `password_id` bigint(11) unsigned NOT NULL DEFAULT 0 COMMENT '密码ID default:  0',
    `use_name` tinyint(1) NOT NULL DEFAULT 0 COMMENT '是否启用用户名  default:  0',
    `email_count` int(11) unsigned NOT NULL DEFAULT 0 COMMENT '绑定邮箱数量  default:  0',
    `mobile_count` int(11) unsigned NOT NULL DEFAULT 0 COMMENT '绑定手机数量  default:  0',
    `external_count` int(11) unsigned NOT NULL DEFAULT 0 COMMENT '绑定外部账号数量  default:  0',
    `address_count` int(11) unsigned NOT NULL DEFAULT 0 COMMENT '收货地址数量  default:  0',
    `add_time` bigint(11) unsigned NOT NULL COMMENT '添加时间',
    `delete_time` bigint(11) unsigned NOT NULL DEFAULT 0 COMMENT '删除时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB AUTO_INCREMENT = 6 DEFAULT CHARSET = utf8;
-- test.yaf_user_address definition
CREATE TABLE `yaf_user_address` (
    `id` bigint(11) unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint(11) unsigned NOT NULL COMMENT '用户ID',
    `address_code` varchar(21) NOT NULL COMMENT '地址代码',
    `address_info` varchar(64) NOT NULL COMMENT '地址信息,冗余,显示用',
    `address_detail` varchar(128) NOT NULL COMMENT '地址详细',
    `name` varchar(12) NOT NULL COMMENT '姓名',
    `mobile` varchar(13) NOT NULL COMMENT '电话',
    `status` tinyint(11) NOT NULL DEFAULT 1 COMMENT '1bit 是否激活 2bit 是否屏蔽 default:  1',
    `add_time` int(11) unsigned NOT NULL COMMENT '添加时间',
    `delete_time` int(11) unsigned NOT NULL DEFAULT 0 COMMENT '删除时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8 COMMENT = '用户收货地址';
-- test.yaf_user_email definition
CREATE TABLE `yaf_user_email` (
    `id` int(11) unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint(11) unsigned NOT NULL COMMENT '用户ID',
    `email` varchar(150) NOT NULL COMMENT '邮箱',
    `status` tinyint(1) NOT NULL DEFAULT 1 COMMENT '绑定状态1正常 2待验证 3关闭',
    `confirm_time` int(11) unsigned NOT NULL DEFAULT 0 COMMENT '确认时间',
    `add_time` int(11) unsigned NOT NULL COMMENT '添加时间',
    `delete_time` int(11) unsigned NOT NULL DEFAULT 0 COMMENT '删除时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB AUTO_INCREMENT = 15 DEFAULT CHARSET = utf8;
-- test.yaf_user_external definition
CREATE TABLE `yaf_user_external` (
    `id` int(11) unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint(11) unsigned NOT NULL COMMENT '用户ID',
    `external_type` varchar(64) unsigned NOT NULL COMMENT '类型 wechat 微信 ',
    `external_id` varchar(125) NOT NULL COMMENT '其他网站用户表示',
    `external_name` varchar(255) NOT NULL DEFAULT '' COMMENT '其他网站用户名',
    `external_gender` varchar(4) NOT NULL DEFAULT '' COMMENT '性别',
    `external_link` varchar(256) NOT NULL DEFAULT '' COMMENT '其他网站用户链接',
    `external_pic` varchar(512) NOT NULL DEFAULT '' COMMENT '其他网站用户头像',
    `external_nikename` varchar(64) NOT NULL DEFAULT '' COMMENT '其他网站用户显示名',
    `status` tinyint(1) NOT NULL DEFAULT 1 COMMENT '是否标注为删除 0 表示删除 1 表示正常',
    `config_name` varchar(32) NOT NULL COMMENT '使用配置名',
    `add_time` int(10) unsigned NOT NULL COMMENT '添加时间',
    `delete_time` int(10) unsigned NOT NULL DEFAULT 0 COMMENT '删除时间',
    `change_time` int(10) unsigned NOT NULL DEFAULT 0 COMMENT '时间',
    `token_data` varchar(256) NOT NULL DEFAULT '',
    `token_timeout` int(10) unsigned DEFAULT 0,
    PRIMARY KEY (`id`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8 COMMENT = '外部账号登录';
-- test.yaf_user_info definition
CREATE TABLE `yaf_user_info` (
    `id` bigint(11) unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint(11) unsigned NOT NULL COMMENT '用户ID',
    `gender` tinyint(1) NOT NULL DEFAULT 0 COMMENT '性别 1 男 2 女',
    `headimg` varchar(512) NOT NULL DEFAULT '' COMMENT '头像地址',
    `birthday` varchar(10) NOT NULL DEFAULT '' COMMENT '生日',
    `block_time` int(10) unsigned NOT NULL DEFAULT 0 COMMENT '屏蔽时间',
    `reg_from` varchar(32) NOT NULL DEFAULT '' COMMENT '注册来源',
    `reg_ip` varchar(32) NOT NULL DEFAULT '' COMMENT '注册IP',
    `confirm_time` int(10) unsigned NOT NULL DEFAULT 0 COMMENT '确认时间',
    `change_time` int(10) unsigned NOT NULL DEFAULT 0,
    PRIMARY KEY (`id`)
) ENGINE = InnoDB AUTO_INCREMENT = 2 DEFAULT CHARSET = utf8 COMMENT = '用户资料表';
-- test.yaf_user_login definition
CREATE TABLE `yaf_user_login` (
    `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
    `login_type` varchar(32) NOT NULL COMMENT '登录方式 ID密码登录 1 账号密码登录2 邮箱登录3 手机登录4 手机验证码登录5 外部账号登录6 链接登录7',
    `auth_session` varchar(12) NOT NULL DEFAULT '' COMMENT '验证方式 ',
    `login_account` varchar(128) NOT NULL COMMENT '尝试登录账号',
    `login_ip` varchar(15) NOT NULL DEFAULT '' COMMENT '登陆者IP',
    `user_id` bigint(20) unsigned NOT NULL DEFAULT 0 COMMENT '尝试登录账号对应用户ID',
    `is_login` tinyint(1) NOT NULL DEFAULT 0 COMMENT '是否登录成功',
    `add_time` bigint(20) unsigned NOT NULL COMMENT '登录时间',
    `login_token` varchar(100) DEFAULT '' NOT NULL COMMENT '登陆token',
    `login_msg` varchar(100) DEFAULT '' NOT NULL COMMENT '登陆消息',
    `login_city` varchar(100) DEFAULT '' NOT NULL COMMENT '登陆城市',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB AUTO_INCREMENT = 226 DEFAULT CHARSET = utf8 COMMENT = '用户登录记录';
-- test.yaf_user_mobile definition
CREATE TABLE `yaf_user_mobile` (
    `id` bigint(11) unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint(11) unsigned NOT NULL COMMENT '用户ID',
    `area_code` char(4) NOT NULL COMMENT '电话区号',
    `mobile` char(13) NOT NULL COMMENT '手机号',
    `status` tinyint(1) NOT NULL DEFAULT 1 COMMENT '绑定状态1正常 2待验证 3关闭',
    `add_time` bigint(11) unsigned NOT NULL COMMENT '添加时间',
    `confirm_time` bigint(11) unsigned NOT NULL DEFAULT 0,
    `delete_time` bigint(11) unsigned NOT NULL DEFAULT 0,
    PRIMARY KEY (`id`)
) ENGINE = InnoDB AUTO_INCREMENT = 5 DEFAULT CHARSET = utf8;
-- test.yaf_user_name definition
CREATE TABLE `yaf_user_name` (
    `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint(11) unsigned NOT NULL COMMENT '用户ID',
    `username` varchar(100) NOT NULL DEFAULT '',
    `add_time` bigint(20) unsigned NOT NULL,
    `change_time` bigint(20) unsigned NOT NULL DEFAULT 0,
    PRIMARY KEY (`id`)
) ENGINE = InnoDB AUTO_INCREMENT = 2 DEFAULT CHARSET = utf8 COMMENT = '用户名表';
-- test.yaf_user_password definition
CREATE TABLE `yaf_user_password` (
    `id` int(11) unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint(11) unsigned NOT NULL COMMENT '用户ID',
    `password` varchar(150) NOT NULL COMMENT '密码',
    `change_time` int(11) unsigned NOT NULL DEFAULT 0 COMMENT '更改时间',
    `add_time` int(11) unsigned NOT NULL COMMENT '绑定时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB AUTO_INCREMENT = 2 DEFAULT CHARSET = utf8;
-- test.yaf_users definition
CREATE TABLE `yaf_users` (
    `id` int(11) unsigned NOT NULL AUTO_INCREMENT COMMENT '用户ID',
    `nickname` varchar(32) DEFAULT NULL COMMENT '昵称',
    `gender` tinyint(3) unsigned NOT NULL DEFAULT 0 COMMENT '性别 1 男 2 女',
    `headimg` varchar(64) DEFAULT NULL COMMENT '头像地址',
    `password_id` int(10) unsigned NOT NULL DEFAULT 0 COMMENT '密码ID',
    `add_time` int(10) unsigned NOT NULL COMMENT '添加时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB AUTO_INCREMENT = 932 DEFAULT CHARSET = utf8mb4;
CREATE TABLE `yaf_user_index` (
    `id` int(11) unsigned NOT NULL AUTO_INCREMENT COMMENT '用户ID',
    `user_id` bigint(11) unsigned NOT NULL COMMENT '用户ID',
    `index_cat` tinyint(3) unsigned NOT NULL COMMENT '索引分类',
    `index_data` varchar(255) DEFAULT NULL COMMENT '索引分词',
    `status` tinyint(1) NOT NULL DEFAULT 1 COMMENT '索引状态 ',
    `add_time` int(10) unsigned NOT NULL COMMENT '添加时间',
    `delete_time` int(10) unsigned DEFAULT 0 NOT NULL COMMENT '删除时间',
    PRIMARY KEY (`id`),
    KEY `yaf_user_index_cat` (
        `index_cat`,
        `index_data`,
        `status`
    ) USING BTREE,
    KEY `yaf_user_index_user` (`user_id`) USING BTREE
) ENGINE = InnoDB AUTO_INCREMENT = 1 DEFAULT CHARSET = utf8mb4;
-- ----------- lsys-user  ---------------
-- ----------- lsys-app  ---------------
CREATE TABLE `yaf_app` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `name` varchar(32) NOT NULL COMMENT '应用名称',
    `client_id` varchar(32) NOT NULL COMMENT '应用key',
    `client_secret` varchar(32) NOT NULL COMMENT '应用秘钥',
    `callback_domain` varchar(255) NOT NULL DEFAULT '' COMMENT '回调域名',
    `status` tinyint(1) NOT NULL COMMENT '状态',
    `user_id` bigint unsigned NOT NULL COMMENT '添加用户ID',
    `add_user_id` bigint unsigned NOT NULL COMMENT '申请用户',
    `add_time` bigint unsigned NOT NULL COMMENT '申请时间',
    `confirm_user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '确认用户',
    `confirm_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '确认时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB AUTO_INCREMENT = 28 DEFAULT CHARSET = utf8mb4;
CREATE TABLE `yaf_app_oauth_token` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `app_id` bigint unsigned NOT NULL COMMENT '应用ID',
    `access_user_id` bigint unsigned NOT NULL COMMENT '授权用户ID',
    `code` varchar(32) NOT NULL COMMENT '来源CODE',
    `token` varchar(32) NOT NULL COMMENT '授权TOKEN',
    `scope` varchar(512) NOT NULL COMMENT '授权范围',
    `status` tinyint(1) NOT NULL COMMENT '状态',
    `token_time` bigint unsigned NOT NULL COMMENT '授权时间',
    `timeout` bigint unsigned NOT NULL COMMENT '超时',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB AUTO_INCREMENT = 28 DEFAULT CHARSET = utf8mb4;
-- ----------- lsys-app  ---------------
-- ----------- lsys-sender  ---------------
CREATE TABLE `yaf_sender_aliyun_config` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `name` varchar(32) NOT NULL COMMENT '应用端配置显示名',
    `access_id` varchar(32) NOT NULL COMMENT '阿里云key',
    `access_secret` varchar(64) NOT NULL COMMENT '阿里云secret',
    `status` tinyint(1) NOT NULL COMMENT '状态',
    `user_id` bigint unsigned NOT NULL COMMENT '申请时间',
    `add_time` bigint unsigned NOT NULL COMMENT '申请时间',
    `delete_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '确认时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB AUTO_INCREMENT = 28 DEFAULT CHARSET = utf8mb4;
CREATE TABLE `yaf_sender_sms_aliyun` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID',
    `aliyun_config_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT 'yaf_sender_aliyun_config ID',
    `sms_tpl` varchar(32) NOT NULL COMMENT '模板KEY',
    `aliyun_sign_name` varchar(32) NOT NULL COMMENT '阿里云签名',
    `aliyun_sms_tpl` varchar(32) NOT NULL COMMENT '阿里云模板名',
    `status` tinyint(1) NOT NULL COMMENT '状态',
    `max_try_num` SMALLINT(3) unsigned NOT NULL DEFAULT 0 COMMENT '最大发送次数',
    `user_id` bigint unsigned NOT NULL COMMENT '用户id',
    `add_time` bigint unsigned NOT NULL COMMENT '申请时间',
    `delete_user_id` bigint unsigned NOT NULL COMMENT '删除用户id',
    `delete_time` bigint unsigned NOT NULL COMMENT '删除时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB AUTO_INCREMENT = 28 DEFAULT CHARSET = utf8mb4;
CREATE TABLE `yaf_sender_sms_config` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID,由应用生成',
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID',
    `priority` tinyint(3) DEFAULT 99,
    `config_type` tinyint(3) NOT NULL COMMENT '配置类型',
    `config_data` varchar(512) NOT NULL COMMENT '配置数据',
    `status` tinyint(1) NOT NULL COMMENT '发送状态',
    `user_id` bigint unsigned NOT NULL COMMENT '用户id',
    `delete_user_id` bigint unsigned NOT NULL COMMENT '删除用户id',
    `add_time` bigint unsigned NOT NULL COMMENT '申请时间',
    `delete_time` bigint unsigned NOT NULL COMMENT '删除时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB AUTO_INCREMENT = 28 DEFAULT CHARSET = utf8mb4;
CREATE TABLE `yaf_sender_sms_message` (
    `id` bigint unsigned NOT NULL COMMENT 'ID,由应用生成',
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID',
    `area` varchar(32) NOT NULL COMMENT '区号',
    `mobile` varchar(32) NOT NULL COMMENT '手机号',
    `tpl_id` varchar(32) NOT NULL COMMENT '模板ID',
    `tpl_var` varchar(512) NOT NULL DEFAULT '' COMMENT '模板变量',
    `try_num` SMALLINT(3) unsigned NOT NULL DEFAULT 0 COMMENT '发送次数',
    `status` tinyint(1) NOT NULL COMMENT '发送状态',
    `add_time` bigint unsigned NOT NULL COMMENT '申请时间',
    `expected_time` bigint unsigned NOT NULL COMMENT '预计发送时间',
    `send_time` bigint unsigned NOT NULL COMMENT '发送时间',
    `user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '用户id',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB AUTO_INCREMENT = 28 DEFAULT CHARSET = utf8mb4;
CREATE TABLE `yaf_sender_sms_cancel` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `sms_message_id` bigint unsigned NOT NULL COMMENT 'ID',
    `cancel_hand` varchar(32) NOT NULL COMMENT '取消key',
    `status` tinyint(1) NOT NULL COMMENT '发送状态',
    `user_id` bigint unsigned NOT NULL COMMENT '用户id',
    `cancel_time` bigint unsigned NOT NULL COMMENT '确认时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB AUTO_INCREMENT = 28 DEFAULT CHARSET = utf8mb4;
CREATE TABLE `yaf_sender_sms_log` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `sms_message_id` bigint unsigned NOT NULL COMMENT 'ID',
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID',
    `log_type` tinyint(3) NOT NULL COMMENT '日志类型',
    `send_type` varchar(32) NOT NULL COMMENT '触发来源',
    `message` varchar(255) NOT NULL COMMENT '发送消息',
    `status` tinyint(1) NOT NULL COMMENT '发送状态',
    `create_time` bigint unsigned NOT NULL COMMENT '确认时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB AUTO_INCREMENT = 28 DEFAULT CHARSET = utf8mb4;
-- ----------- lsys-sender  ---------------