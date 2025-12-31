 
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
barcode-image-error = 条码图片异常:{$msg}   
lsys-rxing-error = 条码解析库异常:{$msg}
area-not-enable = 地址库未启用
barcode-bad-format-error = 条码格式异常{$msg}
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
barcode-add-status-error = 解析记录状态异常:{$msg}
bad-audit-access = 非法访问审计数据
barcode-parse-error = 解析失败,记录为:{$record}
bad-app-id = 非法操作非自身app  
access-token-data-token-code-exits =登陆时发现 token_code 已使用，请更换
show-barcode-bad-type =不支持数据类型,只支持text或base64
barcode-bad-auth-error = 非公开二维码配置
app-notify-only-parent = 回调通知只支持系统应用
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
res-op-admin-barcode-view = 用户查看条形码
res-op-admin-barcode-edit = 用户编辑条形码
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
const-APP_FEATURE_BARCODE = 条码服务
const-APP_FEATURE_MAIL = 邮件服务
const-APP_FEATURE_RBAC = 权限服务
const-APP_FEATURE_SMS = 短信服务

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


var-mqr = 微型 QR 码 
var-aztec=Aztec 码​
var-qrcode=二维矩
var-DXFilmEdge= 胶片边缘码
var-codabar= 条形码
var-datamatrix= Data Matrix 码




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
var-barcode-create-config = 二维码创建
var-barcode-parse-record = 二维码解析
var-message-view = 发送消息查看