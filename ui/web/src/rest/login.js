import isEmail from "validator/lib/isEmail";
import { fialResult, globalRest, restResult, sessionRest } from "../utils/rest";

function LoggedRest() {
    return sessionRest('/api/user')
};
function LoginRest() {
    return globalRest('/api/user')
};



export async function nameLogin(login_type, param, config) {
    const { name, password, captcha_key, captcha_code } = param;
    var param;
    if (login_type == 'email') {
        param = {
            email: name,
            password: password,

        };
    } else if (login_type == 'sms') {
        param = {
            mobile: name,
            area_code: "86",
            password: password,

        };
    } else {
        login_type = "name";
        param = {
            name: name,
            password: password,
        };
    }
    var errors = {};
    if (typeof password !== "string" || password.length < 6) {
        errors.password = "密码不能小于6位";
    }
    if (typeof name !== "string" || name.length < 2) {
        errors.name = "账号必须大于一个字符";
    }
    if (typeof captcha_key == "string" && captcha_key.length > 0) {
        if (typeof captcha_code !== "string" || captcha_code.length < 1) {
            errors.captcha = "请输入图片验证码";
        } else {
            param.captcha = {
                code: captcha_code,
                key: captcha_key,
            };
        }
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await LoginRest().post("login/" + login_type, param, config);
    return restResult(response)
}

export async function matchNameLogin(param, config) {
    const { name } = param
    var login_type;
    if (isEmail(name)) {
        login_type = "email";
    } else if (/^1[0-9]{10}$/.test(name)) {
        login_type = "sms";
    } else {
        login_type = "name";
    }
    return await nameLogin(login_type, param, config);
}


export async function emailLoginSendCode(param, config) {
    const { email, captcha_code, captcha_key } = param;
    let errors = {};
    if (!email || !isEmail(email)) {
        errors.name = "邮箱格式错误";
    }
    if (typeof captcha_code !== "string" || captcha_code.length < 1) {
        errors.captcha = "验证码不能为空";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    var param = {
        email: email,
        captcha: {
            code: captcha_code,
            key: captcha_key
        }
    };
    let response = await LoginRest().post("/login/email-send-code", param, config);
    return restResult(response)
}


export async function emailCodeLogin(param, config) {
    const { code, email, captcha_code, captcha_key } = param;
    let errors = {};
    if (typeof code !== "string" || code.code < 1) {
        errors.code = "请输入验证码";
    }
    if (!email || !isEmail(email)) {
        errors.name = "邮箱格式错误";
    }
    var param = {
        email: email,
        code: code,
    };
    if (typeof captcha_key == "string" && captcha_key.length > 0) {
        if (typeof captcha_code !== "string" || captcha_code.length < 1) {
            errors.captcha = "请输入图片验证码";
        } else {
            param.captcha = {
                code: captcha_code,
                key: captcha_key,
            };
        }
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await LoginRest().post("/login/email-code", param, config);
    return restResult(response)
}

export async function emailLogin(param, config) {
    const { code, email, captcha_code, captcha_key } = param;
    let errors = {};
    if (typeof code !== "string" || code.code < 1) {
        errors.code = "请输入验证码";
    }
    if (!email || !isEmail(email)) {
        errors.name = "邮箱格式错误";
    }
    var param = {
        email: email,
        code: code,
    };
    if (typeof captcha_key == "string" && captcha_key.length > 0) {
        if (typeof captcha_code !== "string" || captcha_code.length < 1) {
            errors.captcha = "请输入图片验证码";
        } else {
            param.captcha = {
                code: captcha_code,
                key: captcha_key,
            };
        }
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await LoginRest().post("/login/email-code", param, config);
    return restResult(response)
}


export async function mobileCodeLogin(param, config) {
    const { code, mobile, captcha_code, captcha_key } = param;
    let errors = {};
    if (typeof code !== "string" || code.code < 1) {
        errors.code = "请输入验证码";
    }
    if (!mobile || !/^1[0-9]{10}$/.test(mobile)) {
        errors.name = "手机号格式错误";
    }
    var param = {
        mobile: mobile,
        area_code: "86",
        code: code,
    };
    if (typeof captcha_key == "string" && captcha_key.length > 0) {
        if (typeof captcha_code !== "string" || captcha_code.length < 1) {
            errors.captcha = "请输入图片验证码";
        } else {
            param.captcha = {
                code: captcha_code,
                key: captcha_key,
            };
        }
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await LoginRest().post("/login/sms-code", param, config);
    return restResult(response)
}


export async function mobileLoginSendCode(param, config) {
    var errors = {};
    const { mobile, captcha_key, captcha_code } = param;
    if (!mobile || !/^1[0-9]{10}$/.test(mobile)) {
        errors.name = "手机号格式错误";
    }
    if (typeof captcha_code !== "string" || captcha_code.length < 1) {
        errors.captcha = "验证码不能为空";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    var param = {
        mobile: mobile,
        area_code: "86",
        captcha: {
            code: captcha_code,
            key: captcha_key
        }
    };
    let response = await LoginRest().post("/login/sms-send-code", param, config);
    return restResult(response)
}




export async function emailSignupSendCode(param, config) {
    const { email, captcha_code, captcha_key } = param;
    var errors = {};
    if (!email || !isEmail(email)) {
        errors.name = "邮箱格式错误";
    }
    if (typeof captcha_code !== "string" || captcha_code.length < 1) {
        errors.captcha = "验证码不能为空";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    var param = {
        email: email,
        captcha: {
            code: captcha_code,
            key: captcha_key
        }
    };
    let response = await LoginRest().post("/signup/email-code", param, config);
    return restResult(response)
}

export async function emailSignup(param, config) {
    const { email, nikename, password, code } = param;
    let errors = {};
    if (typeof code !== "string" || code.code < 1) {
        errors.code = "请输入验证码";
    }
    if (typeof password !== "string" || password.length < 6) {
        errors.password = "密码不小于6个字符";
    }
    if (typeof nikename !== "string" || nikename.length < 2) {
        errors.nikename = "昵称不小于2个字符";
    }
    if (!email || !isEmail(email)) {
        errors.name = "邮箱格式错误";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    var param = {
        email: email,
        code: code,
        password: password,
        nikename: nikename
    };
    let response = await LoginRest().post("/signup/email", param, config);
    return restResult(response)
}


export async function mobileSignupSendCode(param, config) {
    const { mobile, captcha_code, captcha_key } = param;
    let errors = {};
    if (!mobile || !/^1[0-9]{10}$/.test(mobile)) {
        errors.name = "手机号格式错误";
    }
    if (typeof captcha_code !== "string" || captcha_code.length < 1) {
        errors.captcha = "验证码不能为空";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    var param = {
        mobile: mobile,
        area_code: "86",
        captcha: {
            code: captcha_code,
            key: captcha_key
        }
    };
    let response = await LoginRest().post("/signup/sms-code", param, config);
    return restResult(response)
}

export async function mobileSignup(param, config) {
    const { mobile, nikename, password, code } = param;
    let errors = {};
    if (typeof code !== "string" || code.code < 1) {
        errors.code = "请输入验证码";
    }

    if (typeof password !== "string" || password.length < 6) {
        errors.password = "密码不小于6个字符";
    }

    if (typeof nikename !== "string" || nikename.length < 2) {
        errors.nikename = "昵称不小于2个字符";
    }

    if (!mobile || !/^1[0-9]{10}$/.test(mobile)) {
        errors.name = "手机号格式错误";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }

    var param = {
        mobile: mobile,
        area_code: "86",
        code: code,
        password: password,
        nikename: nikename
    };
    let response = await LoginRest().post("/signup/sms", param, config);
    return restResult(response)
}

export async function emailFindPasswordSendCode(param, config) {
    const { captcha_code, captcha_key, email } = param;
    var errors = {};
    if (!email || !isEmail(email)) {
        errors.name =
            "邮箱格式错误";
    }
    if (typeof captcha_code !== "string" || captcha_code.length < 1) {
        errors.captcha = "验证码不能为空";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    var param = {
        email: email,
        captcha: {
            code: captcha_code,
            key: captcha_key
        }
    };
    let response = await LoginRest().post("/password_reset/email_code", param, config);
    return restResult(response)
}


export async function emailFindPassword(param, config) {
    const { email, password, code } = param;
    var errors = {};
    if (typeof code !== "string" || code.code < 1) {
        errors.code = "请输入验证码";
    }
    if (!email || !isEmail(email)) {
        errors.name = "邮箱格式错误";
    }
    if (typeof password !== "string" || password.length < 6) {
        errors.password = "密码不能小于6位";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    var param = {
        email: email,
        code: code,
        new_password: password
    };
    let response = await LoginRest().post("/password_reset/email", param, config);
    return restResult(response)
}



export async function mobileFindPasswordSendCode(param, config) {
    const { captcha_code, captcha_key, mobile } = param;

    var errors = {};

    if (!mobile || !/^1[0-9]{10}$/.test(mobile)) {
        errors.name = "手机号格式错误";
    }
    if (typeof captcha_code !== "string" || captcha_code.length < 1) {
        errors.captcha = "验证码不能为空";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    var param = {
        mobile: mobile,
        area_code: "86",
        captcha: {
            code: captcha_code,
            key: captcha_key
        }
    };

    let response = await LoginRest().post("/password_reset/mobile_code", param, config);
    return restResult(response)
}


export async function mobileFindPassword(param, config) {
    const { mobile, password, code } = param;
    var errors = {};
    if (typeof code !== "string" || code.code < 1) {
        errors.code = "请输入验证码";
    }
    if (!mobile || !/^1[0-9]{10}$/.test(mobile)) {
        errors.name = "手机号格式错误";
    }

    if (typeof password !== "string" || password.length < 6) {
        errors.password = "密码不能小于6位";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    var param = {
        mobile: mobile,
        area_code: "86",
        code: code,
        new_password: password
    };
    let response = await LoginRest().post("/password_reset/mobile", param, config);
    return restResult(response)
}

export async function loginData(param, config) {
    let response = await LoggedRest().post("/login_data", param, config);
    return restResult(response)
}


export async function logout(config) {
    let response = await LoggedRest().get("/logout", config);
    return restResult(response)
}





export async function QrcodeLogin(login_type, login_state, config) {
    let url;
    switch (login_type) {
        case 'wechat':
            url = window.location.protocol + "//" + window.location.host + window.location.pathname + "mobile/wechat-login.html";
            break
        default:
            return fialResult({}, `绑定类型[${login_type}]不支持`);
    }
    let response = await LoginRest().post("/external_login_url", {
        "login_type": login_type,
        "login_callback": url,
        "login_state": login_state
    }, config);
    return restResult(response)
}

export async function QrcodeLoginCheck(login_type, login_state, config) {
    let response = await LoginRest().post("/external_state_check", {
        "login_state": login_state,
        "login_type": login_type,
    }, config)
    return restResult(response)
}

export async function OauthGetScope(client_id, scope, config) {
    let response = await LoggedRest().post("/oauth/scope", {
        "client_id": client_id,
        "scope": scope,
    }, config)
    return restResult(response)
}
export async function OauthDo(client_id, scope, redirect_uri, config) {
    let response = await LoggedRest().post("/oauth/do", {
        "client_id": client_id,
        "scope": scope,
        "redirect_uri": redirect_uri
    }, config)
    return restResult(response)
}




