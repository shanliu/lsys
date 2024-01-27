tpl-exits = template ID {$tpl_id} already used [{$id}]
check-email-error = Mailbox regular exception:{$msg}
check-email-not-match = = Mailbox [{$mail}] formatting error
smtp-check-error = Smpt server exception:{$msg}
sms-config-url-error = Gateway address error, only support http or https.
sms-config-branch-error = Batch send quantity cannot be exceeded:{$max}
mail-cancel-status-error = Message is not pending: {$status}
mail-config-add-error = mail-config: validation failed for field {$name}:{$msg}
mail-config-add-max-num-error = Please submit a maximum number of messages that can be sent at a time.
mail-send-check-miss-error = Receive mailbox is empty.
mail-send-check-max-send = Maximum number of messages sent per session exceeded:{$max}
mail-send-check-block = Receive mailbox:{$to} has been blocked [{$config_id}].
mail-send-check-block-domain = Receive mailbox domain:{$domain} Blocked [{$config_id}].
mail-send-check-close = Mail sending has been disabled [{$config_id}].
mail-send-check-limit = Incoming mailbox [{$to_mail}] exceeded the maximum send:{$max_send}[{$config_id}]
mail-task-empty = no mail send task configured
mail-send-ok-cancel = mail sent to mailbox:{$to_mail}[{$msg_id}]
sms-cancel-status-error = Message is not pending: {$status}
sms-config-add-error = sms-config-add-error = sms-config:Failed to validate field {$name}:{$msg}
sms-config-add-max-num-error = Please submit the maximum number of messages that can be sent at one time.
sms-send-check-miss-error = Received SMS is empty.
sms-send-check-max-send = Maximum number of SMS sent per session exceeded:{$max}
sms-send-check-block = Received number:{$mobile} has been blocked [{$config_id}].
sms-send-check-close = SMS sending has been disabled [{$config_id}].
sms-send-check-limit = Receive number [{$mobile}] exceeded max send:{$max_send}[{$config_id}
sms-task-empty = no sms send task configured
sms-send-ok-cancel = SMS sent:{$mobile}[{$msg_id}]
sms-status-task-empty = No SMS status checking task configured