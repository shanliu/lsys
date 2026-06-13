 
mail-send-check = 邮箱发送错误:{$msg}
user-external-login-url = 加载外部账号登陆url失败:{$msg}
user-external-call = 请求外部账号信息:{$msg}
user-external-other-bind = 账号{$name}已绑定其他用户:{$name}
rbac-unkown-res = 资源{$res}不存在
address-miss-city = 地址请选择到县区一级
address-bad-area = 提交的区域编码不存在
mail-bind-other-user = 邮箱已绑定其他账号({$other_user_id})
db-not-found = 不存在相关记录
db-error = sqlx错误:{$msg}
user-old-passwrod-bad = 提交的原密码错误
user-old-passwrod-empty = 请提交原密码
mail-is-confirm = 邮箱已经确认过了
username-is-exists = 账号已经存在:{$id}
password-not-set = 登陆密码未设置
client-secret-not-match = Secret 不匹配
app-redirect-uri-not-match = 跳转域名(redirect_uri)不支持
reg-mobile-registered = 该手机号已注册过账号
mobile-bind-other-user = 邮箱已绑定其他账号:{$id}
mobile-is-bind = 邮箱重复绑定
lsys-lib-area-error = 地址库异常:{$msg}
lsys-lib-area-db = 地址数据异常:{$msg}
area-not-found = 地址信息未找到
area-store-error = 地址存储异常:{$msg}
area-tantivy-error = 地址搜索异常:{$msg}
area-not-enable = 地址库未启用
bad-session-data = 登陆数据类型不支持
auth-need-captcha = Code登陆请输入验证码
app-oauth-login-bad-scope = 应用不存在授权:{$scope_data}
not-system-app-confirm = 非系统应用
role-perm-bad-op = 资源操作id:{$op_id}数据丢失或不可用
role-perm-bad-res = 资源id:{$res_id}数据丢失或不可用
role-user-not-system-user = 用户{$user_name}({$user_id})非系统用户，属于应用:{$app_id}
role-user-not-found = 无法添加用户ID{$user_id}到角色,用户id不存在
not-user-app-confirm = 非子应用
app-is-subapp = 该应用为子应用
bad-audit-access = 非法访问审计数据
bad-app-id = 非法操作非自身app  
access-token-data-token-code-exits =登陆时发现 token_code 已使用，请更换
app-notify-only-parent = 回调通知只支持系统应用
oauth-user-not-match=OAuth不能跨用户进行授权(用户应用ID:{$user_app_id},OAuth属于应用:{$oauth_parent_app_id})
#rbac 权限
res-admin-global-system = 系统后台权限
res-op-admin-main = 查看系统后台
res-op-admin-view-app = 查看应用
res-op-admin-edit-app = 编辑应用
res-op-admin-view-docs = 查看文档配置
res-op-admin-edit-docs = 编辑文档配置
res-op-admin-view-rbac = 查看权限配置
res-op-admin-edit-rbac = 编辑权限配置
res-op-admin-sys-sms-config = 短信应用配置
res-op-admin-sys-sms-mgr = 短信应用管理
res-op-admin-sys-mail-config = 邮件应用配置
res-op-admin-sys-mail-mgr = 邮件应用管理
res-op-admin-site-setting = 站点配置
res-op-admin-manage-user = 用户管理
res-op-admin-see-change-log = 查看变更日志
res-op-admin-sys-sms-manage = 短信设置管理
res-op-admin-sys-sms-send = 短信发送设置
res-op-admin-sys-mail-manage = 邮件设置管理
res-op-admin-sys-mail-send = 邮件发送设置
res-admin-global-public = 系统公共权限
res-op-admin-register = 用户注册
res-op-admin-login = 用户登录
res-admin-global-app = 应用管理(系统)
res-admin-global-user = 用户权限
res-op-admin-rest = 接口访问权限(系统)
res-op-rest = 应用接口访问

res-user-global-user = 用户全局权限
res-op-admin-address-edit = 用户收货地址编辑
res-op-admin-email-edit = 用户邮箱编辑
res-op-admin-info-edit = 用户信息编辑
res-op-admin-mobile-edit = 用户手机号编辑
res-op-admin-app-view = 用户查看应用
res-op-admin-app-edit = 用户编辑应用
res-op-admin-notify-view = 用户查看回调通知
res-op-admin-rbac-view = 用户权限检测
res-op-admin-rbac-check = 用户权限检测
res-op-admin-rbac-edit = 用户权限编辑
res-op-admin-external-edit = 绑定账号管理
res-op-admin-app-mail-config = 用户邮件应用配置
res-op-admin-app-mail-view = 用户邮件应用查看
res-op-admin-app-mail-manage = 用户邮件应用管理
res-op-admin-app-mail-send = 用户邮件应用发送
res-op-admin-app-sms-config = 用户短信应用配置
res-op-admin-app-sms-view = 用户短信应用查看
res-op-admin-app-sms-manage = 用户短信应用管理
res-op-admin-app-sms-send = 用户短信应用发送



#校验名称
valid-rule-name-area_code = 地区编码


#字典 
const-SMS_NOTIFY_METHOD = 短信发送结果回调
const-SUB_APP_SECRET_NOTIFY_METHOD = 子应用密钥更改回调


#字典 
const-APP_FEATURE_MAIL = 邮件服务
const-APP_FEATURE_RBAC = 权限服务
const-APP_FEATURE_SMS = 短信服务
const-APP_FEATURE_FILE = 文件服务

var-login-type-email = 邮箱
var-login-type-email-code = 邮箱验证码
var-login-type-name = 账号
var-login-type-mobile = 手机号
var-login-type-mobile-code = 短信验证码
var-login-type-external = 外部账号


var-smtp-config= SMTP服务器配置
var-ali-sms-config= 阿里云短信
var-col-sms-config= 融联云短信
var-hwyun-sms-config= 华为云短信
var-jd-cloud-sms-config= 京东云短信
var-163-sms-config= 网易云短信
var-tenyun-sms-config= 腾讯云短信






var-app = 应用操作
var-app-request = 应用请求处理
var-app-oauth-client-set = 应用oauth设置
var-app-oauth-client-secret-set = 应用oauth密钥修改
var-app-oauth-server-set = 应用oauth服务设置
var-app-view-secret = 应用查看密钥
var-app-notify-set = 应用回调设置
var-app-notify-del = 应用回调删除
var-account-address =  账号地址修改
var-account-email = 账号邮箱修改
var-account-external =  账号关联外部账号
var-account-info = 设置账号信息
var-account-mobile = 账号手机修改
var-account-name =  账号名称修改
var-user =  用户信息
var-set-password = 账号设置密码
var-rbac-op = 权限资源操作管理
var-rbac-res = 权限资源管理
var-rbac-res-op = 权限资源关联操作
var-rbac-role = 权限角色管理
var-rbac-role-user =权限角色关联用户 
var-rbac-role-perm = 权限角色关联权限
var-setting = 设置修改
var-sender-message = 发送消息处理
var-sender-tpl = 发送模版管理
var-sender-app-config = 发送应用配置
var-sender-config = 发送配置
var-message-view = 发送消息查看
# 文件存储类型
# CSV导出列头

# user_file_list
export-user_file_list-id = ID
export-user_file_list-file_name = 文件名
export-user_file_list-file_md5 = 文件MD5
export-user_file_list-file_size = 文件大小
export-user_file_list-storage_type = 存储类型
export-user_file_list-status = 状态
export-user_file_list-content_type = 内容类型
export-user_file_list-add_time = 创建时间

# user_login_history
export-user_login_history-id = ID
export-user_login_history-login_type = 登录类型
export-user_login_history-login_account = 登录账号
export-user_login_history-login_ip = 登录IP
export-user_login_history-login_city = 登录城市
export-user_login_history-account_id = 账号ID
export-user_login_history-is_login = 是否登录
export-user_login_history-login_msg = 登录消息
export-user_login_history-add_time = 创建时间

# user_mailer_message_list
export-user_mailer_message_list-id = ID
export-user_mailer_message_list-snid = 发送批次
export-user_mailer_message_list-to_mail = 收件邮箱
export-user_mailer_message_list-try_num = 发送次数
export-user_mailer_message_list-status = 状态
export-user_mailer_message_list-add_time = 创建时间
export-user_mailer_message_list-send_time = 发送时间
export-user_mailer_message_list-setting_id = 配置ID

# user_smser_message_list
export-user_smser_message_list-id = ID
export-user_smser_message_list-snid = 发送批次
export-user_smser_message_list-area = 区号
export-user_smser_message_list-mobile = 手机号
export-user_smser_message_list-try_num = 发送次数
export-user_smser_message_list-status = 状态
export-user_smser_message_list-add_time = 创建时间
export-user_smser_message_list-send_time = 发送时间
export-user_smser_message_list-setting_id = 配置ID

# app_notify_list
export-app_notify_list-id = ID
export-app_notify_list-app_id = 应用ID
export-app_notify_list-notify_type = 通知类型
export-app_notify_list-notify_method = 通知方式
export-app_notify_list-notify_key = 通知键
export-app_notify_list-status = 状态
export-app_notify_list-try_num = 发送次数
export-app_notify_list-try_max = 最大重试次数
export-app_notify_list-publish_time = 发布时间
export-app_notify_list-next_time = 下次时间

# app_script_records
export-app_script_records-id = ID
export-app_script_records-request_id = 请求ID
export-app_script_records-script_id = 脚本ID
export-app_script_records-add_user_id = 创建用户ID
export-app_script_records-app_id = 应用ID
export-app_script_records-status = 状态
export-app_script_records-elapsed_ms = 耗时(ms)
export-app_script_records-error_message = 错误信息
export-app_script_records-add_time = 创建时间
export-app_script_records-start_time = 开始时间
export-app_script_records-finish_time = 完成时间

# app_file_list
export-app_file_list-id = ID
export-app_file_list-file_name = 文件名
export-app_file_list-file_md5 = 文件MD5
export-app_file_list-file_size = 文件大小
export-app_file_list-storage_type = 存储类型
export-app_file_list-status = 状态
export-app_file_list-content_type = 内容类型
export-app_file_list-add_time = 创建时间

# app_role_data
export-app_role_data-role_id = 角色ID
export-app_role_data-role_key = 角色键
export-app_role_data-role_name = 角色名称
export-app_role_data-user_range = 用户范围
export-app_role_data-res_range = 资源范围
export-app_role_data-user_id = 用户ID
export-app_role_data-user_timeout = 用户超时
export-app_role_data-op_key = 操作键
export-app_role_data-op_name = 操作名称
export-app_role_data-res_type = 资源类型
export-app_role_data-res_data = 资源数据
export-app_role_data-res_name = 资源名称

# app_res_data
export-app_res_data-id = ID
export-app_res_data-user_id = 用户ID
export-app_res_data-res_type = 资源类型
export-app_res_data-res_data = 资源数据
export-app_res_data-res_name = 资源名称
export-app_res_data-change_time = 修改时间
export-app_res_data-op_count = 操作数量
export-app_res_data-perm_count = 权限数量

# system_admin_file_list
export-system_admin_file_list-id = ID
export-system_admin_file_list-file_name = 文件名
export-system_admin_file_list-file_md5 = 文件MD5
export-system_admin_file_list-file_size = 文件大小
export-system_admin_file_list-storage_type = 存储类型
export-system_admin_file_list-status = 状态
export-system_admin_file_list-content_type = 内容类型
export-system_admin_file_list-user_id = 用户ID
export-system_admin_file_list-add_time = 创建时间

# system_user_change_log
export-system_user_change_log-id = ID
export-system_user_change_log-log_type = 日志类型
export-system_user_change_log-add_user_id = 创建用户ID
export-system_user_change_log-log_data = 日志数据
export-system_user_change_log-add_time = 创建时间

# system_user_access
export-system_user_access-id = ID
export-system_user_access-app_id = 应用ID
export-system_user_access-oauth_app_id = OAuth应用ID
export-system_user_access-user_id = 用户ID
export-system_user_access-token_data = Token数据
export-system_user_access-login_type = 登录类型
export-system_user_access-login_ip = 登录IP
export-system_user_access-status = 状态
export-system_user_access-add_time = 创建时间
export-system_user_access-expire_time = 过期时间
export-system_user_access-logout_time = 退出时间

# system_mailer_message_list
export-system_mailer_message_list-id = ID
export-system_mailer_message_list-snid = 发送批次
export-system_mailer_message_list-to_mail = 收件邮箱
export-system_mailer_message_list-try_num = 发送次数
export-system_mailer_message_list-status = 状态
export-system_mailer_message_list-add_time = 创建时间
export-system_mailer_message_list-send_time = 发送时间
export-system_mailer_message_list-setting_id = 配置ID

# system_smser_message_list
export-system_smser_message_list-id = ID
export-system_smser_message_list-snid = 发送批次
export-system_smser_message_list-area = 区号
export-system_smser_message_list-mobile = 手机号
export-system_smser_message_list-try_num = 发送次数
export-system_smser_message_list-status = 状态
export-system_smser_message_list-add_time = 创建时间
export-system_smser_message_list-send_time = 发送时间
export-system_smser_message_list-setting_id = 配置ID

# system_app_list
export-system_app_list-id = ID
export-system_app_list-name = 名称
export-system_app_list-client_id = 客户端ID
export-system_app_list-status = 状态
export-system_app_list-user_id = 用户ID
export-system_app_list-change_user_id = 修改用户ID
export-system_app_list-change_time = 修改时间

# system_sub_app_list
export-system_sub_app_list-id = ID
export-system_sub_app_list-name = 名称
export-system_sub_app_list-client_id = 客户端ID
export-system_sub_app_list-status = 状态
export-system_sub_app_list-user_id = 用户ID
export-system_sub_app_list-change_user_id = 修改用户ID
export-system_sub_app_list-change_time = 修改时间

# system_request_list
export-system_request_list-id = ID
export-system_request_list-app_id = 应用ID
export-system_request_list-parent_app_id = 父应用ID
export-system_request_list-status = 状态
export-system_request_list-request_type = 请求类型
export-system_request_list-request_user_id = 请求用户ID
export-system_request_list-request_time = 请求时间
export-system_request_list-confirm_user_id = 审核用户ID
export-system_request_list-confirm_time = 审核时间
export-system_request_list-confirm_note = 审核备注
