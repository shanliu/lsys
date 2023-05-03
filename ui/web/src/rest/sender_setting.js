import isEmail from "validator/lib/isEmail";
import { fialResult, restResult, sessionRest } from "../utils/rest";
import { isDomain } from "../utils/utils";

export const SenderTypeSms = 1;
export const SenderTypeMail = 2;

export const SenderType = [

    {
        key: SenderTypeMail,
        val: '邮件模板'
    },
    // {
    //     key: SenderTypeSms,
    //     val: '短信模板'
    // }
];

export const MessageLogType = [

    {
        key: 1,
        val: '发送操作'
    },
    {
        key: 2,
        val: '取消发送'
    }
];
export const MessageLogStatus = [

    {
        key: 2,
        val: '成功'
    },
    {
        key: 3,
        val: '失败'
    },
    {
        key: 4,
        val: '接口取消'
    },
    {
        key: 5,
        val: '后台取消'
    }
];

// sms
export const MessageStatus = [
    {
        key: 1,
        val: '待发送'
    },
    {
        key: 2,
        val: '发送完成'
    },
    {
        key: 3,
        val: '发送失败'
    },
    {
        key: 4,
        val: '已取消'
    }
];



function senderSettingRest(type) {
    return sessionRest(`/api/sender/${type}`)
};

//config

export async function senderAddConfig(type, param, config) {
    const { app_id, priority, user_id, config_type, config_data } = param;
    let data = {};
    var errors = {};
    if (app_id < 0) {
        errors.app_id = "未提供app";
    }
    if (user_id >= 0) {
        data.user_id = parseInt(user_id);
    }
    data.app_id = parseInt(app_id);
    data.priority = parseInt(priority);
    if (config_type <= 0) {
        errors.config_type = "类型未知";
    }
    data.config_type = parseInt(config_type);
    switch (data.config_type) {
        case 2:
            let range_time = parseInt(config_data.range_time);
            let max_send = parseInt(config_data.max_send);
            if (range_time <= 0 || max_send <= 0) {
                errors.config_data = "请设置限制数量跟时间";
            } else {
                data.config_data = {
                    range_time,
                    max_send
                };
            }
            break;
        case 1:
            data.config_data = '';
            break;
        case 4:
            let tpl = config_data + '';
            if (/^\s*$/.test(tpl)) {
                errors.config_data = "模板不能为空";
            } else {
                data.config_data = tpl;
            }
            break;
        case 3:
            let num = parseInt(config_data);
            if (num <= 0) {
                errors.config_data = "数量必须大于0";
            } else {
                data.config_data = num;
            }
            break;
        case 10:
            let mobile = config_data + '';
            if (!/^1\d{10}$/.test(mobile)) {
                errors.config_data = "请提供正确手机号";
            } else {
                data.config_data = mobile;
            }
            break;
        case 20:
            let mail = config_data + '';
            if (!isEmail(mail)) {
                errors.config_data = "请提供正确邮箱";
            } else {
                data.config_data = mail;
            }
            break;
        case 21:
            let domain = config_data + '';
            if (!isDomain(domain, false)) {
                errors.config_data = "请提供正确域名";
            } else {
                data.config_data = domain;
            }
            break;
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest(type).post("config_add", data, config);
    return restResult(response)
}
export async function senderListConfig(type, param, config) {
    const { id, app_id, user_id, page, page_size } = param;
    let data = {
        "page": {
            page: parseInt(page) >= 0 ? (parseInt(page) + 1) : 1,
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 10
        }
    };
    if (id > 0) {
        data.id = parseInt(id);
    }
    if (app_id >= 0) {
        data.app_id = parseInt(app_id);
    }
    if (user_id > 0) {
        data.user_id = parseInt(user_id);
    }
    let response = await senderSettingRest(type).post("config_list", data, config);
    return restResult(response, ['not_found'])
}
export async function senderDelConfig(type, param, config) {
    const { config_id } = param;
    let data = {};
    if (config_id > 0) {
        data.config_id = parseInt(config_id);
    }
    let response = await senderSettingRest(type).post("config_del", data, config);
    return restResult(response)
}



//message

export async function senderListAppMessage(type, param, config) {
    const { app_id, user_id, tpl_id, mobile, status, start_pos, end_pos, page_size } = param;
    let data = {
        count_num: false,
        limit: {
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 10,
            next: false,
            more: true,
        }
    };
    if (end_pos && end_pos != '0' && end_pos != '') {
        data.limit = {
            ...data.limit,
            pos: end_pos,
            eq_pos: false,
            next: true,
        }
    } else if (start_pos && start_pos != '0' && start_pos != '') {
        data.limit = {
            ...data.limit,
            pos: start_pos,
            eq_pos: true,
            next: false,
        }
    }
    let errors = {};
    if (parseInt(user_id) > 0) {
        data.user_id = parseInt(user_id);
    }
    if (parseInt(status) > 0) {
        data.status = parseInt(status);
    }
    if (parseInt(app_id) >= 0) {
        data.app_id = parseInt(app_id);
    }
    if (typeof tpl_id == "string" && tpl_id.length > 0) {
        data.tpl_id = tpl_id;
    }
    if (typeof mobile == "string" && mobile.length > 0) {
        data.mobile = mobile;
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest(type).post("/message_list", data, config);
    return restResult(response, ['not_found'])
}


export async function senderListAppMessageLog(type, param, config) {
    const { message_id, page, page_size } = param;
    let data = {
        count_num: true,
        message_id: message_id.toString(),
        page: {
            page: parseInt(page) >= 0 ? (parseInt(page) + 1) : 1,
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 10
        }
    };
    let response = await senderSettingRest(type).post("/message_log", data, config);
    return restResult(response, ['not_found'])
}



export async function senderCancelAppMessage(type, param, config) {
    const { message_id } = param;
    if (message_id <= 0) {
        return fialResult({ message_id: "缺少ID" });
    }
    let response = await senderSettingRest(type).post("/message_cancel", {
        message_id: message_id.toString(),
    }, config);
    return restResult(response)
}

export async function senderSeeAppMessage(type, param, config) {
    const { message_id } = param;
    if (message_id <= 0) {
        return fialResult({ message_id: "缺少ID" });
    }
    let response = await senderSettingRest(type).post("/message_body", {
        message_id: message_id.toString(),
    }, config);
    return restResult(response)
}

//sms


export const SmsLimitStatusMap = [
    {
        key: 1,
        val: '关闭发送功能',
        show: (data) => {
            return "已禁止发送";
        }
    },
    {
        key: 2,
        val: '发送频率限制',
        show: (d) => {
            let data = JSON.parse(d);
            if (!data) return "数据解析失败";
            return `${data.range_time} 秒内最多发送 ${data.max_send} 条信息`;
        }
    },
    {
        key: 3,
        val: '批量发送量限制',
        show: (data) => {
            return "单次最大发送量:" + data + "条";
        }
    },
    {
        key: 4,
        val: '指定模板忽略限制',
        show: (data) => {
            return "模板[" + data + "]不限制发送频率";
        }
    },
    {
        key: 10,
        val: '屏蔽指定号码',
        show: (data) => {
            return "号码[" + data + "]已被屏蔽";
        }
    },

];

export async function smsListAliConfig(param, config) {
    const { id, full_data } = param;
    let data = {};
    if (id > 0) {
        data.ids = [parseInt(id)];
    }
    if (typeof full_data == 'boolean') {
        data.full_data = full_data;
    }
    let response = await senderSettingRest("smser").post("ali_config_list", data, config);
    return restResult(response, ['not_found'])
}
export async function smsAddAliConfig(param, config) {
    const { name, access_id, access_secret } = param;
    var errors = {};
    if (typeof name !== "string" || name.length < 1) {
        errors.name = "应用名不能为空";
    }
    if (typeof access_id !== "string" || access_id.length < 12) {
        errors.access_id = "access id 错误";
    }
    if (typeof access_secret !== "string" || access_secret.length < 12) {
        errors.access_secret = "access secret 错误";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("smser").post("/ali_config_add", param, config);
    return restResult(response)
}
export async function smsEditAliConfig(param, config) {
    const { id, name, access_id, access_secret } = param;
    var errors = {};
    if (parseInt(id) <= 0) {
        errors.name = "id未知";
    }
    if (typeof name !== "string" || name.length < 1) {
        errors.name = "应用名不能为空";
    }
    if (typeof access_id !== "string" || access_id.length < 12) {
        errors.access_id = "access id 错误";
    }
    if (typeof access_secret !== "string" || access_secret.length < 12) {
        errors.access_secret = "access secret 错误";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("smser").post("/ali_config_edit", {
        ...param,
        id: parseInt(id),
    }, config);
    return restResult(response)
}
export async function smsDelAliConfig(param, config) {
    const { id } = param;
    let response = await senderSettingRest("smser").post("/ali_config_del", {
        "id": parseInt(id),
    }, config);
    return restResult(response)
}


export async function smsListAppAliConfig(param, config) {
    const { id, app_id, tpl, user_id, page, page_size } = param;
    let data = {
        "page": {
            page: parseInt(page) >= 0 ? (parseInt(page) + 1) : 1,
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 10
        }
    };
    let errors = {};
    if (parseInt(user_id) > 0) {
        data.user_id = parseInt(user_id);
    }
    if (parseInt(app_id) >= 0) {
        data.app_id = parseInt(app_id);
    }
    if (parseInt(id) > 0) {
        data.id = parseInt(id);
    }
    if (typeof tpl !== "string" || tpl.length > 1) {
        data.tpl = tpl;
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("smser").post("/ali_app_config_list", data, config);
    return restResult(response, ['not_found'])
}


export async function smsAddAppAliConfig(param, config) {
    const { app_id, user_id, aliconfig_id, name, tpl_id, aliyun_sms_tpl, aliyun_sign_name, max_try_num } = param;
    let errors = {};
    let data = {
        name: name,
        ali_config_id: parseInt(aliconfig_id),
        tpl_id: tpl_id,
        aliyun_sms_tpl: aliyun_sms_tpl,
        aliyun_sign_name: aliyun_sign_name,
        try_num: max_try_num > 0 ? parseInt(max_try_num) : 1
    };
    if (user_id >= 0) {
        data.user_id = parseInt(user_id);
    }
    if (app_id < 0) {
        errors.app_id = "请提供操作应用";
    } else {
        data.app_id = parseInt(app_id);
    }
    if (typeof name !== "string" || name.length < 1) {
        errors.name = "名称必须大于等于2个字符";
    }
    if (typeof tpl_id !== "string" || tpl_id.length <= 2) {
        errors.tpl_id = "模板名必须大于等于3个字符";
    }
    if (typeof aliyun_sms_tpl !== "string" || aliyun_sms_tpl.length <= 2) {
        errors.aliyun_sms_tpl = "阿里云模板错误";
    }
    if (typeof aliyun_sign_name !== "string" || aliyun_sign_name.length < 2) {
        errors.aliyun_sign_name = "阿里云签名错误";
    }

    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("smser").post("/ali_app_config_add", data, config);
    return restResult(response)
}

export async function smsDelAppAliConfig(param, config) {
    const { id } = param;
    let response = await senderSettingRest("smser").post("/ali_app_config_del", {
        "app_config_id": parseInt(id),
    }, config);
    return restResult(response)
}


//mail



export const MailLimitStatusMap = [
    {
        key: 1,
        val: '关闭发送功能',
        show: (data) => {
            return "已禁止发送";
        }
    },
    {
        key: 2,
        val: '发送频率限制',
        show: (d) => {
            let data = JSON.parse(d);
            if (!data) return "数据解析失败";
            return `${data.range_time} 秒内最多发送 ${data.max_send} 条信息`;
        }
    },
    {
        key: 3,
        val: '批量发送量限制',
        show: (data) => {
            return "单次最大发送量:" + data + "条";
        }
    },
    {
        key: 4,
        val: '指定模板忽略限制',
        show: (data) => {
            return "模板[" + data + "]不限制发送频率";
        }
    },
    {
        key: 20,
        val: '屏蔽指定邮箱',
        show: (data) => {
            return "邮箱[" + data + "]已被屏蔽";
        }
    },
    {
        key: 21,
        val: '屏蔽指定域名',
        show: (data) => {
            return "域名[" + data + "]已被屏蔽";
        }
    }
];
export async function mailListSmtpConfig(param, config) {
    const { id, full_data } = param;
    let data = {};
    if (id > 0) {
        data.ids = [parseInt(id)];
    }
    if (typeof full_data == 'boolean') {
        data.full_data = full_data;
    }
    let response = await senderSettingRest("mailer").post("smtp_config_list", data, config);
    return restResult(response, ['not_found'])
}

export async function mailCheckSmtpConfig(param, config) {
    const { id } = param;
    let response = await senderSettingRest("mailer").post("/smtp_config_check", {
        "app_config_id": parseInt(id),
    }, config);
    return restResult(response)
}
export async function mailAddSmtpConfig(param, config) {
    const { name,
        host,
        port,
        timeout,
        user,
        email,
        password,
        tls_domain, } = param;
    var errors = {};
    if (typeof name !== "string" || name.length < 1) {
        errors.name = "应用名不能为空";
    }
    if (typeof host !== "string" || host.length < 1) {
        errors.host = "主机不能为空";
    }
    if (!isDomain(host, true)) {
        errors.host = "主机为域名或IP";
    }
    if (parseInt(port) <= 0) {
        errors.port = "端口不能为空";
    }
    if (parseInt(timeout) <= 0) {
        errors.timeout = "超时不能为空";
    }
    if (typeof tls_domain == "string" && tls_domain.length > 0 && !isDomain(tls_domain, true)) {
        errors.tls_domain = "tls_domain需要为域名或IP";
    }
    if (typeof email == "string" && email.length > 0 && !isEmail(email)) {
        errors.email = "请输入邮箱或留空";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("mailer").post("/smtp_config_add", {
        name: name + '',
        host: host + '',
        port: parseInt(port),
        timeout: parseInt(timeout),
        user: user + '',
        email: email + '',
        password: password + '',
        tls_domain: tls_domain + '',
    }, config);
    return restResult(response)
}
export async function mailEditSmtpConfig(param, config) {
    const { id, name,
        host,
        port,
        timeout,
        user,
        email,
        password,
        tls_domain, } = param;
    var errors = {};
    var sid = parseInt(id);
    if (sid <= 0 || isNaN(sid)) {
        errors.name = "ID异常";
    }
    if (typeof name !== "string" || name.length < 1) {
        errors.name = "应用名不能为空";
    }
    if (typeof host !== "string" || host.length < 1) {
        errors.host = "主机不能为空";
    }
    if (!isDomain(host, true)) {
        errors.host = "主机为域名或IP";
    }
    if (typeof tls_domain == "string" && tls_domain.length > 0 && !isDomain(tls_domain, true)) {
        errors.tls_domain = "tls_domain需要为域名或IP";
    }
    if (parseInt(port) <= 0) {
        errors.port = "端口不能为空";
    }
    if (parseInt(timeout) <= 0) {
        errors.timeout = "超时不能为空";
    }
    if (typeof email == "string" && email.length > 0 && !isEmail(email)) {
        errors.email = "请输入邮箱或留空";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("mailer").post("/smtp_config_edit", {
        name: name + '',
        host: host + '',
        port: parseInt(port),
        timeout: parseInt(timeout),
        user: user + '',
        email: email + '',
        password: password + '',
        tls_domain: tls_domain + '',
        id: sid,
    }, config);
    return restResult(response)
}
export async function mailDelSmtpConfig(param, config) {
    const { id } = param;
    let response = await senderSettingRest("mailer").post("/smtp_config_del", {
        "id": parseInt(id),
    }, config);
    return restResult(response)
}


export async function mailListAppSmtpConfig(param, config) {
    const { id, app_id, tpl, user_id, page, page_size } = param;
    let data = {
        "page": {
            page: parseInt(page) >= 0 ? (parseInt(page) + 1) : 1,
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 10
        }
    };
    let errors = {};
    if (parseInt(user_id) > 0) {
        data.user_id = parseInt(user_id);
    }
    if (parseInt(app_id) >= 0) {
        data.app_id = parseInt(app_id);
    }
    if (parseInt(id) > 0) {
        data.id = parseInt(id);
    }
    if (typeof tpl !== "string" || tpl.length > 1) {
        data.tpl = tpl;
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("mailer").post("/smtp_app_config_list", data, config);
    return restResult(response, ['not_found'])
}


export async function mailAddAppSmtpConfig(param, config) {
    const { app_id, user_id, smtp_config_id, name, tpl_id, from_email, body_tpl_id, subject_tpl_id, try_num } = param;
    let errors = {};

    let data = {
        name: name,
        smtp_config_id: parseInt(smtp_config_id),
        tpl_id: tpl_id,
        from_email: from_email,
        body_tpl_id: body_tpl_id,
        subject_tpl_id: subject_tpl_id,
        try_num: try_num > 0 ? parseInt(try_num) : 1
    };
    if (user_id >= 0) {
        data.user_id = parseInt(user_id);
    }
    if (parseInt(smtp_config_id) <= 0 || isNaN(parseInt(smtp_config_id))) {
        errors.smtp_config_id = "请选择smtp配置";
    }
    if (app_id < 0) {
        errors.app_id = "请提供操作应用";
    } else {
        data.app_id = parseInt(app_id);
    }
    if (typeof name !== "string" || name.length < 1) {
        errors.name = "名称必须大于等于2个字符";
    }
    if (typeof tpl_id !== "string" || tpl_id.length <= 2) {
        errors.tpl_id = "模板名必须大于等于3个字符";
    }
    if (typeof body_tpl_id !== "string" || body_tpl_id.length <= 2) {
        errors.body_tpl_id = "请选择内容模板";
    }
    if (typeof from_email !== "string" || !isEmail(from_email)) {
        errors.from_email = "来源邮箱不能为空";
    }
    if (typeof subject_tpl_id !== "string" || subject_tpl_id.length <= 1) {
        errors.subject_tpl_id = "请选择标题模板";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("mailer").post("/smtp_app_config_add", data, config);
    return restResult(response)
}

export async function mailDelAppSmtpConfig(param, config) {
    const { id } = param;
    let response = await senderSettingRest("mailer").post("/smtp_app_config_del", {
        "app_config_id": parseInt(id),
    }, config);
    return restResult(response)
}


export async function tplsAddConfig(param, config) {
    const { tpl_id, tpl_data, sender_type, user_id } = param;
    let errors = {};
    if (typeof tpl_id !== "string" || tpl_id.length < 1) {
        errors.tpl_id = "请输入模板内容";
    }

    if (typeof tpl_data !== "string" || tpl_data.length < 1) {
        errors.tpl_data = "请输入模板内容";
    }

    if (Object.keys(errors).length) {
        return fialResult(errors);
    }

    let data = {
        sender_type: parseInt(sender_type),
        tpl_id: tpl_id,

        tpl_data: tpl_data,
        user_id: parseInt(user_id),
    };
    let response = await senderSettingRest("tpls").post("/add", data, config);
    return restResult(response)
}

export async function tplsEditConfig(param, config) {
    const { id, tpl_data } = param;
    let errors = {};
    if (typeof tpl_data !== "string" || tpl_data.length < 1) {
        errors.name = "请输入模板内容";
    }

    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("tpls").post("/edit", {
        id: parseInt(id),

        tpl_data: tpl_data,
    }, config);
    return restResult(response)
}


export async function tplsListConfig(param, config) {
    const { tpl_id, sender_type, id, user_id, page, page_size } = param;
    let data = {
        user_id: parseInt(user_id),
        sender_type: parseInt(sender_type),
        count_num: true,
        page: {
            page: parseInt(page) >= 0 ? (parseInt(page) + 1) : 1,
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 10
        }
    }
    let errors = {};
    if (!isNaN(parseInt(id)) && parseInt(id) > 0) {
        errors.id = parseInt(id);
    }
    if (typeof tpl_id == "string" && tpl_id.length > 1) {
        data.tpl_id = tpl_id;
    }
    let response = await senderSettingRest("tpls").post("/list", data, config);
    return restResult(response)
}

export async function tplsDelConfig(param, config) {
    const { id } = param;
    let response = await senderSettingRest("tpls").post("/del", {
        "id": parseInt(id),
    }, config);
    return restResult(response)
}