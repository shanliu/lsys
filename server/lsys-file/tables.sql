CREATE TABLE `lst_file` (
	`id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `storage_type` VARCHAR(32) NOT NULL COMMENT '文件存储类型：local_*=本地存储, 其他值为OSS配置的config_key',
    `status` TINYINT NOT NULL COMMENT '状态：1=正常, 2=已删除,3=未完成,4=失败',
    `origin_name` VARCHAR(255) NOT NULL DEFAULT '' COMMENT '文件首次入库时的原始文件名，用于生成存储路径/OSS key，不随用户重命名变化',
    `file_md5` CHAR(32) NOT NULL DEFAULT '' COMMENT '文件md5',
    `file_size` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '文件大小',
    `modify_time` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '文件最后修改时间',
    `content_type` VARCHAR(128) NOT NULL DEFAULT '' COMMENT '文件MIME类型',
    `local_path_owner_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '本地物理路径持有者：0=当前记录独立拥有磁盘文件，>0=与该file_id共享相同本地路径（上传去重时产生，仅local_*类型有效）',
    `from_user_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '上传用户ID',
    `add_time` BIGINT UNSIGNED NOT NULL COMMENT '添加时间',
    `change_time` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '记录更新时间',
    KEY `file_md5` (`file_md5`) USING BTREE,
    KEY `file_ref_user_time` (`from_user_id`,`add_time`) USING BTREE,
    KEY `file_storage_type` (`storage_type`) USING BTREE,
    KEY `local_path_owner_id` (`local_path_owner_id`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '文件列表';

CREATE TABLE `lst_file_local` (
	`id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `file_id` BIGINT UNSIGNED NOT NULL COMMENT '文件ID',
    `source_type` TINYINT NOT NULL COMMENT '文件来源：1=上传, 2=URL下载, 3=本地路径,4=OSS同步',
	`source_name` VARCHAR(255) NOT NULL  DEFAULT '' COMMENT '文件原始文件名',
    `local_path` VARCHAR(512) NOT NULL COMMENT '文件存储路径',

    `file_chunk_total` INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '分片总数,上传或URL下载时分片总数,<=1时不分片',
	`file_chunk_succ` INT UNSIGNED NOT NULL DEFAULT 0 COMMENT '已完成分片数',
	`file_chunk_size` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '已完成分片大小汇总,分片完成时累加,用于列表展示进度',

    
    `last_error` VARCHAR(255) NOT NULL DEFAULT '' COMMENT '最后一次同步错误消息,方便列表查看',
    UNIQUE KEY `file_id` (`file_id`) USING BTREE,
    KEY `file_source_type` (`source_type`) USING BTREE,
    KEY `local_path` (`local_path`(256)) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '本地文件';


CREATE TABLE `lst_file_local_chunk` (
	`id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
	`file_id` BIGINT UNSIGNED NOT NULL COMMENT '文件ID',
	`chunk_index` INT UNSIGNED NOT NULL COMMENT '分片索引(从0开始)',
	`start_offset` BIGINT UNSIGNED NOT NULL COMMENT '分片起始字节',
    `chunk_md5` CHAR(32) NOT NULL DEFAULT '' COMMENT '文件md5',
	`upload_md5` CHAR(32) NOT NULL DEFAULT '' COMMENT '上传时文件hash,用户提交',
    `chunk_path` VARCHAR(512) NOT NULL COMMENT '分片文件存储路径',
    `file_size` BIGINT UNSIGNED NOT NULL COMMENT '预计文件大小',
	`complete_size` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '分片完成时写入的实际文件大小,由文件系统metadata读取',
    `status` TINYINT NOT NULL COMMENT '状态：1=正常, 2=已删除,3=未完成,4=失败,5=已合并,6=已清理',
    `add_time` BIGINT UNSIGNED NOT NULL COMMENT '添加时间',
    `change_time` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '修改时间',
    UNIQUE KEY `file_chunk_index` (`file_id`,`chunk_index`) USING BTREE,
    KEY `file_chunk_status` (`file_id`,`status`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '本地文件 上传 或 URL下载分块(多节点同时读写,多个文件不会有并发问题)';


CREATE TABLE `lst_file_oss` (
  `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  `file_id` BIGINT UNSIGNED NOT NULL COMMENT '关联 lst_file.id',
  `object_key` VARCHAR(1024) NOT NULL COMMENT '对象键（路径）',
  `object_url` VARCHAR(2048) NOT NULL DEFAULT '' COMMENT '对外访问 URL（若可公开访问/或 CDN 地址）',
  `object_url_md5` CHAR(32) NOT NULL DEFAULT '' COMMENT '对外访问 URL md5 方便查询',
  `bucket` VARCHAR(255) NOT NULL DEFAULT '' COMMENT '目标 bucket 如果OSS支持',
  `region` VARCHAR(128) NOT NULL DEFAULT '' COMMENT '区域/endpoint 信息（可选）如果OSS支持',
  `size` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '对象大小（字节）',
  `last_error` VARCHAR(255) NOT NULL DEFAULT '' COMMENT '最后一次同步错误消息,方便列表查看',
   UNIQUE KEY `file_id` (`file_id`) USING BTREE,
    KEY `oss_url_md5` (`object_url_md5`) USING BTREE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='OSS远程文件';

CREATE TABLE `lst_file_log` (
	`id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
	`file_id` BIGINT UNSIGNED NOT NULL COMMENT '文件ID',
    `file_chunk_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '分片ID',
	`message` VARCHAR(1024) NOT NULL COMMENT '日志内容',
	`user_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '上传用户ID',
    `add_time` BIGINT UNSIGNED NOT NULL  COMMENT '添加时间',
    KEY `file_log_file_time` (`file_id`,`add_time`) USING BTREE
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '文件日志';


CREATE TABLE `lst_file_ref` (
	`id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `user_id` BIGINT UNSIGNED NOT NULL COMMENT '文件属于的用户ID,0=系统',
    `add_user_id` BIGINT UNSIGNED NOT NULL COMMENT '文件添加(上传)用户ID',
    `app_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '应用ID,0=系统,>0=具体应用',
    `file_id` BIGINT UNSIGNED NOT NULL COMMENT '文件ID',
    `status` TINYINT NOT NULL DEFAULT 1 COMMENT '状态：1=正常, 2=已删除',
	`source_url` VARCHAR(2048) NOT NULL  DEFAULT '' COMMENT '来源URL,默认同个用户的URL下载时相同URL只记录一次,但可多次下载多个记录',
    `source_md5` CHAR(32) NOT NULL  DEFAULT '' COMMENT '来源URL hash,上传时用户提交的hash',
    `trigger_host` VARCHAR(255) NOT NULL DEFAULT '' COMMENT '触发下载的主机标识(用于多节点下载完成通知)',
    `add_time` BIGINT UNSIGNED NOT NULL COMMENT '添加时间',
    `delete_time` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT  '软删除时间',
    `expire_time` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '过期时间，0表示永不过期',
    `file_name` VARCHAR(255) NOT NULL DEFAULT '' COMMENT '用户自定义文件名，可独立修改，不影响其他用户对同一文件的命名',
    KEY `user_source_md5` (`user_id`,`app_id`,`source_md5`) USING BTREE,
    KEY `file_ref_user_status_time` (`user_id`,`app_id`,`file_id`,`add_time`,`status`) USING BTREE,
    KEY `file_id` (`file_id`) USING BTREE COMMENT '用于JOIN查询',
    KEY `idx_expire_time` (`expire_time`,`status`) USING BTREE COMMENT '过期任务扫描：expire_time>0 AND expire_time<=now',
    KEY `source_url_download` (`source_url`(12), `id`) USING BTREE COMMENT '用于下载任务查询'
) ENGINE = InnoDB CHARSET = utf8mb4 COMMENT = '文件引用用户列表';


CREATE TABLE `lst_file_tag` (
    `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `file_id` BIGINT UNSIGNED NOT NULL COMMENT '文件ID (lst_file.id)',
    `user_id` BIGINT UNSIGNED NOT NULL COMMENT '用户ID',
    `app_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '应用ID',
    `tag_name` VARCHAR(100) NOT NULL COMMENT '标签名称（归一化存储：trim+小写）',
    `status` TINYINT NOT NULL DEFAULT 1 COMMENT '状态：1=正常, 2=已删除',
    `add_time` BIGINT UNSIGNED NOT NULL COMMENT '添加时间',
    `change_time` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '最后修改时间',
    KEY `idx_file_ref_user_app_tag` (`file_id`, `user_id`, `app_id`, `tag_name`, `status`) USING BTREE,
    KEY `idx_tag_name_status` (`tag_name`, `status`, `app_id`, `file_id`) USING BTREE,
    KEY `idx_user_app_status` (`user_id`, `app_id`, `status`) USING BTREE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='文件标签关联表';


CREATE TABLE `lst_file_lineage` (
    `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    `user_id` BIGINT UNSIGNED NOT NULL COMMENT '触发操作的用户 ID',
    `app_id` BIGINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '应用 ID',
    `src_file_id` BIGINT UNSIGNED NOT NULL COMMENT '来源文件 ID (lst_file.id)',
    `dst_file_id` BIGINT UNSIGNED NOT NULL COMMENT '派生文件 ID (lst_file.id)',
    `rel_type` TINYINT NOT NULL COMMENT '关系类型：1=主动拷贝(独立物理文件), 2=本地类型转换(public/private/crypto互转,方向由src/dst的storage_type决定), 3=OSS↔本地同步(双向,方向由src/dst的storage_type决定)',
    `status` TINYINT NOT NULL DEFAULT 1 COMMENT '状态：1=正常, 2=已删除',
    `add_time` BIGINT UNSIGNED NOT NULL COMMENT '添加时间',
    KEY `idx_src_user` (`src_file_id`, `status`, `rel_type`, `user_id`, `app_id`) USING BTREE,
    KEY `idx_dst` (`dst_file_id`) USING BTREE,
    KEY `idx_user` (`user_id`, `app_id`, `status`) USING BTREE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='文件派生关系表（用户维度隔离，同一文件不同用户各有独立的关系记录）';



