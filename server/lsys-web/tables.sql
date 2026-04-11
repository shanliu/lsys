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

-- ----------- 验证码配置 ---------------
INSERT INTO lst_setting (id,setting_type,name,setting_key,setting_data,user_id,status,change_user_id,change_time) VALUES
	 (4,2,'111211','smtp-config','{"host":"smtp.qq.com","port":465,"timeout":30,"user":"rustlang@qq.com","email":"rustlang@qq.com","password":"","tls_domain":"","branch_limit":1}',0,1,1,0);

INSERT INTO lst_sender_tpl_body (sender_type,tpl_id,tpl_data,status,user_id,change_time,change_user_id) VALUES
	 (2,'valid_code_body','验证码是 {{code}} ,请在 {{ttl|second_format(i="分钟")}} 内使用',1,0,1747994349,1),
	 (2,'valid_code_title','验证码',1,0,1747988835,1);

INSERT INTO lst_sender_tpl_config (sender_type,app_id,name,tpl_key,setting_id,config_data,status,user_id,change_time,change_user_id) VALUES
	 (2,0,'账号新增邮箱','valid_code_account_email',4,'{"body_tpl_id":"valid_code_body","from_email":"rustlang@qq.com","reply_email":"rustlang@qq.com","subject_tpl_id":"valid_code_title"}',1,0,1747995342,1),
	 (1,0,'账号新增手机号','valid_code_account_mobile',4,'{"body_tpl_id":"valid_code_body","from_email":"rustlang@qq.com","reply_email":"rustlang@qq.com","subject_tpl_id":"valid_code_title"}',1,0,1747995356,1),
	 (1,0,'短信重置密码','valid_code_reset_password_mobile',4,'{"body_tpl_id":"valid_code_body","from_email":"rustlang@qq.com","reply_email":"rustlang@qq.com","subject_tpl_id":"valid_code_title"}',1,0,1747995380,1),
	 (2,0,'邮件重置密码','valid_code_reset_password_email',4,'{"body_tpl_id":"valid_code_body","from_email":"rustlang@qq.com","reply_email":"rustlang@qq.com","subject_tpl_id":"valid_code_title"}',1,0,1747995396,1),
	 (1,0,'短信注册','valid_code_register_mobile',4,'{"body_tpl_id":"valid_code_body","from_email":"rustlang@qq.com","reply_email":"rustlang@qq.com","subject_tpl_id":"valid_code_title"}',1,0,1747995410,1),
	 (2,0,'邮箱注册','valid_code_register_email',4,'{"body_tpl_id":"valid_code_body","from_email":"rustlang@qq.com","reply_email":"rustlang@qq.com","subject_tpl_id":"valid_code_title"}',1,0,1747995420,1),
	 (2,0,'邮箱登录','valid_code_login_email',4,'{"body_tpl_id":"valid_code_body","from_email":"rustlang@qq.com","reply_email":"rustlang@qq.com","subject_tpl_id":"valid_code_title"}',1,0,1747995433,1),
	 (1,0,'短信登录','valid_code_login_mobile',4,'{"body_tpl_id":"valid_code_body","from_email":"rustlang@qq.com","reply_email":"rustlang@qq.com","subject_tpl_id":"valid_code_title"}',1,0,1747995440,1);
