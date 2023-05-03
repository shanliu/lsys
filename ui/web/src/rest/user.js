import isEmail from "validator/lib/isEmail";
import { fialResult, globalRest, restResult, sessionRest } from "../utils/rest";

function userRest() {
    return sessionRest('/api/user')
};


export async function getPasswordModify(config) {
    let response = await userRest().post("/info/password_modify", {}, config);
    return restResult(response)
}


export async function loginHistroy(param, config) {
    const { login_type, login_account, is_login, page, page_size } = param;
    let response = await userRest().post("/login_history", {
        "login_type": login_type === '' ? null : login_type,
        "login_account": login_account,
        "is_login": is_login == '' ? null : parseInt(is_login),
        "page": {
            page: parseInt(page) >= 0 ? (parseInt(page) + 1) : 1,
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 10
        }
    }, config);
    return restResult(response, ['not_found'])
}


export const genderStatus = [
    { key: 0, val: '保密' },
    { key: 1, val: '男' },
    { key: 2, val: '女' },
];


export async function setInfo(param, config) {
    const { nikename, gender, birthday } = param;
    var errors = {};
    if (typeof nikename != 'string' || nikename.length <= 1) {
        errors.nikename = "登陆名需大于1个字符";
    }
    let sgender = parseInt(gender);
    if (isNaN(sgender) || !genderStatus.find((e) => { return e.key == sgender })) {
        errors.gender = "性别选错错误";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    var param = {
        nikename: nikename,
        gender: parseInt(sgender),
        birthday: birthday == '' ? null : birthday,
    };
    let response = await userRest().post("/info/set_info", param, config);
    return restResult(response)
}



export async function setUsername(param, config) {
    const { name } = param;
    var errors = {};
    if (typeof name != 'string' || name.length <= 1) {
        errors.name = "登陆名需大于1个字符";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    var param = {
        name: name,
    };
    let response = await userRest().post("/info/set_username", param, config);
    return restResult(response)
}


export async function setPassword(param, config) {
    const { old_password, new_password, skip_old } = param;

    var errors = {};

    if (skip_old) {
        if (typeof old_password == 'string' && old_password.length < 1) {
            errors.old_password =
                "请原密码未输入";
        }

        if (typeof new_password != 'string' || new_password.length < 6) {
            errors.new_password =
                "新密码必须大于等于6个字符";
        }
    } else {
        if (typeof new_password != 'string' || new_password.length < 6) {
            errors.new_password =
                "设置的登录密码必须大于等于6个字符";
        }
    }

    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    var param = {
        old_password: old_password,
        new_password: new_password
    };
    let response = await userRest().post("/password/set", param, config);
    return restResult(response)
}

//mobile page 

export const mobileComfirmStatus = [
    { key: 1, val: '未确认' },
    { key: 2, val: '已确认' },
];

export async function mobileList(config) {
    let statarr = mobileComfirmStatus.map((e) => { return e.key });
    let response = await userRest().post("/mobile/list_data", {
        "status": statarr,
    }, config);
    return restResult(response, ['not_found'])
}


export async function mobileAdd(param, config) {
    const { mobile } = param;
    var errors = {};
    if (!mobile || !/^1[0-9]{10}$/.test(mobile)) {
        errors.name = "手机格式错误";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    var param = {
        area_code: "86",
        mobile: mobile,
    };
    let response = await userRest().post("/mobile/add", param, config);
    return restResult(response)
}

export async function mobileSendCode(param, config) {
    const { mobile, captcha_code, captcha_key } = param;
    var errors = {};
    if (!mobile || !/^1[0-9]{10}$/.test(mobile)) {
        errors.name =
            "手机号格式错误";
    }
    if (typeof captcha_code !== "string" || captcha_code.length < 1) {
        errors.captcha = "验证码不能为空";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    var param = {
        area_code: "86",
        mobile: mobile,
        captcha: {
            code: captcha_code,
            key: captcha_key
        }
    };
    let response = await userRest().post("/mobile/send_code", param, config);
    return restResult(response)
}

export async function mobileConfirm(param, config) {
    const { id, code } = param;
    var errors = {};
    if (typeof code !== "string" || code.length < 1) {
        errors.captcha = "验证码不能为空";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    var param = {
        mobile_id: parseInt(id),
        code: code
    };
    let response = await userRest().post("/mobile/confirm", param, config);
    return restResult(response)
}

export async function mobileDelete(id, config) {
    id = parseInt(id)
    let response = await userRest().post("/mobile/delete", {
        "mobile_id": id,
    }, config);
    return restResult(response)
}



//email page


export const emailComfirmStatus = [
    { key: 1, val: '未确认' },
    { key: 2, val: '已确认' },
];

export async function emailList(config) {
    let statarr = emailComfirmStatus.map((e) => { return e.key });
    let response = await userRest().post("/email/list_data", {
        "status": statarr,
    }, config);
    return restResult(response, ['not_found'])
}

export async function emailAdd(param, config) {
    const { email } = param;
    var errors = {};
    if (!email || !isEmail(email)) {
        errors.name = "邮箱格式错误";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    var param = {
        email: email,
    };
    let response = await userRest().post("/email/add", param, config);
    return restResult(response)
}

export async function emailSendCode(param, config) {
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
    let response = await userRest().post("/email/send_code", param, config);
    return restResult(response)
}

export async function emailConfirm(param, config) {
    const { id, code } = param;
    var errors = {};
    if (typeof code !== "string" || code.length < 1) {
        errors.captcha = "验证码不能为空";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    var param = {
        email_id: parseInt(id),
        code: code
    };
    let response = await globalRest('/api/user').post("/email_confirm", param, config);
    return restResult(response)
}


export async function emailDelete(id, config) {
    id = parseInt(id)
    let response = await userRest().post("/email/delete", {
        "email_id": id,
    }, config);
    return restResult(response)
}


//oauth page


export const OauthType = [
    { key: "wechat", val: '微信' },
];
export const OauthConfig = [
    { key: "wechat", val: '微信', type: "qrcode" },
];

export async function oauthList(config) {
    let statarr = []
    let response = await userRest().post("/external/list_data", {
        "oauth_type": statarr,
    }, config)
    return restResult(response, ['not_found'])
}

export async function oauthDelete(id, config) {
    id = parseInt(id)
    let response = await userRest().post("/external/delete", {
        "ext_id": id,
    }, config);
    return restResult(response)
}

export async function oauthAdd(login_type, login_state, config) {
    let url;
    switch (login_type) {
        case 'wechat':
            url = window.location.protocol + "//" + window.location.host + window.location.pathname + "mobile/wechat-login.html";
            break
        default:
            return fialResult({}, `绑定类型[${login_type}]不支持`);
    }
    let response = await userRest().post("/external/bind_url", {
        "login_state": login_state,
        "login_type": login_type,
        "callback_url": url
    }, config)
    return restResult(response)
}

export async function oauthCheck(login_type, login_state, config) {
    let response = await userRest().post("/external/bind_check", {
        "login_state": login_state,
        "login_type": login_type,
    }, config)
    return restResult(response)
}


export const userStatus = [
    { key: 1, val: '待确认' },
    { key: 2, val: '已确认' },
];

export const searchType = [
    { key: 'mobile', val: '手机号' },
    { key: 'email', val: '邮箱' },
    { key: 'username', val: '登录名' },
    { key: 'nikename', val: '昵称' },
];


export async function userSearch(param, config) {
    const { enable_user, key_word, page_size, more, end_pos, start_pos, opt } = param;
    // pub name: Option<bool>,
    // pub info: Option<bool>,
    // pub address: Option<bool>,
    // pub external: Option<Vec<String>>,
    // pub email: Option<Vec<i8>>,
    // pub mobile: Option<Vec<i8>>,
    var param = {
        key_word: key_word,
        enable: !!enable_user,
        base: !!opt,
        name: !!opt,
        info: !!opt,
        address: !!opt,
        external: opt ? [] : null,
        email: opt ? [] : null,
        mobile: opt ? [] : null,
        limit: {
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 10,
            next: false,
            more: more,
        },
    };
    if (end_pos && end_pos != '0' && end_pos != '') {
        param.limit = {
            ...param.limit,
            pos: end_pos,
            eq_pos: false,
            next: true,
        }
    } else if (start_pos && start_pos != '0' && start_pos != '') {
        param.limit = {
            ...param.limit,
            pos: start_pos,
            eq_pos: true,
        }
    }


    let response = await userRest().post("/list/search", param, config);
    return restResult(response)
}


export async function userIdSearch(param, config) {
    const { user_id, opt } = param;
    // pub name: Option<bool>,
    // pub info: Option<bool>,
    // pub address: Option<bool>,
    // pub external: Option<Vec<String>>,
    // pub email: Option<Vec<i8>>,
    // pub mobile: Option<Vec<i8>>,
    var param = {
        user_id: parseInt(user_id),
        base: !!opt,
        name: !!opt,
        info: !!opt,
        address: !!opt,
        external: opt ? [] : null,
        email: opt ? [] : null,
        mobile: opt ? [] : null,
    };
    let response = await userRest().post("/list/id_search", param, config);
    return restResult(response)
}