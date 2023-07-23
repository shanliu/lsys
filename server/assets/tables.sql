CREATE TABLE `yaf_change_logs` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `log_type` varchar(32) NOT NULL COMMENT '日志类型',
    `log_data` text NOT NULL COMMENT '日志数据',
    `message` varchar(255) DEFAULT '' NULL COMMENT '消息',
    `user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '操作记录用户ID',
    `source_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '相关操作记录ID',
    `add_user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '操作用户ID',
    `user_ip` varchar(40) NOT NULL DEFAULT '' COMMENT '操作者IP',
    `request_id` varchar(32) NOT NULL DEFAULT '' COMMENT '请求id',
    `request_user_agent` varchar(254) NOT NULL DEFAULT '' COMMENT '请求UA',
    `add_time` bigint unsigned NOT NULL COMMENT '添加时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '操作日志';
-- ----------- lsys-rbac  ---------------
CREATE TABLE `yaf_rbac_res` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint unsigned NOT NULL COMMENT '用户ID',
    `name` varchar(32) NOT NULL COMMENT '资源名',
    `res_key` varchar(32) NOT NULL COMMENT '资源KEY',
    `status` tinyint NOT NULL COMMENT '状态',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更新用户',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '资源';
-- test.yaf_rbac_res_op definition
CREATE TABLE `yaf_rbac_res_op` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `name` varchar(32) NOT NULL COMMENT '资源操作名',
    `op_key` varchar(32) NOT NULL COMMENT '资源操作KEY',
    `res_id` bigint unsigned NOT NULL COMMENT '资源ID',
    `status` tinyint NOT NULL COMMENT '状态',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更新用户',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '资源可用操作';
-- test.yaf_rbac_role definition
CREATE TABLE `yaf_rbac_role` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint unsigned NOT NULL COMMENT '用户ID',
    `name` varchar(32) NOT NULL COMMENT '角色名',
    `relation_key` varchar(32) NOT NULL COMMENT '关系角色KEY,其他角色类型为空',
    `user_range` tinyint NOT NULL COMMENT '角色包含用户范围',
    `res_op_range` tinyint NOT NULL COMMENT '角色可操作资源范围',
    `status` tinyint NOT NULL COMMENT '状态',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更新用户',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    `priority` tinyint DEFAULT 99,
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
    KEY `yaf_rbac_role_relation_key_IDX` (
        `user_id`,
        `status`,
        `user_range`,
        `relation_key`
    ) USING BTREE,
    KEY `yaf_rbac_role_priority_IDX` (`priority`, `id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '角色';
-- test.yaf_rbac_role_op definition
CREATE TABLE `yaf_rbac_role_op` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `res_op_id` bigint unsigned NOT NULL COMMENT '角色关联资源操作ID',
    `role_id` bigint unsigned NOT NULL COMMENT '角色ID',
    `positivity` tinyint NOT NULL COMMENT '授权类型: 授权操作 禁止操作',
    `status` tinyint NOT NULL COMMENT '状态',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更新用户',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    PRIMARY KEY (`id`),
    KEY `yaf_rbac_role_op_IDX` (
        `role_id`,
        `res_op_id`,
        `status`
    ) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '角色关联的资源操作';
-- test.yaf_rbac_role_user definition
CREATE TABLE `yaf_rbac_role_user` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `role_id` bigint unsigned NOT NULL COMMENT '角色ID',
    `user_id` bigint unsigned NOT NULL COMMENT '用户ID',
    `timeout` bigint unsigned NOT NULL COMMENT '角色关联用户超时',
    `status` tinyint NOT NULL COMMENT '状态',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更新用户',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    PRIMARY KEY (`id`),
    KEY `yaf_rbac_role_user_IDX` (`role_id`, `user_id`, `status`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '角色关联的用户';
-- test.yaf_rbac_tags definition
CREATE TABLE `yaf_rbac_tags` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `from_id` bigint unsigned NOT NULL COMMENT 'TAG关联来源 ID',
    `from_source` tinyint NOT NULL COMMENT 'TAG关联来源 res role',
    `user_id` bigint unsigned NOT NULL COMMENT 'TAG建立用户ID',
    `name` varchar(32) NOT NULL COMMENT 'TAG名称',
    `status` tinyint NOT NULL COMMENT '状态',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更新用户',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = 'TAG,用于分组资源跟角色';
-- ----------- lsys-rbac  ---------------
-- ----------- lsys-user  ---------------
-- test.yaf_user definition
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
-- test.yaf_user_address definition
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
-- test.yaf_user_email definition
CREATE TABLE `yaf_user_email` (
    `id` int unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint unsigned NOT NULL COMMENT '用户ID',
    `email` varchar(150) NOT NULL COMMENT '邮箱',
    `status` tinyint NOT NULL DEFAULT 1 COMMENT '绑定状态',
    `confirm_time` int unsigned NOT NULL DEFAULT 0 COMMENT '确认时间',
    `change_time` int unsigned NOT NULL DEFAULT 0 COMMENT '最后更改时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '用户关联邮箱';
-- test.yaf_user_external definition
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
-- test.yaf_user_info definition
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
-- test.yaf_user_login definition
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
-- test.yaf_user_mobile definition
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
-- test.yaf_user_name definition
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
-- test.yaf_user_password definition
CREATE TABLE `yaf_user_password` (
    `id` int unsigned NOT NULL AUTO_INCREMENT,
    `user_id` bigint unsigned NOT NULL COMMENT '用户ID',
    `password` varchar(150) NOT NULL COMMENT '密码',
    `add_time` int unsigned NOT NULL COMMENT '绑定时间',
    `disable_time` int unsigned NOT NULL DEFAULT 0 COMMENT '停用时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '用户登录密码';
-- test.yaf_users definition
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
-- ----------- lsys-app  ---------------
CREATE TABLE `yaf_app` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT 'ID',
    `name` varchar(32) NOT NULL COMMENT '应用名称',
    `client_id` varchar(32) NOT NULL COMMENT '应用key',
    `client_secret` varchar(32) NOT NULL COMMENT '应用秘钥',
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
-- ----------- lsys-app  ---------------
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
-- ----------- lsys-sender  ---------------
CREATE TABLE `yaf_sender_config` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID',
    `priority` tinyint DEFAULT 99,
    `sender_type` tinyint NOT NULL COMMENT '发送类型',
    `config_type` tinyint NOT NULL COMMENT '配置类型',
    `config_data` varchar(512) NOT NULL COMMENT '配置数据',
    `status` tinyint NOT NULL COMMENT '启用状态',
    `user_id` bigint unsigned NOT NULL COMMENT '用户id',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后修改用户id',
    `change_time` bigint unsigned NOT NULL COMMENT '最后修改时间',
    PRIMARY KEY (`id`),
    KEY `appid` (`app_id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '发送配置,如发送限额等';
CREATE TABLE `yaf_sender_key_cancel` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID',
    `sender_type` tinyint NOT NULL COMMENT '发送类型',
    `message_id` bigint unsigned NOT NULL COMMENT '消息ID',
    `cancel_key` varchar(32) NOT NULL COMMENT '取消key',
    `status` tinyint NOT NULL COMMENT '取消状态',
    `cancel_user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '用户id',
    `cancel_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '确认时间',
    PRIMARY KEY (`id`),
    KEY `appid` (`app_id`) USING BTREE,
    KEY `message_id` (`message_id`) USING BTREE,
    KEY `sender_type` (`sender_type`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '取消KEY记录';
CREATE TABLE `yaf_sender_log` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `sender_type` tinyint NOT NULL COMMENT '发送类型',
    `message_id` bigint unsigned NOT NULL COMMENT 'ID',
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID',
    `log_type` tinyint NOT NULL COMMENT '日志类型,如发送,取消等',
    `send_note` varchar(32) NOT NULL COMMENT '发送时信息,发送时使用的外部应用ID等',
    `message` varchar(255) NOT NULL COMMENT '发送相关消息',
    `status` tinyint NOT NULL COMMENT '操作状态',
    `user_id` bigint unsigned NOT NULL COMMENT '操作用户id',
    `user_ip` varchar(40) NOT NULL DEFAULT '' COMMENT '操作者IP',
    `request_id` varchar(32) NOT NULL DEFAULT '' COMMENT '请求id',
    `create_time` bigint unsigned NOT NULL COMMENT '创建时间',
    PRIMARY KEY (`id`),
    KEY `appid` (`app_id`) USING BTREE,
    KEY `message_id` (`message_id`) USING BTREE,
    KEY `sender_type` (`sender_type`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '发送日志';
CREATE TABLE `yaf_sender_tpl_config` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `sender_type` tinyint NOT NULL COMMENT '发送类型',
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID',
    `name` varchar(32) NOT NULL COMMENT '名称',
    `tpl_id` varchar(32) NOT NULL COMMENT '模板KEY',
    `setting_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '配置ID',
    `config_data` text NOT NULL COMMENT '配置JSON数据',
    `status` tinyint NOT NULL COMMENT '状态',
    `user_id` bigint unsigned NOT NULL COMMENT '用户id',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更改用户id',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '发送模板配置';
CREATE TABLE `yaf_sender_tpl_body` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `sender_type` tinyint NOT NULL COMMENT '发送类型',
    `tpl_id` varchar(32) NOT NULL COMMENT ' 模板ID',
    `tpl_data` text NOT NULL COMMENT '模板',
    `status` tinyint NOT NULL COMMENT '操作状态',
    `user_id` bigint unsigned NOT NULL COMMENT '操作用户id',
    `change_time` bigint unsigned NOT NULL COMMENT '最后更改时间',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后更改用户id',
    PRIMARY KEY (`id`),
    KEY `tpl_id` (`tpl_id`, `status`) USING BTREE,
    KEY `sender_type` (`sender_type`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '发送模板内容,有些接口不用这个';
CREATE TABLE `yaf_sender_mail_message` (
    `id` bigint unsigned NOT NULL COMMENT 'ID,由应用生成',
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID',
    `to_mail` varchar(254) NOT NULL COMMENT '邮箱',
    `reply_mail` varchar(254) NOT NULL COMMENT '回复',
    `tpl_id` varchar(32) NOT NULL COMMENT '模板ID',
    `tpl_var` varchar(512) NOT NULL DEFAULT '' COMMENT '模板变量',
    `try_num` smallint unsigned NOT NULL DEFAULT 0 COMMENT '发送次数',
    `max_try_num` smallint unsigned NOT NULL DEFAULT 1 COMMENT '最大发送次数',
    `status` tinyint NOT NULL COMMENT '启用状态',
    `add_time` bigint unsigned NOT NULL COMMENT '申请时间',
    `expected_time` bigint unsigned NOT NULL COMMENT '预计发送时间',
    `send_time` bigint unsigned NOT NULL COMMENT '发送时间',
    `user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '用户id',
    PRIMARY KEY (`id`),
    KEY `sender_record_data_IDX` (`expected_time`, `status`, `id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '发送邮件数据';
CREATE TABLE `yaf_sender_sms_message` (
    `id` bigint unsigned NOT NULL COMMENT 'ID,由应用生成',
    `app_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '应用ID',
    `area` varchar(32) NOT NULL COMMENT '区号',
    `mobile` varchar(32) NOT NULL COMMENT '手机号',
    `tpl_id` varchar(32) NOT NULL COMMENT '模板ID',
    `tpl_var` varchar(512) NOT NULL DEFAULT '' COMMENT '模板变量',
    `try_num` smallint unsigned NOT NULL DEFAULT 0 COMMENT '发送次数',
    `max_try_num` smallint unsigned NOT NULL DEFAULT 1 COMMENT '最大发送次数',
    `status` tinyint NOT NULL COMMENT '启用状态',
    `add_time` bigint unsigned NOT NULL COMMENT '申请时间',
    `expected_time` bigint unsigned NOT NULL COMMENT '预计发送时间',
    `send_time` bigint unsigned NOT NULL COMMENT '发送时间',
    `user_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '用户id',
    PRIMARY KEY (`id`),
    KEY `sender_record_data_IDX` (`expected_time`, `status`, `id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '发送短信数据';
-- ----------- lsys-sender  ---------------
-- ----------- lsys-doc  ---------------
CREATE TABLE `yaf_doc_git` (
    `id` int unsigned NOT NULL AUTO_INCREMENT,
    `name` varchar(64) NOT NULL COMMENT '源名称',
    `url` varchar(254) NOT NULL COMMENT 'GIT地址,包含用户名',
    `status` tinyint NOT NULL COMMENT '状态:删除 正常',
    `max_try` tinyint unsigned NOT NULL COMMENT '尝试克隆次数',
    `change_user_id` bigint unsigned NOT NULL COMMENT '最后修改用户',
    `change_time` bigint unsigned NOT NULL COMMENT '最后修改时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '文档GIT源';
CREATE TABLE `yaf_doc_tag` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `doc_git_id` int unsigned NOT NULL COMMENT '文档GIT来源ID',
    `tag` varchar(64) NOT NULL COMMENT 'Tag',
    `build_version` varchar(64) NOT NULL COMMENT '当前使用构建版本',
    `clear_rule` varchar(512) NOT NULL DEFAULT "" COMMENT '清理路径表达式',
    `status` tinyint NOT NULL COMMENT '状态 删除 未启用 已启用',
    `add_user_id` bigint unsigned NOT NULL COMMENT '最后修改用户',
    `add_time` bigint unsigned NOT NULL COMMENT '最后修改时间',
    PRIMARY KEY (`id`),
    KEY `yaf_user_tag_IDX` (`doc_git_id`) USING BTREE,
    KEY `yaf_tag_IDX` (`tag`) USING BTREE,
    KEY `yaf_ver_IDX` (`build_version`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '文档GIT-TAG';
CREATE TABLE `yaf_doc_clone` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `doc_tag_id` bigint unsigned NOT NULL COMMENT '文档GIT tag ID',
    `host` varchar(255) NOT NULL COMMENT '克隆主机',
    `start_time` bigint unsigned NOT NULL COMMENT '最后CLONE开始时间',
    `finish_time` bigint unsigned NOT NULL DEFAULT 0 COMMENT '克隆完成时间',
    `status` tinyint NOT NULL COMMENT '状态:待克隆 已克隆 克隆失败 已删除',
    PRIMARY KEY (`id`),
    KEY `yaf_user_clone_IDX` (`doc_tag_id`, `host`, `status`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '文档GIT CLONE记录';
CREATE TABLE `yaf_doc_menu` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `doc_tag_id` bigint unsigned NOT NULL COMMENT '文档GIT tag ID',
    `menu_path` varchar(254) NOT NULL COMMENT '目录文件路径',
    `menu_check_host` varchar(255) NOT NULL COMMENT '天机是检测通过的机器',
    `status` tinyint NOT NULL COMMENT '状态 正常 删除 ',
    `add_user_id` bigint unsigned NOT NULL COMMENT '修改用户',
    `add_time` bigint unsigned NOT NULL COMMENT '修改时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '文档目录文件配置';
CREATE TABLE `yaf_doc_logs` (
    `id` bigint unsigned NOT NULL AUTO_INCREMENT,
    `doc_tag_id` bigint unsigned NOT NULL COMMENT '文档GIT tag ID',
    `doc_clone_id` bigint unsigned NOT NULL DEFAULT 0 COMMENT '文档GIT来源ID',
    `host` varchar(255) NOT NULL COMMENT '执行时主机名',
    `message` varchar(255) NOT NULL COMMENT '消息内容',
    `add_time` bigint unsigned NOT NULL COMMENT '最后修改时间',
    PRIMARY KEY (`id`)
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '文档日志';
-- ----------- lsys-doc  ---------------