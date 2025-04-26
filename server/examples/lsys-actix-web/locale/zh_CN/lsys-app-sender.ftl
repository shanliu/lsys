

tpl-exits  = 模板ID {$tpl_id} 已被使用 [{$id}]
check-email-not-match  =  = 邮箱[{$mail}]格式错误
smtp-check-error   = Smpt服务器异常:{$msg}
mail-send-fail = 邮件发送失败:{$msg}
mail-send-wait-fail = 邮件发送中,获取发送结果失败:{$msg}
mail-config-add-error  = 邮件配置:字段{$name}校验失败:{$msg}
mail-config-add-max-num-error = 请提交每次最大邮件限制发送数
mail-send-check-miss-error =  接收邮箱为空
mail-send-check-max-send  = 超过每次最大邮件发送量:{$max}
mail-send-check-block  =  接收邮箱:{$to} 已被屏蔽[{$config_id}]
mail-send-check-block-domain  =  接收邮箱域名:{$domain} 已被屏蔽[{$config_id}]
mail-send-check-close  = 邮件发送功能已被关闭[{$config_id}]
mail-send-check-limit  = 接收邮箱[{$to_mail}]超过最大发送量:{$max_send}[{$config_id}]
mail-task-empty = 未配置邮件发送任务
mail-send-cancel-status-bad=当前状态不可被取消
mail-send-cancel-is-ing  = 邮件发送中,无法取消
mail-cancel-status-error = 邮件非待发送状态:{$status}
sms-send-cancel-status-bad=当前状态不可被取消
sms-send-cancel-is-ing  = 短信发送中,无法取消
sms-cancel-status-error  = 短信非待发送状态:{$status}
sms-config-add-error  = 短信配置:字段{$name}校验失败:{$msg}
sms-config-add-max-num-error = 请提交每次最大短信限制发送数
sms-send-fail = 短信发送失败:{$msg}
sms-send-wait-fail = 短信发送中,获取发送结果失败:{$msg}
sms-send-check-miss-error = 接收短信为空
sms-send-check-max-send  = 超过每次最大短信发送量:{$max}
sms-send-check-block  = 接收号码:{$mobile} 已被屏蔽[{$config_id}]
sms-send-check-close  =  短信发送功能已被关闭[{$config_id}]
sms-send-check-limit  = 接收号码[{$mobile}]超过最大发送量:{$max_send}[{$config_id}]
sms-task-empty = 未配置短信发送任务
sms-status-task-empty = 未配置短信状态检测任务
sms-hw-config-url-error=华为接口短信发送地址配置异常
sms-config-branch-error  = 批量发送数量不能超过:{$max}




# 状态
status-SenderType-Smser=  短信
status-SenderType-Mailer= 邮件




# 状态
status-SenderLogType-Init=   新增完成
status-SenderLogType-Send=   发送日志
status-SenderLogType-Cancel= 取消发送





status-SenderLogStatus-Succ= 成功
status-SenderLogStatus-Fail= 失败

status-SenderLogStatus-MessageCancel= 取消
status-SenderLogStatus-NotifySucc=    回调成功
status-SenderLogStatus-NotifyFail=    回调失败




status-SenderConfigStatus-Enable= 正常
status-SenderConfigStatus-Delete=删除





status-SenderTplBodyStatus-Enable= 正常
status-SenderTplBodyStatus-Delete=删除





status-SenderTplConfigStatus-Enable= 正常
status-SenderTplConfigStatus-Delete=删除



status-SenderSmsConfigType-Close=      关闭功能
status-SenderSmsConfigType-Limit=     频率限制
status-SenderSmsConfigType-MaxOfSend= 每次最大发送数量
status-SenderSmsConfigType-PassTpl=   指定模板不检测限制
status-SenderSmsConfigType-Block = 10,    指定号码屏蔽






status-SenderSmsBodyStatus-Init=   待发送
status-SenderSmsBodyStatus-Finish= 已发送





status-SenderSmsMessageStatus-Init=       待发送
status-SenderSmsMessageStatus-IsSend=     已发送
status-SenderSmsMessageStatus-IsReceived= 已接收
status-SenderSmsMessageStatus-SendFail=   发送失败
status-SenderSmsMessageStatus-IsCancel=   已取消




status-SenderSmsAliyunStatus-Enable=启用
status-SenderSmsAliyunStatus-Delete=删除




status-SenderMailConfigType-Close=        关闭功能
status-SenderMailConfigType-Limit=        频率限制
status-SenderMailConfigType-MaxOfSend=    每次最大发送数量
status-SenderMailConfigType-PassTpl=      指定模板不检测限制
status-SenderMailConfigType-Block =       指定邮箱屏蔽
status-SenderMailConfigType-BlockDomain = 指定邮箱屏蔽




status-SenderMailBodyStatus-Init=   待发送
status-SenderMailBodyStatus-Finish= 已发送




status-SenderMailMessageStatus-Init=       待发送
status-SenderMailMessageStatus-IsSend=     已发送
status-SenderMailMessageStatus-IsReceived= 已接收
status-SenderMailMessageStatus-SendFail=   发送失败
status-SenderMailMessageStatus-IsCancel=   已取消




status-SenderMailSmtpStatus-Enable=启用
status-SenderMailSmtpStatus-Delete=删除
