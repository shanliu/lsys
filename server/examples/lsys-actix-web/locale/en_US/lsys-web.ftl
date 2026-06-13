mail-send-check = Email sending error: {$msg}
user-external-login-url = Failed to load external account login URL: {$msg}
user-external-call = Failed to request external account info: {$msg}
user-external-other-bind = Account {$name} is already linked to another user: {$name}
rbac-unkown-res    = Resource {$res} does not exist
address-miss-city = Please select a district/county-level address
address-bad-area = Submitted area code is invalid
mail-bind-other-user = Email is already linked to another account [{$other_user_id}]
db-not-found = No records: {$msg}
db-error = Database error: {$msg}
user-old-passwrod-bad = Submitted old password is incorrect
user-old-passwrod-empty = Please enter old password
mail-is-confirm = Email already verified
username-is-exists = Account already exists: {$id}
password-not-set = Login password not set
client-secret-not-match = Secret key mismatch
app-redirect-uri-not-match = Unsupported redirect URI [redirect_uri]
reg-mobile-registered = Phone number already registered
mobile-bind-other-user = Phone number is linked to another account: {$id}
mobile-is-bind = Duplicate phone number link
lsys-lib-area-error = Address library error: {$msg}
lsys-lib-area-db = Address data error: {$msg}
area-not-found = Address not found
area-store-error = Address storage error: {$msg}
area-tantivy-error = Address search error: {$msg}

area-not-enable = Address library not enabled
bad-session-data = Unsupported login data type
auth-need-captcha = Please enter verification code for CODE login
app-oauth-login-bad-scope = App lacks authorization: {$scope_data}
not-system-app-confirm = Not a system app
role-perm-bad-op = Invalid resource operation ID: {$op_id}
role-perm-bad-res = Invalid resource ID: {$res_id}
role-user-not-system-user = User {$user_name}[{$user_id}] is not a system user, belongs to app: {$app_id}
role-user-not-found = User ID {$user_id} does not exist
not-user-app-confirm = Not a user app
app-is-subapp = This app is a sub-app
bad-audit-access = Unauthorized access to audit data
bad-app-id = Illegal operation on non-own app  
# RBAC Permissions
res-admin-global-system = System backend permission
res-op-admin-main = View system backend
res-op-admin-view-app = View apps
res-op-admin-edit-app = Edit apps
res-op-admin-view-docs = View documentation config
res-op-admin-edit-docs = Edit documentation config
res-op-admin-view-rbac = View permission config
res-op-admin-edit-rbac = Edit permission config
res-op-admin-app-sms-config = SMS app configuration
res-op-admin-app-sms-mgr = SMS app management
res-op-admin-app-mail-config = Email app configuration
res-op-admin-app-mail-mgr = Email app management
res-op-admin-site-setting = Site configuration
res-op-admin-manage-user = User management
res-op-admin-see-change-log = View change logs
res-admin-global-public = System public permission
res-op-admin-register = User registration
res-op-admin-login = User login
res-admin-global-app = System app permission
res-op-rest = App API access
res-user-global-user = User global permission
res-op-user-address-base = View user shipping addresses
res-op-user-address-edit = Edit user shipping addresses
res-op-user-email-base = View user email
res-op-user-email-edit = Edit user email
res-op-user-info-edit = Edit user info
res-op-user-mobile-edit = Edit user phone number
res-op-user-view-app = View apps
res-op-user-edit-app = Edit apps
res-op-user-view-notify = View callback notifications
res-op-user-rbac-check = User permission check
res-op-user-rbac-edit = Edit user permissions
res-op-user-app-mail-config = Configure user email apps
res-op-user-app-mail-veiw = View user email apps
res-op-user-app-mail-manage = Manage user email apps
res-op-user-app-mail-send = Send user email app messages
res-op-user-app-sms-config = Configure user SMS apps
res-op-user-app-sms-view = View user SMS apps
res-op-user-app-sms-manage = Manage user SMS apps
res-op-user-app-sms-send = Send user SMS app messages
# Validation Names
valid-rule-name-area_code = Area code
# Dictionaries 
const-SMS_NOTIFY_METHOD = SMS send result callback
const-SUB_APP_SECRET_NOTIFY_METHOD = Sub-app secret key change callback
# File Storage Type=
const-APP_FEATURE_MAIL = Mail
const-APP_FEATURE_RBAC = RBAC
const-APP_FEATURE_SMS = SMS
const-APP_FEATURE_FILE = File
# CSV Export Column Headers

# user_file_list
export-user_file_list-id = ID
export-user_file_list-file_name = File Name
export-user_file_list-file_md5 = File MD5
export-user_file_list-file_size = File Size
export-user_file_list-storage_type = Storage Type
export-user_file_list-status = Status
export-user_file_list-content_type = Content Type
export-user_file_list-add_time = Created At

# user_login_history
export-user_login_history-id = ID
export-user_login_history-login_type = Login Type
export-user_login_history-login_account = Login Account
export-user_login_history-login_ip = Login IP
export-user_login_history-login_city = Login City
export-user_login_history-account_id = Account ID
export-user_login_history-is_login = Is Login
export-user_login_history-login_msg = Login Message
export-user_login_history-add_time = Created At

# user_mailer_message_list
export-user_mailer_message_list-id = ID
export-user_mailer_message_list-snid = Send Batch
export-user_mailer_message_list-to_mail = To Mail
export-user_mailer_message_list-try_num = Try Count
export-user_mailer_message_list-status = Status
export-user_mailer_message_list-add_time = Created At
export-user_mailer_message_list-send_time = Send Time
export-user_mailer_message_list-setting_id = Setting ID

# user_smser_message_list
export-user_smser_message_list-id = ID
export-user_smser_message_list-snid = Send Batch
export-user_smser_message_list-area = Area Code
export-user_smser_message_list-mobile = Mobile
export-user_smser_message_list-try_num = Try Count
export-user_smser_message_list-status = Status
export-user_smser_message_list-add_time = Created At
export-user_smser_message_list-send_time = Send Time
export-user_smser_message_list-setting_id = Setting ID

# app_notify_list
export-app_notify_list-id = ID
export-app_notify_list-app_id = App ID
export-app_notify_list-notify_type = Notify Type
export-app_notify_list-notify_method = Notify Method
export-app_notify_list-notify_key = Notify Key
export-app_notify_list-status = Status
export-app_notify_list-try_num = Try Count
export-app_notify_list-try_max = Max Retries
export-app_notify_list-publish_time = Publish Time
export-app_notify_list-next_time = Next Time

# app_script_records
export-app_script_records-id = ID
export-app_script_records-request_id = Request ID
export-app_script_records-script_id = Script ID
export-app_script_records-add_user_id = Created By
export-app_script_records-app_id = App ID
export-app_script_records-status = Status
export-app_script_records-elapsed_ms = Elapsed (ms)
export-app_script_records-error_message = Error Message
export-app_script_records-add_time = Created At
export-app_script_records-start_time = Start Time
export-app_script_records-finish_time = Finish Time

# app_file_list
export-app_file_list-id = ID
export-app_file_list-file_name = File Name
export-app_file_list-file_md5 = File MD5
export-app_file_list-file_size = File Size
export-app_file_list-storage_type = Storage Type
export-app_file_list-status = Status
export-app_file_list-content_type = Content Type
export-app_file_list-add_time = Created At

# app_role_data
export-app_role_data-role_id = Role ID
export-app_role_data-role_key = Role Key
export-app_role_data-role_name = Role Name
export-app_role_data-user_range = User Range
export-app_role_data-res_range = Resource Range
export-app_role_data-user_id = User ID
export-app_role_data-user_timeout = User Timeout
export-app_role_data-op_key = Operation Key
export-app_role_data-op_name = Operation Name
export-app_role_data-res_type = Resource Type
export-app_role_data-res_data = Resource Data
export-app_role_data-res_name = Resource Name

# app_res_data
export-app_res_data-id = ID
export-app_res_data-user_id = User ID
export-app_res_data-res_type = Resource Type
export-app_res_data-res_data = Resource Data
export-app_res_data-res_name = Resource Name
export-app_res_data-change_time = Modified At
export-app_res_data-op_count = Operation Count
export-app_res_data-perm_count = Permission Count

# system_admin_file_list
export-system_admin_file_list-id = ID
export-system_admin_file_list-file_name = File Name
export-system_admin_file_list-file_md5 = File MD5
export-system_admin_file_list-file_size = File Size
export-system_admin_file_list-storage_type = Storage Type
export-system_admin_file_list-status = Status
export-system_admin_file_list-content_type = Content Type
export-system_admin_file_list-user_id = User ID
export-system_admin_file_list-add_time = Created At

# system_user_change_log
export-system_user_change_log-id = ID
export-system_user_change_log-log_type = Log Type
export-system_user_change_log-add_user_id = Created By
export-system_user_change_log-log_data = Log Data
export-system_user_change_log-add_time = Created At

# system_user_access
export-system_user_access-id = ID
export-system_user_access-app_id = App ID
export-system_user_access-oauth_app_id = OAuth App ID
export-system_user_access-user_id = User ID
export-system_user_access-token_data = Token Data
export-system_user_access-login_type = Login Type
export-system_user_access-login_ip = Login IP
export-system_user_access-status = Status
export-system_user_access-add_time = Created At
export-system_user_access-expire_time = Expire Time
export-system_user_access-logout_time = Logout Time

# system_mailer_message_list
export-system_mailer_message_list-id = ID
export-system_mailer_message_list-snid = Send Batch
export-system_mailer_message_list-to_mail = To Mail
export-system_mailer_message_list-try_num = Try Count
export-system_mailer_message_list-status = Status
export-system_mailer_message_list-add_time = Created At
export-system_mailer_message_list-send_time = Send Time
export-system_mailer_message_list-setting_id = Setting ID

# system_smser_message_list
export-system_smser_message_list-id = ID
export-system_smser_message_list-snid = Send Batch
export-system_smser_message_list-area = Area Code
export-system_smser_message_list-mobile = Mobile
export-system_smser_message_list-try_num = Try Count
export-system_smser_message_list-status = Status
export-system_smser_message_list-add_time = Created At
export-system_smser_message_list-send_time = Send Time
export-system_smser_message_list-setting_id = Setting ID

# system_app_list
export-system_app_list-id = ID
export-system_app_list-name = Name
export-system_app_list-client_id = Client ID
export-system_app_list-status = Status
export-system_app_list-user_id = User ID
export-system_app_list-change_user_id = Modified By
export-system_app_list-change_time = Modified At

# system_sub_app_list
export-system_sub_app_list-id = ID
export-system_sub_app_list-name = Name
export-system_sub_app_list-client_id = Client ID
export-system_sub_app_list-status = Status
export-system_sub_app_list-user_id = User ID
export-system_sub_app_list-change_user_id = Modified By
export-system_sub_app_list-change_time = Modified At

# system_request_list
export-system_request_list-id = ID
export-system_request_list-app_id = App ID
export-system_request_list-parent_app_id = Parent App ID
export-system_request_list-status = Status
export-system_request_list-request_type = Request Type
export-system_request_list-request_user_id = Request User ID
export-system_request_list-request_time = Request Time
export-system_request_list-confirm_user_id = Confirmed By
export-system_request_list-confirm_time = Confirmed At
export-system_request_list-confirm_note = Confirm Note
