tpl-exits  = 模板ID {$tpl_id} 已被使用 [{$id}]
check-email-error = 邮箱正则异常:{$msg}
check-email-not-match  =  = 邮箱[{$mail}]格式错误
smtp-check-error   = Smpt服务器异常:{$msg}
sms-config-url-error  = 网关地址错误,仅支持http或https.
sms-config-branch-error  = 批量发送数量不能超过:{$max}

mail-send-fail = 邮件发送失败:{$msg}
mail-send-wait-fail = 邮件发送中,获取发送结果失败:{$msg}
mail-cancel-status-error = 邮件非待发送状态:{$status}
mail-config-add-error  = 邮件配置:字段{$name}校验失败:{$msg}
mail-config-add-max-num-error = 请提交每次最大邮件限制发送数
mail-send-check-miss-error =  接收邮箱为空
mail-send-check-max-send  = 超过每次最大邮件发送量:{$max}
mail-send-check-block  =  接收邮箱:{$to} 已被屏蔽[{$config_id}]
mail-send-check-block-domain  =  接收邮箱域名:{$domain} 已被屏蔽[{$config_id}]
mail-send-check-close  = 邮件发送功能已被关闭[{$config_id}]
mail-send-check-limit  = 接收邮箱[{$to_mail}]超过最大发送量:{$max_send}[{$config_id}]
mail-task-empty = 未配置邮件发送任务
mail-send-ok-cancel  = 邮件已发送到邮箱:{$to_mail}[{$msg_id}]
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
sms-send-ok-cancel  = 短信已发送:{$mobile}[{$msg_id}]
sms-status-task-empty = 未配置短信状态检测任务