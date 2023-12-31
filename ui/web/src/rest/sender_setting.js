import isEmail from "validator/lib/isEmail";
import { fialResult, restResult, sessionRest } from "../utils/rest";
import { isCallbackKey, isDomain } from "../utils/utils";
import dayjs from "dayjs";

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
        val: '新增发送'
    },
    {
        key: 2,
        val: '发送消息'
    },
    {
        key: 3,
        val: '取消消息'
    }
];


export const MessageLogStatus = [
    {
        key: 2,
        val: '成功发送'
    },
    {
        key: 3,
        val: '发送失败'
    },
    {
        key: 5,
        val: '后台取消'
    }, {
        key: 6,
        val: '回调通知成功'
    }, {
        key: 7,
        val: '回调通知失败'
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
        val: '已发送'
    },
    {
        key: 3,
        val: '已失败'
    },
    {
        key: 4,
        val: '已取消'
    },
    {
        key: 5,
        val: '已接收'
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
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 25
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

export async function mailListAppMessage(param, config) {
    const { app_id, user_id,sn_id, tpl_id, to_mail, status, start_pos, end_pos, page_size } = param;
    let data = {
        count_num: false,
        limit: {
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 25,
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
    if (parseInt(sn_id) >= 0) {
        data.snid = sn_id.toString();
    }
    if (typeof to_mail == "string" && to_mail.length > 0) {
        data.to_mail = to_mail;
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("mailer").post("/message_list", data, config);
    return restResult(response, ['not_found'])
}

export async function smsListAppMessage(param, config) {
    const { app_id, user_id,sn_id, tpl_id, mobile, status, start_pos, end_pos, page_size } = param;
    let data = {
        count_num: false,
        limit: {
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 25,
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
    if (parseInt(sn_id) >= 0) {
        data.snid = sn_id.toString();
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
    let response = await senderSettingRest("smser").post("/message_list", data, config);
    return restResult(response, ['not_found'])
}


export async function senderListAppMessageLog(type, param, config) {
    const { message_id, page, page_size } = param;
    let data = {
        count_num: true,
        message_id: message_id.toString(),
        page: {
            page: parseInt(page) >= 0 ? (parseInt(page) + 1) : 1,
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 25
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



export async function senderSetMessageNotify(type, param, config) {
    const { url, app_id } = param;
    if (app_id <= 0) {
        return fialResult({ app_id: "缺少ID" });
    }
    if (url.substr(0, 7) != "http://" && url.substr(0, 8) != "https://") {
        return fialResult({ url: "url错误" });
    }
    let response = await senderSettingRest(type).post("/notify_set_config", {
        app_id: parseInt(app_id),
        url: url
    }, config);
    return restResult(response)
}



export async function senderGetMessageNotify(type, config) {
    let response = await senderSettingRest(type).post("/notify_get_config", {

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



export async function smsListTplConfig(param, config) {
    const { id, app_id, tpl, user_id, page, page_size, app_info } = param;
    let data = {
        count_num: true,
        app_info: app_info ? true : false,
        page: {
            page: parseInt(page) >= 0 ? (parseInt(page) + 1) : 1,
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 25
        }
    };
    let errors = {};
    if (parseInt(user_id) >= 0) {
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
    let response = await senderSettingRest("smser").post("/tpl_config_list", data, config);
    return restResult(response, ['not_found'])
}

export async function smsDelTplConfig(param, config) {
    const { id } = param;
    let response = await senderSettingRest("smser").post("/tpl_config_del", {
        "app_config_id": parseInt(id),
    }, config);
    return restResult(response)
}




export async function smsAppSend(param, config) {
    const { tpl_id, data, mobile, send_time, max_try } = param;
    let errors = {};
    if (parseInt(tpl_id) <= 0) {
        errors.tpl_id = "模板未选择"
    }
    if (!mobile || mobile.length <= 0) {
        errors.mobile = "请输入接收手机号"
    }
    for (var tmp of mobile) {
        if (!tmp || !/^1[0-9]{10}$/.test(tmp)) {
            errors.mobile = "手机格式错误";
            break;
        }
    }

    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let sp = {
        tpl_id: parseInt(tpl_id),
        data: data,
        mobile: mobile,
        send_time: send_time,
    }
    if (max_try > 0) sp.max_try = max_try;
    let response = await senderSettingRest("smser").post("/message_send", sp, config);
    return restResult(response)
}


//阿里短信相关接口
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
    const { name, access_id, access_secret, region, callback_key } = param;
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
    if (typeof region !== "string" || region.length < 3) {
        errors.region = "区域不能为空";
    }
    if (typeof callback_key !== "string" || !isCallbackKey(callback_key)) {
        errors.callback_key = "回调KEY错误,只能包含数字字母的字符串";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("smser").post("/ali_config_add", param, config);
    return restResult(response)
}
export async function smsEditAliConfig(param, config) {
    const { id, name, access_id, access_secret, region, callback_key } = param;
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
    if (typeof region !== "string" || region.length < 3) {
        errors.region = "区域不能为空";
    }
    if (typeof access_secret !== "string" || access_secret.length < 12) {
        errors.access_secret = "access secret 错误";
    }
    if (typeof callback_key !== "string" || !isCallbackKey(callback_key)) {
        errors.callback_key = "回调KEY错误,只能包含数字字母的字符串";
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
export async function smsAddAppAliConfig(param, config) {
    const { app_id, user_id, aliconfig_id, name, tpl_id, aliyun_sms_tpl, aliyun_sign_name } = param;
    let errors = {};
    let data = {
        name: name,
        ali_config_id: parseInt(aliconfig_id),
        tpl_id: tpl_id,
        aliyun_sms_tpl: aliyun_sms_tpl,
        aliyun_sign_name: aliyun_sign_name,

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

//华为云短信相关接口
export async function smsListHwConfig(param, config) {
    const { id, full_data } = param;
    let data = {};
    if (id > 0) {
        data.ids = [parseInt(id)];
    }
    if (typeof full_data == 'boolean') {
        data.full_data = full_data;
    }
    let response = await senderSettingRest("smser").post("hw_config_list", data, config);
    return restResult(response, ['not_found'])
}
export async function smsAddHwConfig(param, config) {
    const { name, app_key, app_secret, url, callback_key } = param;
    var errors = {};
    if (typeof name !== "string" || name.length < 1) {
        errors.name = "应用名不能为空";
    }
    if (typeof app_key !== "string" || app_key.length < 12) {
        errors.app_key = " key 错误";
    }
    if (typeof app_secret !== "string" || app_secret.length < 12) {
        errors.app_secret = " secret 错误";
    }
    if (typeof url !== "string" || (
        url.substr(0, 7) != 'http://' &&
        url.substr(0, 8) != 'https://'
    )) {
        errors.url = "提供URL";
    }
    if (typeof callback_key !== "string" || !isCallbackKey(callback_key)) {
        errors.callback_key = "回调KEY错误,只能包含数字字母的字符串";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("smser").post("/hw_config_add", param, config);
    return restResult(response)
}
export async function smsEditHwConfig(param, config) {
    const { id, name, app_key, app_secret, url } = param;
    var errors = {};
    if (parseInt(id) <= 0) {
        errors.name = "id未知";
    }
    if (typeof name !== "string" || name.length < 1) {
        errors.name = "应用名不能为空";
    }
    if (typeof app_key !== "string" || app_key.length < 12) {
        errors.app_key = "access id 错误";
    }
    if (typeof app_secret !== "string" || app_secret.length < 12) {
        errors.app_secret = "access secret 错误";
    }
    if (typeof url !== "string" || (
        url.substr(0, 7) != 'http://' &&
        url.substr(0, 8) != 'https://'
    )) {
        errors.url = "提供URL";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("smser").post("/hw_config_edit", {
        ...param,
        id: parseInt(id),
    }, config);
    return restResult(response)
}
export async function smsDelHwConfig(param, config) {
    const { id } = param;
    let response = await senderSettingRest("smser").post("/hw_config_del", {
        "id": parseInt(id),
    }, config);
    return restResult(response)
}
export async function smsAddAppHwConfig(param, config) {
    const { app_id, user_id, hw_config_id, name, tpl_id, template_map, signature, sender, template_id, } = param;
    let errors = {};
    let data = {
        name: name,
        hw_config_id: parseInt(hw_config_id),
        tpl_id: tpl_id,
        template_map: map_data,
        signature: signature,
        sender: sender,
        template_id: template_id,
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
    if (typeof template_id !== "string" || template_id.length <= 2) {
        errors.template_id = "模板ID错误";
    }
    if (typeof signature !== "string" || signature.length <= 2) {
        errors.signature = "模板签名错误";
    }
    if (typeof sender !== "string" || sender.length < 1) {
        errors.sender = "模板通道错误";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let map_data = (template_map + '').split(",");
    map_data = map_data.filter((e) => e)
    map_data = map_data.join(",")
    map_data = map_data.split("，")
    map_data = map_data.filter((e) => e)
    data.template_map = map_data.join(",")

    let response = await senderSettingRest("smser").post("/hw_app_config_add", data, config);
    return restResult(response)
}


//腾讯云短信相关接口
export async function smsListTenConfig(param, config) {
    const { id, full_data } = param;
    let data = {};
    if (id > 0) {
        data.ids = [parseInt(id)];
    }
    if (typeof full_data == 'boolean') {
        data.full_data = full_data;
    }
    let response = await senderSettingRest("smser").post("ten_config_list", data, config);
    return restResult(response, ['not_found'])
}
export async function smsAddTenConfig(param, config) {
    const { name, secret_id, secret_key, sms_app_id, region, callback_key } = param;
    var errors = {};
    if (typeof name !== "string" || name.length < 1) {
        errors.name = "应用名不能为空";
    }
    if (typeof secret_id !== "string" || secret_id.length < 12) {
        errors.secret_id = " id 错误";
    }
    if (typeof secret_key !== "string" || secret_key.length < 12) {
        errors.secret_key = " secret 错误";
    }
    if (typeof sms_app_id !== "string" || sms_app_id.length <= 2) {
        errors.sms_app_id = "腾讯短信应用ID错误";
    }
    if (typeof region !== "string" || region.length < 3) {
        errors.region = "区域不能为空";
    }
    if (typeof callback_key !== "string" || !isCallbackKey(callback_key)) {
        errors.callback_key = "回调KEY错误,只能包含数字字母的字符串";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("smser").post("/ten_config_add", param, config);
    return restResult(response)
}
export async function smsEditTenConfig(param, config) {
    const { id, name, secret_id, secret_key, sms_app_id, region } = param;
    var errors = {};
    if (parseInt(id) <= 0) {
        errors.name = "id未知";
    }
    if (typeof name !== "string" || name.length < 1) {
        errors.name = "应用名不能为空";
    }
    if (typeof secret_id !== "string" || secret_id.length < 12) {
        errors.secret_id = " id 错误";
    }
    if (typeof secret_key !== "string" || secret_key.length < 12) {
        errors.secret_key = " secret 错误";
    }
    if (typeof sms_app_id !== "string" || sms_app_id.length <= 2) {
        errors.sms_app_id = "腾讯短信应用ID错误";
    }
    if (typeof region !== "string" || region.length < 3) {
        errors.region = "区域不能为空";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("smser").post("/ten_config_edit", {
        ...param,
        id: parseInt(id),
    }, config);
    return restResult(response)
}
export async function smsDelTenConfig(param, config) {
    const { id } = param;
    let response = await senderSettingRest("smser").post("/ten_config_del", {
        "id": parseInt(id),
    }, config);
    return restResult(response)
}
export async function smsAddAppTenConfig(param, config) {
    const { app_id, user_id, config_id, name, tpl_id, template_map, sign_name, template_id, } = param;
    let errors = {};
    let data = {
        name: name,
        config_id: parseInt(config_id),
        tpl_id: tpl_id,


        sign_name: sign_name,
        template_id: template_id,
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
    if (typeof template_id !== "string" || template_id.length <= 2) {
        errors.template_id = "模板ID错误";
    }


    if (typeof sign_name !== "string" || sign_name.length < 1) {
        errors.sign_name = "签名必填";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let map_data = (template_map + '').split(",");
    map_data = map_data.filter((e) => e)
    map_data = map_data.join(",")
    map_data = map_data.split("，")
    map_data = map_data.filter((e) => e)
    data.template_map = map_data.join(",")

    let response = await senderSettingRest("smser").post("/ten_app_config_add", data, config);
    return restResult(response)
}



//融连云短信相关接口
export async function smsListCloOpenConfig(param, config) {
    const { id, full_data } = param;
    let data = {};
    if (id > 0) {
        data.ids = [parseInt(id)];
    }
    if (typeof full_data == 'boolean') {
        data.full_data = full_data;
    }
    let response = await senderSettingRest("smser").post("cloopen_config_list", data, config);
    return restResult(response, ['not_found'])
}
export async function smsAddCloOpenConfig(param, config) {
    const { name, account_sid, account_token, callback_key, sms_app_id } = param;
    var errors = {};
    if (typeof name !== "string" || name.length < 1) {
        errors.name = "应用名不能为空";
    }
    if (typeof account_sid !== "string" || account_sid.length < 12) {
        errors.account_sid = " id 错误";
    }
    if (typeof account_token !== "string" || account_token.length < 12) {
        errors.account_token = " secret 错误";
    }
    if (typeof sms_app_id !== "string" || sms_app_id.length < 2) {
        errors.sms_app_id = " 应用ID错误";
    }
    if (typeof callback_key !== "string" || !isCallbackKey(callback_key)) {
        errors.callback_key = "回调KEY错误,只能包含数字字母的字符串";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("smser").post("/cloopen_config_add", param, config);
    return restResult(response)
}
export async function smsEditCloOpenConfig(param, config) {
    const { id, name, account_sid, account_token, sms_app_id } = param;
    var errors = {};
    if (parseInt(id) <= 0) {
        errors.name = "id未知";
    }
    if (typeof name !== "string" || name.length < 1) {
        errors.name = "应用名不能为空";
    }

    if (typeof account_sid !== "string" || account_sid.length < 12) {
        errors.account_sid = " id 错误";
    }
    if (typeof sms_app_id !== "string" || sms_app_id.length < 2) {
        errors.sms_app_id = " 应用ID错误";
    }
    if (typeof account_token !== "string" || account_token.length < 12) {
        errors.account_token = " secret 错误";
    }

    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("smser").post("/cloopen_config_edit", {
        ...param,
        id: parseInt(id),
    }, config);
    return restResult(response)
}
export async function smsDelCloOpenConfig(param, config) {
    const { id } = param;
    let response = await senderSettingRest("smser").post("/cloopen_config_del", {
        "id": parseInt(id),
    }, config);
    return restResult(response)
}
export async function smsAddAppCloOpenConfig(param, config) {
    const { app_id, user_id, config_id, name, tpl_id, template_map, template_id, } = param;
    let errors = {};
    let data = {
        name: name,
        config_id: parseInt(config_id),
        tpl_id: tpl_id,

        template_id: template_id,
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
    if (typeof template_id !== "string" || template_id.length <= 2) {
        errors.template_id = "模板ID错误";
    }


    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let map_data = (template_map + '').split(",");
    map_data = map_data.filter((e) => e)
    map_data = map_data.join(",")
    map_data = map_data.split("，")
    map_data = map_data.filter((e) => e)
    data.template_map = map_data.join(",")

    let response = await senderSettingRest("smser").post("/cloopen_app_config_add", data, config);
    return restResult(response)
}
//JD云短信相关接口
export async function smsListJdConfig(param, config) {
    const { id, full_data } = param;
    let data = {};
    if (id > 0) {
        data.ids = [parseInt(id)];
    }
    if (typeof full_data == 'boolean') {
        data.full_data = full_data;
    }
    let response = await senderSettingRest("smser").post("jd_config_list", data, config);
    return restResult(response, ['not_found'])
}

// pub name: String,
// pub region: String,
// pub secret_id: String,
// pub secret_key: String,
// pub limit: Option<u16>,
export async function smsAddJdConfig(param, config) {
    const { name, access_key, access_secret, region } = param;
    var errors = {};
    if (typeof name !== "string" || name.length < 1) {
        errors.name = "应用名不能为空";
    }
    if (typeof access_key !== "string" || access_key.length < 12) {
        errors.access_key = " id 错误";
    }
    if (typeof access_secret !== "string" || access_secret.length < 12) {
        errors.access_secret = " secret 错误";
    }
    if (typeof region !== "string" || region.length < 3) {
        errors.region = "区域不能为空";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("smser").post("/jd_config_add", param, config);
    return restResult(response)
}
export async function smsEditJdConfig(param, config) {
    const { id, name, access_key, access_secret, region } = param;
    var errors = {};
    if (parseInt(id) <= 0) {
        errors.name = "id未知";
    }
    if (typeof name !== "string" || name.length < 1) {
        errors.name = "应用名不能为空";
    }
    if (typeof access_key !== "string" || access_key.length < 12) {
        errors.access_key = " secret_id 错误";
    }
    if (typeof access_secret !== "string" || access_secret.length < 12) {
        errors.access_secret = " secret_key 错误";
    }
    if (typeof region !== "string" || region.length < 3) {
        errors.region = "区域不能为空";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("smser").post("/jd_config_edit", {
        ...param,
        id: parseInt(id),
    }, config);
    return restResult(response)
}
export async function smsDelJdConfig(param, config) {
    const { id } = param;
    let response = await senderSettingRest("smser").post("/jd_config_del", {
        "id": parseInt(id),
    }, config);
    return restResult(response)
}
export async function smsAddAppJdConfig(param, config) {
    const { app_id, user_id, config_id, name, tpl_id, template_map, sign_id, template_id, } = param;
    let errors = {};
    let data = {
        name: name,
        config_id: parseInt(config_id),
        tpl_id: tpl_id,

        sign_id: sign_id,
        template_id: template_id,
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
    if (typeof template_id !== "string" || template_id.length <= 2) {
        errors.template_id = "模板ID错误";
    }

    if (typeof sign_id !== "string" || sign_id.length < 1) {
        errors.sign_id = "签名必填";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let map_data = (template_map + '').split(",");
    map_data = map_data.filter((e) => e)
    map_data = map_data.join(",")
    map_data = map_data.split("，")
    map_data = map_data.filter((e) => e)
    data.template_map = map_data.join(",")

    let response = await senderSettingRest("smser").post("/jd_app_config_add", data, config);
    return restResult(response)
}

//网易短信相关接口
export async function smsListNeteaseConfig(param, config) {
    const { id, full_data } = param;
    let data = {};
    if (id > 0) {
        data.ids = [parseInt(id)];
    }
    if (typeof full_data == 'boolean') {
        data.full_data = full_data;
    }
    let response = await senderSettingRest("smser").post("netease_config_list", data, config);
    return restResult(response, ['not_found'])
}
export async function smsAddNeteaseConfig(param, config) {
    const { name, access_key, access_secret } = param;
    var errors = {};
    if (typeof name !== "string" || name.length < 1) {
        errors.name = "应用名不能为空";
    }
    if (typeof access_key !== "string" || access_key.length < 12) {
        errors.access_key = " access key 错误";
    }
    if (typeof access_secret !== "string" || access_secret.length < 12) {
        errors.access_secret = " access secret 错误";
    }

    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("smser").post("/netease_config_add", param, config);
    return restResult(response)
}
export async function smsEditNeteaseConfig(param, config) {
    const { id, name, access_key, access_secret } = param;
    var errors = {};
    if (parseInt(id) <= 0) {
        errors.name = "id未知";
    }
    if (typeof name !== "string" || name.length < 1) {
        errors.name = "应用名不能为空";
    }
    if (typeof access_key !== "string" || access_key.length < 12) {
        errors.access_key = " access key 错误";
    }
    if (typeof access_secret !== "string" || access_secret.length < 12) {
        errors.access_secret = " access secret 错误";
    }

    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("smser").post("/netease_config_edit", {
        ...param,
        id: parseInt(id),
    }, config);
    return restResult(response)
}
export async function smsDelNeteaseConfig(param, config) {
    const { id } = param;
    let response = await senderSettingRest("smser").post("/netease_config_del", {
        "id": parseInt(id),
    }, config);
    return restResult(response)
}
export async function smsAddAppNeteaseConfig(param, config) {
    const { app_id, user_id, config_id, name, tpl_id, template_map, template_id, } = param;
    let errors = {};
    let data = {
        name: name,
        config_id: parseInt(config_id),
        tpl_id: tpl_id,

        template_id: template_id,
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
    if (typeof template_id !== "string" || template_id.length <= 2) {
        errors.template_id = "模板ID错误";
    }

    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let map_data = (template_map + '').split(",");
    map_data = map_data.filter((e) => e)
    map_data = map_data.join(",")
    map_data = map_data.split("，")
    map_data = map_data.filter((e) => e)
    data.template_map = map_data.join(",")

    let response = await senderSettingRest("smser").post("/netease_app_config_add", data, config);
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


export async function mailListTplConfig(param, config) {
    const { id, app_id, tpl, user_id, page, page_size, app_info } = param;
    let data = {
        count_num: true,
        app_info: app_info ? true : false,
        page: {
            page: parseInt(page) >= 0 ? (parseInt(page) + 1) : 1,
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 25
        }
    };
    let errors = {};
    if (parseInt(user_id) >= 0) {
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
    let response = await senderSettingRest("mailer").post("/tpl_config_list", data, config);
    return restResult(response, ['not_found'])
}


export async function mailAppSend(param, config) {
    const { tpl_id, data, to, reply, send_time, max_try } = param;
    let errors = {};
    if (parseInt(tpl_id) <= 0) {
        errors.tpl_id = "模板未选择"
    }
    if (typeof reply === "string" && reply.length > 0) {
        if (!isEmail(reply)) {
            errors.reply = "回复邮箱格式错误"
        }
    }
    if (!to || to.length <= 0) {
        errors.to = "请输入接收邮箱"
    }
    for (var tmp of to) {
        if (!isEmail(tmp)) {
            errors.to = "接收邮箱地址错误:" + tmp
            break;
        }
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let sp = {
        tpl_id: parseInt(tpl_id),
        data: data,
        to: to,
        reply: reply,
        send_time: send_time,
    };
    if (max_try > 0) sp.max_try = max_try;
    let response = await senderSettingRest("mailer").post("/message_send", sp, config);
    return restResult(response)
}


export async function mailAddAppSmtpConfig(param, config) {
    const { app_id, user_id, smtp_config_id, name, tpl_id, from_email, reply_email, body_tpl_id, subject_tpl_id } = param;
    let errors = {};

    let data = {
        name: name,
        smtp_config_id: parseInt(smtp_config_id),
        tpl_id: tpl_id,
        from_email: from_email,
        reply_email: reply_email,
        body_tpl_id: body_tpl_id,
        subject_tpl_id: subject_tpl_id,

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
    if (typeof body_tpl_id !== "string" || body_tpl_id.length < 1) {
        errors.body_tpl_id = "请选择内容模板";
    }
    if (typeof from_email !== "string" || !isEmail(from_email)) {
        errors.from_email = "来源邮箱不能为空";
    }
    if (typeof reply_email === "string" && reply_email.length > 0 && !isEmail(reply_email)) {
        errors.reply_email = "回复邮箱格式错误";
    }
    if (typeof subject_tpl_id !== "string" || subject_tpl_id.length < 1) {
        errors.subject_tpl_id = "请选择标题模板";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("mailer").post("/smtp_app_config_add", data, config);
    return restResult(response)
}

export async function mailDelAppTplConfig(param, config) {
    const { id } = param;
    let response = await senderSettingRest("mailer").post("/tpl_config_del", {
        "app_config_id": parseInt(id),
    }, config);
    return restResult(response)
}


export async function tplsAddConfig(param, config) {
    const { tpl_id, tpl_data, sender_type, user_id } = param;
    let set_id = tpl_id.replace(/\s*$/, '').replace(/^\s*/, '');
    let set_data = tpl_data.replace(/\s*$/, '').replace(/^\s*/, '');
    let errors = {};
    if (typeof set_id !== "string" || set_id.length < 1) {
        errors.tpl_id = "请输入模板内容";
    }

    if (typeof set_data !== "string" || set_data.length < 1) {
        errors.tpl_data = "请输入模板内容";
    }

    if (Object.keys(errors).length) {
        return fialResult(errors);
    }

    let data = {
        sender_type: parseInt(sender_type),
        tpl_id: set_id,
        tpl_data: set_data,
        user_id: parseInt(user_id),
    };
    let response = await senderSettingRest("tpls").post("/add", data, config);
    return restResult(response)
}

export async function tplsEditConfig(param, config) {
    const { id, tpl_data } = param;
    let errors = {};
    let set_data = tpl_data.replace(/\s*$/, '').replace(/^\s*/, '');
    if (typeof set_data !== "string" || set_data.length < 1) {
        errors.name = "请输入模板内容";
    }

    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await senderSettingRest("tpls").post("/edit", {
        id: parseInt(id),
        tpl_data: set_data,
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
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 25
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