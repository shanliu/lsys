-- MFA (TOTP) tables
--
-- Notes:
-- - Uses (app_id, user_data) as the subject key to support both internal (app_id=0) and external accounts.
-- - Keeps history: enabling a new secret inserts a new row; older enabled rows should be marked disabled.
-- - Reads the latest enabled row by `ORDER BY id DESC LIMIT 1`.

CREATE TABLE IF NOT EXISTS yaf_mfa_totp (
    `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,

    `app_id` BIGINT UNSIGNED NOT NULL DEFAULT 0,
    `user_data` VARCHAR(32) NOT NULL DEFAULT '',

    -- 1: enabled, 0: disabled
    `status` TINYINT NOT NULL DEFAULT 1,

    -- Base32 secret (or encrypted/encoded secret string if you later add encryption)
    `secret_data` VARCHAR(128) NOT NULL DEFAULT '',

    -- TOTP anti-replay: last accepted time-step (e.g. floor(now/30))
    `last_used_step` BIGINT UNSIGNED NOT NULL DEFAULT 0,
    `last_used_time` BIGINT UNSIGNED NOT NULL DEFAULT 0,

    `add_time` BIGINT UNSIGNED NOT NULL DEFAULT 0,
    `change_time` BIGINT UNSIGNED NOT NULL DEFAULT 0,

    PRIMARY KEY (`id`),
    KEY `idx_subject_status` (`app_id`, `user_data`, `status`, `id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
