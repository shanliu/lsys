-- ----------- 初始化账号  ---------------
INSERT INTO yaf_account (
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
INSERT INTO yaf_account_name(account_id, username, change_time, status)
VALUES(1, 'aaaaa', UNIX_TIMESTAMP(), 1);
-- ----------- 密码:000000  ---------------
INSERT INTO yaf_account_password (account_id, password, add_time, disable_time)
VALUES (
        1,
        '670b14728ad9902aecba32e22fa4f6bd',
        UNIX_TIMESTAMP(),
        0
    );
-- ----------- 用户可搜索  ---------------
INSERT INTO yaf_account_index (
        account_id,
        index_cat,
        index_data,
        status,
        change_time
    )
VALUES (1, 5, 'root', 1, UNIX_TIMESTAMP()),
    (1, 6, 'aaaaa', 1, UNIX_TIMESTAMP()),
    (1, 7, '2', 1, UNIX_TIMESTAMP());

-- ----------- 关联 lsys-access 设置 user_data = yaf_account.id  ---------------
INSERT INTO yaf_user (
        id,
        app_id,
        user_data,
        user_account,
        user_nickname,
        change_time
    )
VALUES(1, 0, '1', 'aaaaa', 'root', UNIX_TIMESTAMP());

