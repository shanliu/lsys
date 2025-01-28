time-error = Get system time exception:{$msg}

user-address-is-delete = Address duplicate deletion [{$id}].

user-passwrod-wrong = Password must be [{$min}-{$max}] characters, current:{$len}

auth-not-login = User is not logged in or TOKEN has expired.

auth-not-user = User {$name} does not exist.

auth-bad-password = User password is wrong.

auth-not-set-password = User has not set password for login.


user-status-invalid = user-status-invalid [{$user}:{$status}]


user-auth-parse-bad = user TOKEN exception

user-auth-parse-error = Parsing TOKEN exception:{$msg}

check-user-lock = User {$user} is locked and will be restored in {$time} seconds.

auth-user-captcha = Authentication error submitted when user {$user} logged in.

auth-user-disable = User {$user} has been disabled.


user-address-not-empty = Address field: {$name} cannot be null

user-email-exits-other-account = Mailbox [{$email}] is bound to another account [{$id}].
user-external-other-bind = external-exits-other-account [{$name}] is already bound to other account [{$id}]

user-mobile-exits = Mobile phone number [{$mobile}] is already bound to another account [{$id}].

user-username-error = Login account must be [{$min}-{$max}] characters and cannot start with [{$bad_start}].
user-name-exits = Login account [{$name}] is already in use.

user-old-passwrod = old password cannot be used
user-passwrod-delete = Password record lost
user-bad-status = The status of the added user must be current.
user-is-delete = user {$user} cannot be enabled, status is invalid
user-nikename-wrong = Nickname must be non-empty and within {$max} characters
auth-email-error = Email regular exception: {$msg}
auth-email-not-match = Mailbox [{$mail}] format error
auth-mobile-error = Failed to validate cell phone number:{$msg}
auth-mobile-area-error = Cell phone area code {$area} exception
serde-error = Serialization exception:{$msg}
utf-error = String is not a valid UTF character, error details:{$msg}
not-login-empty-token = user not logged in: token does not exist
