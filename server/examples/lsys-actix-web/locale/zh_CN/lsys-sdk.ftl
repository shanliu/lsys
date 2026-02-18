# lsys-sdk 错误消息

service-invalid-url = 无效的服务URL: { $url }
service-http-error = HTTP请求错误: { $error }
service-http-rejected = 服务拒绝请求 [{ $method } { $url }] 状态码: { $status }
service-api-error = 服务API错误 [{ $method } { $url }] code={ $code } state={ $state } message={ $message }
service-parse-error = 响应解析错误 [{ $method } { $url }]: { $message }
