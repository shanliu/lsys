

tpl-exits  = 模板ID {$tpl_id} 已被使用 ({$id})
smtp-check-error   = Smpt服务器异常:{$msg}
mail-send-fail = 邮件发送失败:{$msg}
mail-send-wait-fail = 邮件发送中,获取发送结果失败:{$msg}
mail-config-add-error  = 邮件配置:字段{$name}校验失败:{$msg}
mail-config-add-max-num-error = 请提交每次最大邮件限制发送数
mail-send-check-miss-error = 接收邮箱为空
mail-send-check-max-send  = 超过每次最大邮件发送量:{$max}
mail-send-check-block  = 接收邮箱:{$to} 已被屏蔽({$config_id})
mail-send-check-block-domain  = 接收邮箱域名:{$domain} 已被屏蔽({$config_id})
mail-send-check-close  = 邮件发送功能已被关闭({$config_id})
mail-send-check-limit  = 接收邮箱({$to_mail})超过最大发送量:{$max_send}({$config_id})
mail-task-empty = 未配置邮件发送任务
mail-send-cancel-status-bad = 当前状态不可被取消
mail-send-cancel-is-ing  = 邮件发送中,无法取消
mail-cancel-status-error = 邮件非待发送状态:{$status}
sms-send-cancel-status-bad = 当前状态不可被取消
sms-send-cancel-is-ing  = 短信发送中,无法取消
sms-cancel-status-error  = 短信非待发送状态:{$status}
sms-config-add-error  = 短信配置:字段{$name}校验失败:{$msg}
sms-config-add-max-num-error = 请提交每次最大短信限制发送数
sms-send-fail = 短信发送失败:{$msg}
sms-send-wait-fail = 短信发送中,获取发送结果失败:{$msg}
sms-send-check-miss-error = 接收短信为空
sms-send-check-max-send  = 超过每次最大短信发送量:{$max}
sms-send-check-block  = 接收号码:{$mobile} 已被屏蔽({$config_id})
sms-send-check-close  = 短信发送功能已被关闭({$config_id})
sms-send-check-limit  = 接收号码({$mobile})超过最大发送量:{$max_send}({$config_id})
sms-task-empty = 未配置短信发送任务
sms-status-task-empty = 未配置短信状态检测任务
tpl-name-exits = 同应用下已存在名称为{$name}的应用,ID为:{$id}
sender-setting-used = 当前配置已被使用中，引用次数:{$total}
# 状态
status-SenderType-Smser = 短信
status-SenderType-Mailer = 邮件




# 状态
status-SenderLogType-Init = 新增完成
status-SenderLogType-Send = 发送日志
status-SenderLogType-Cancel = 取消发送





status-SenderLogStatus-Succ = 成功
status-SenderLogStatus-Fail = 失败

status-SenderLogStatus-MessageCancel = 取消
status-SenderLogStatus-NotifySucc = 回调成功
status-SenderLogStatus-NotifyFail = 回调失败




status-SenderConfigStatus-Enable = 正常
status-SenderConfigStatus-Delete = 删除





status-SenderTplBodyStatus-Enable = 正常
status-SenderTplBodyStatus-Delete = 删除





status-SenderTplConfigStatus-Enable = 正常
status-SenderTplConfigStatus-Delete = 删除



status-SenderSmsConfigType-Close = 关闭功能
status-SenderSmsConfigType-Limit = 频率限制
status-SenderSmsConfigType-MaxOfSend = 每次最大发送数量
status-SenderSmsConfigType-PassTpl = 指定模板不检测限制
status-SenderSmsConfigType-Block = 指定号码屏蔽






status-SenderSmsBodyStatus-Init = 发送中
status-SenderSmsBodyStatus-Finish = 处理完成





status-SenderSmsMessageStatus-Init = 待发送
status-SenderSmsMessageStatus-IsSend = 已发送
status-SenderSmsMessageStatus-IsReceived = 已接收
status-SenderSmsMessageStatus-SendFail = 发送失败
status-SenderSmsMessageStatus-IsCancel = 已取消




status-SenderSmsAliyunStatus-Enable = 启用
status-SenderSmsAliyunStatus-Delete = 删除




status-SenderMailConfigType-Close = 关闭功能
status-SenderMailConfigType-Limit = 频率限制
status-SenderMailConfigType-MaxOfSend = 每次最大发送数量
status-SenderMailConfigType-PassTpl = 指定模板不检测限制
status-SenderMailConfigType-Block = 指定邮箱屏蔽
status-SenderMailConfigType-BlockDomain = 指定邮箱屏蔽




status-SenderMailBodyStatus-Init = 发送中
status-SenderMailBodyStatus-Finish = 处理完成




status-SenderMailMessageStatus-Init = 待发送
status-SenderMailMessageStatus-IsSend = 已发送
status-SenderMailMessageStatus-IsReceived = 已接收
status-SenderMailMessageStatus-SendFail = 发送失败
status-SenderMailMessageStatus-IsCancel = 已取消




status-SenderMailSmtpStatus-Enable = 启用
status-SenderMailSmtpStatus-Delete = 删除

#校验名称

valid-rule-name-app_name = 应用名
valid-rule-name-tpl_data = 模板数据
valid-rule-name-from_mail = 来源邮箱
valid-rule-name-subject_tpl_id = 消息主题内容模板ID
valid-rule-name-body_tpl_id = 消息体内容模板ID
valid-rule-name-reply_email = 回复邮箱
valid-rule-name-access_secret = 应用秘钥
valid-rule-name-branch_limit = 批量限制次数
valid-rule-name-access_id = 应用ID
valid-rule-name-region = 区域
valid-rule-name-sms_setting_id = 短信端口配置ID
valid-rule-name-tpl_id = 内容模板ID
valid-rule-name-tpl_key = 发送模板标识
valid-rule-name-aliyun_sms_tpl = 阿里云短信模版
valid-rule-name-aliyun_sign_name = 阿里云短信签名
valid-rule-name-account_token = 应用秘钥
valid-rule-name-sms_app_id = 短信应用ID
valid-rule-name-callback_key = 回调秘钥
valid-rule-name-account_sid = 应用ID
valid-rule-name-config_id = 配置ID
valid-rule-name-hw_url = 华为短信网关
valid-rule-name-hw_app_key = 华为短信应用KEY
valid-rule-name-hw_app_secret = 华为短信应用秘钥
valid-rule-name-hw_signature = 华为短信签名 
valid-rule-name-hw_sender = 华为短信发送方
valid-rule-name-access_key = 应用KEY
valid-rule-name-app_id = 应用ID
valid-rule-name-sign_id = 签名ID
valid-rule-name-template_id = 模板ID
valid-rule-name-template_map = 模板关系
valid-rule-name-secret_id = 应用ID
valid-rule-name-secret_key = 应用秘钥
valid-rule-name-config_name = 配置名
valid-rule-name-sign_name = 签名
valid-rule-name-mail = 邮箱
valid-rule-name-reply_mail = 回复邮箱
valid-rule-name-tpl_var = 模板变量
valid-rule-name-max_try_num = 重试次数
valid-rule-name-mobile = 手机号
valid-rule-name-smtp_port = SMTP端口号
valid-rule-name-smtp_branch_limit = 邮件收件人最大数量
valid-rule-name-smtp_email = 发件人邮箱
valid-rule-name-smtp_host = SMTP服务器主机
valid-rule-name-tpl_name=配置名