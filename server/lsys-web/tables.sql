-- ----------- 初始化账号  ---------------
INSERT INTO lst_account (
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
INSERT INTO lst_account_name(account_id, username, change_time, status)
VALUES(1, 'aaaaa', UNIX_TIMESTAMP(), 1);
-- ----------- 密码:000000  ---------------
INSERT INTO lst_account_password (account_id, password, add_time, disable_time)
VALUES (
        1,
        '670b14728ad9902aecba32e22fa4f6bd',
        UNIX_TIMESTAMP(),
        0
    );
-- ----------- 用户可搜索  ---------------
INSERT INTO lst_account_index (
        account_id,
        index_cat,
        index_data,
        status,
        change_time
    )
VALUES (1, 5, 'root', 1, UNIX_TIMESTAMP()),
    (1, 6, 'aaaaa', 1, UNIX_TIMESTAMP()),
    (1, 7, '2', 1, UNIX_TIMESTAMP());

-- ----------- 关联 lsys-access 设置 user_data = lst_account.id  ---------------
INSERT INTO lst_user (
        id,
        app_id,
        user_data,
        user_account,
        user_nickname,
        change_time
    )
VALUES(1, 0, '1', 'aaaaa', 'root', UNIX_TIMESTAMP());

-- ----------- 采集脚本配置  ---------------
CREATE TABLE `lst_collector_script` (
    `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `user_id` BIGINT UNSIGNED NOT NULL COMMENT '创建用户ID',
    `app_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '应用ID,0=系统',
    `name` VARCHAR(100) NOT NULL COMMENT '脚本名称（唯一标识，用作文件TAG）',
    `script_code` MEDIUMTEXT NOT NULL COMMENT 'JS脚本代码',
    `script_md5` CHAR(32) NOT NULL DEFAULT '' COMMENT '脚本代码MD5',
    `timeout_secs` INT UNSIGNED NOT NULL DEFAULT 30 COMMENT '执行超时秒数',
    `memory_limit` BIGINT UNSIGNED NOT NULL DEFAULT 67108864 COMMENT '内存限制(字节),默认64MB',
    `status` TINYINT NOT NULL DEFAULT 1 COMMENT '状态:1=启用,2=禁用,3=已删除',
    `add_time` BIGINT UNSIGNED NOT NULL COMMENT '创建时间',
    `change_time` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '修改时间',
    UNIQUE KEY `uk_user_app_name` (`user_id`,`app_id`,`name`,`status`) USING BTREE,
    KEY `idx_app_status` (`app_id`,`status`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '采集脚本配置';

-- ----------- 采集执行记录  ---------------
CREATE TABLE `lst_collector_record` (
    `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `request_id` VARCHAR(64) NOT NULL COMMENT '请求ID（来自RequestEnv.request_id或自动生成）',
    `script_id` BIGINT UNSIGNED NOT NULL COMMENT '脚本ID',
    `user_id` BIGINT UNSIGNED NOT NULL COMMENT '触发用户ID',
    `app_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '应用ID',
    `task_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT 'JsTaskRunner分配的task_id',
    `exec_params` TEXT NOT NULL COMMENT '执行参数(JSON)',
    `status` TINYINT NOT NULL DEFAULT 1 COMMENT '状态:1=Pending,2=Running,3=Success,4=Failed,5=Timeout',
    `elapsed_ms` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '执行耗时(毫秒)',
    `error_message` TEXT NOT NULL COMMENT '错误信息',
    `add_time` BIGINT UNSIGNED NOT NULL COMMENT '提交时间',
    `start_time` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '开始执行时间',
    `finish_time` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '完成时间',
    UNIQUE KEY `uk_request_id` (`request_id`) USING BTREE,
    KEY `idx_script_time` (`script_id`,`add_time`) USING BTREE,
    KEY `idx_user_app_time` (`user_id`,`app_id`,`add_time`) USING BTREE,
    KEY `idx_status` (`status`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '采集执行记录';

-- ----------- 采集日志（JS脚本日志 + 系统关键操作日志）  ---------------
CREATE TABLE `lst_collector_log` (
    `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `request_id` VARCHAR(64) NOT NULL COMMENT '请求ID（关联lst_collector_record.request_id）',
    `script_id` BIGINT UNSIGNED NOT NULL COMMENT '脚本ID',
    `user_id` BIGINT UNSIGNED NOT NULL COMMENT '触发用户ID',
    `app_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '应用ID',
    `level` TINYINT UNSIGNED NOT NULL COMMENT '等级:0=Trace,1=Debug,2=Info,3=Warn,4=Error,10=System',
    `message` TEXT NOT NULL COMMENT '日志消息',
    `add_time` BIGINT UNSIGNED NOT NULL COMMENT '写入时间',
    KEY `idx_request_id` (`request_id`) USING BTREE,
    KEY `idx_script_time` (`script_id`,`add_time`) USING BTREE,
    KEY `idx_user_app_time` (`user_id`,`app_id`,`add_time`) USING BTREE,
    KEY `idx_level` (`level`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '采集日志';