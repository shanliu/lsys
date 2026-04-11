-- ----------- 采集脚本配置  ---------------
CREATE TABLE `lst_collector_script` (
    `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `add_user_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '创建用户ID',
    `app_user_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '应用关联用户ID，仅冗余，不做过滤',
    `app_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '应用ID,0=系统',
    `name` VARCHAR(100) NOT NULL COMMENT '脚本名称（唯一标识，用作文件TAG）',
    `script_code` MEDIUMTEXT NOT NULL COMMENT 'JS脚本代码',
    `script_md5` CHAR(32) NOT NULL DEFAULT '' COMMENT '脚本代码MD5',
    `timeout_secs` INT UNSIGNED NOT NULL DEFAULT 30 COMMENT '执行超时秒数',
    `memory_limit` BIGINT UNSIGNED NOT NULL DEFAULT 67108864 COMMENT '内存限制(字节),默认64MB',
    `status` TINYINT NOT NULL DEFAULT 1 COMMENT '状态:1=启用,2=禁用,3=已删除',
    `add_time` BIGINT UNSIGNED NOT NULL COMMENT '创建时间',
    `change_time` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '修改时间',
    UNIQUE KEY `uk_user_app_name` (`add_user_id`,`app_id`,`name`,`status`) USING BTREE,
    KEY `idx_app_status` (`app_id`,`status`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '采集脚本配置';

-- ----------- 采集执行记录  ---------------
CREATE TABLE `lst_collector_record` (
    `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `request_id` VARCHAR(64) NOT NULL COMMENT '请求ID（来自RequestEnv.request_id或自动生成）',
    `script_id` BIGINT UNSIGNED NOT NULL COMMENT '脚本ID',
    `add_user_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '触发用户ID',
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
    KEY `idx_user_app_time` (`add_user_id`,`app_id`,`add_time`) USING BTREE,
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

-- ----------- 数据导出任务  ---------------
CREATE TABLE `lst_export_task` (
    `id`             BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `app_id`         BIGINT UNSIGNED NOT NULL DEFAULT 0
                         COMMENT '应用ID，user_id=0 app_id=0=系统，user_id>0 app_id=0 用户本身 user_id>0 app_id>0 用户应用',
    `app_user_id`    BIGINT UNSIGNED NOT NULL DEFAULT 0
                         COMMENT '应用关联用户ID，仅冗余 app 维度的用户信息，不做过滤',
    `user_id`        BIGINT UNSIGNED NOT NULL DEFAULT 0
                         COMMENT '导出属于用户ID（必须，系统时为0，用户端为当前登录用户ID）',
    `add_user_id`    BIGINT UNSIGNED NOT NULL DEFAULT 0
                         COMMENT '创建导出的用户ID（用户端=user_id，系统端=实际操作的管理员ID）',
    `export_type`    VARCHAR(64)     NOT NULL DEFAULT ''
                         COMMENT '导出类型标识，对应多语言 key（如 collector_record / user_list 等）',
    `export_params`  TEXT            NOT NULL
                         COMMENT '导出参数 JSON（过滤条件/权限由各 Exporter 实现层校验）',
    `status`         TINYINT         NOT NULL DEFAULT 1
                         COMMENT '状态: 1=Pending 2=Running 3=Success 4=Failed 5=Deleted',
    `error_message`  TEXT            NOT NULL
                         COMMENT '失败时的错误信息',
    `add_time`       BIGINT UNSIGNED NOT NULL DEFAULT 0
                         COMMENT '提交并开始时间（提交即执行，无需拆分）',
    `change_time`    BIGINT UNSIGNED NOT NULL DEFAULT 0
                         COMMENT '最后状态变更时间（完成/失败/删除均更新），0=未变更',
    `request_id`     VARCHAR(64)    NOT NULL DEFAULT ''
                         COMMENT '请求 ID（用于追踪和关联请求）',
    KEY `idx_user_app_time`    (`user_id`, `app_id`, `add_time`) USING BTREE COMMENT '用户维度列表分页（user_id必须，app_id可选）',
    KEY `idx_export_type_time` (`export_type`, `add_time`)      USING BTREE COMMENT '按类型统计/列表',
    KEY `idx_status_add_time`  (`status`, `add_time`)           USING BTREE COMMENT '超时检测：status=Running AND add_time<阈值'
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '数据导出任务';
