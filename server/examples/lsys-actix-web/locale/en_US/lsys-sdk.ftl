# lsys-sdk error messages

service-invalid-url = Invalid service URL: { $url }
service-http-error = HTTP request error: { $error }
service-http-rejected = Service rejected request [{ $method } { $url }] status: { $status }
service-api-error = Service API error [{ $method } { $url }] code={ $code } state={ $state } message={ $message }
service-parse-error = Response parse error [{ $method } { $url }]: { $message }
