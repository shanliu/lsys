app-client-id-exits = 应用ID[{$client_id}]已被其他应用 {$other_name} 使用
app-client-id-req = 应用ID[{$client_id}]已被占用
papp-id-bad = 应用{$name}不可添加子应用
auth-alpha-domain-error = 域名规则解析错误:{$msg}
app-secret-wrong = 秘钥格式错误
app-name-wrong = 应用名应在[{$min}-{$max}]之间,当前为:{$len}
app-client-id-wrong = 应用ID应在[{$min}-{$max}]之间,当前为:{$len}
auth-alpha-check-error = 应用ID仅能是字母或数字
app-find-bad-status = 应用ID[{$client_id}]状态异常
app-req-is-confirm = 该应用请求已处理
app-req-is-invalid = 改应用请求无效
app-req-status-invalid = 应用确认状态无效
app-req-bad = 应用未请求或请求授权异常
app-oauth-server-skey-wrong = KEY格式错误,仅支持英文数字下划线
app-oauth-server-name-wrong = 显示名称不能为空
app-is-not-system-app=应用{$name}:{$client_id}非系统应用
app-oauth-login-bad-scope=应用不存在授权:{$scope_data}
app-oauth-server-use-scope = SCOPE已在以下应用中被使用:{$scope_data}
app-bad-id-error = 账号对应的应用ID{$id}异常，错误详细：{$msg}
app-bad-error=应用 {$client_id} 查询异常:{$msg}
app-bad-scope = 应用不存在以下功能:{$scope}
app-not-found= 应用不存在 {$name}
app-bad-status = 应用非正常启用状态
app-feature-not-support = 应用{$name}的以下功能未开通:{$feature}
app-config-bad= 应用 {$name}配置异常
app-session-get-error= 获取应用{$client_id}的授权信息失败:{$msg}
app-session-refresh-error= 重置应用${client_id}的授权信息失败:{$msg}
app-session-clear-error= 清除应用的登录信息失败:{$msg}
papp-not-match-parent = 当前选择的父应用{$name}于账号应用不匹配
papp-bad-parent= 请选择父应用

bad-timeout-param = 时间参数异常,不能设置为:{$time}

notify-call-not-support=回调地址仅支持HTTP或HTTPS
notify-reqwest-build-error=构建回调通知请求异常:{$msg}
notify-reqwest-check-error=回调通知[{$url}]时异常:{$msg}

# 状态
status-NotifyDataStatus-Init = 未回调
status-NotifyDataStatus-Succ = 回调完成
status-NotifyDataStatus-Fail = 回调失败


status-AppStatus-Init=    正常
status-AppStatus-Enable=  正常
status-AppStatus-Disable= 被禁用
status-AppStatus-Delete= 删除




status-AppFeatureStatus-Enable=  正常
status-AppFeatureStatus-Disable= 被禁用
status-AppFeatureStatus-Delete= 删除


status-AppOAuthServerScopeStatus-Enable=  正常
status-AppOAuthServerScopeStatus-Delete= 删除


status-AppRequestType-AppReq=           新应用申请
status-AppRequestType-AppChange=        应该更改申请
status-AppRequestType-SubApp=           子应用可用申请
status-AppRequestType-ExterLogin=       外部账号登录系统申请
status-AppRequestType-OAuthServer=      Oauth服务申请
status-AppRequestType-OAuthClient=      Oauth登录申请
status-AppRequestType-OAuthClientScope= OAUTH登录新增权限申请
status-AppRequestType-ExterFeatuer=     外部功能申请:如发短信邮件等





status-AppRequestStatus-Pending=  待审
status-AppRequestStatus-Approved= 批准
status-AppRequestStatus-Rejected= 驳回
status-AppRequestStatus-Invalid=  作废
status-AppRequestStatus-Delete=  删除





status-AppSecretType-App=    应用
status-AppSecretType-OAuth=  oauth
status-AppSecretType-Notify= 回调





status-AppSecretStatus-Enable=  正常
status-AppSecretStatus-Delete= 删除




valid-rule-name-name=应用名
valid-rule-name-client-id=应用标识
valid-rule-name-secret=应用秘钥
valid-rule-name-callback-domain=登录跳转域名