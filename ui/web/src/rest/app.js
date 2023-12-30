import { fialResult, restResult, sessionRest } from "../utils/rest";
import { isDomain } from "../utils/utils";
function appRest() {
    return sessionRest('/api/app')
};


export async function appList(param, config) {
    const { app_id, status, user_id, client_id,
        check_view_app, check_sms, check_mail,
        page, page_size } = param;
    let params = {
        "count_num": true,
        "page": {
            page: parseInt(page) >= 0 ? (parseInt(page) + 1) : 1,
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 25
        }
    };
    if (parseInt(user_id) > 0) {
        params.user_id = parseInt(user_id);
    }
    if (status && status != '') {
        params.status = [parseInt(status)];
    }
    if (client_id && client_id != '') {
        params.client_ids = [client_id];
    }
    if (app_id && app_id > 0) {
        params.app_id = [parseInt(app_id)];
    }
    params.check_sms = !!check_sms;
    params.check_mail = !!check_mail;
    params.check_view_app = !!check_view_app;
    let response = await appRest().post("list", params, config);
    return restResult(response, ['not_found'])
}

export async function appAdd(param, config) {
    const { name, client_id, user_id, domain } = param;
    var errors = {};
    if (typeof name !== "string" || name.length < 1) {
        errors.name = "应用名不能为空";
    }
    if (typeof client_id !== "string" || client_id.length < 3) {
        errors.client_id = "appid 必须大于3个字符";
    }
    if (typeof domain == "string" && domain.length > 0) {
        if (!isDomain(domain, true)) {
            errors.domain = "回地域名[IP]格式错误,不需要请留空";
        }
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await appRest().post("add", {
        user_id: user_id,
        name: name,
        client_id: client_id,
        domain: domain
    }, config);
    return restResult(response)
}


export async function appEdit(param, config) {
    const { appid, name, client_id, domain } = param;
    var errors = {};
    if (typeof name !== "string" || name.length < 1) {
        errors.name = "应用名不能为空";
    }
    if (typeof client_id !== "string" || client_id.length < 3) {
        errors.client_id = "appid 必须大于3个字符";
    }
    if (typeof domain == "string" && domain.length > 0) {
        if (!/^[\d]{1,3}\.[\d]{1,3}\.[\d]{1,3}\.[\d]{1,3}(:[\d]{1,5})?$/.test(domain)
            && !/^[0-9a-zA-Z]{0,1}[0-9a-zA-Z-]*(\.[0-9a-zA-Z-]*)*(\.[0-9a-zA-Z]*)+(:[\d]{1,5})?$/.test(domain)) {
            errors.domain = "回地域名[IP]格式错误,不需要请留空";
        }
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await appRest().post("edit", {
        app_id: appid,
        name: name,
        client_id: client_id,
        domain: domain
    }, config);
    return restResult(response)
}

export async function statusApp(param, config) {
    const { appid,status} = param;
    let response = await appRest().post("status", {
        "app_id": appid,
        "status":status
    }, config);
    return restResult(response)
}


export async function confirmApp(param, config) {
    const { appid } = param;
    let response = await appRest().post("confirm", {
        "app_id": appid,
    }, config);
    return restResult(response)
}


export async function viewSecretApp(param, config) {
    const { appid } = param;
    let response = await appRest().post("view_secret", {
        "app_id": appid,
    }, config);
    return restResult(response)
}

export async function resetSecretApp(param, config) {
    const { appid } = param;
    let response = await appRest().post("reset_secret", {
        "app_id": appid,
    }, config);
    return restResult(response)
}

export async function setSubUser(param, config) {
    const { appid,user_id,used } = param;
    let response = await appRest().post("set_sub_user", {
        "app_id": parseInt(appid),
        "user_id": parseInt(user_id),
        "used": !!used,
    }, config);
    return restResult(response)
}


export async function listSubUser(param, config) {
    const { appid,user_id, page, page_size } = param;
    let params={
        "app_id": parseInt(appid),
        "count_num":true,
        "page": {
            page: parseInt(page) >= 0 ? (parseInt(page) + 1) : 1,
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 25
        }
    };
    if (user_id>0){
        params.user_id=parseInt(user_id)
    }
    let response = await appRest().post("list_sub_user", params, config);
    return restResult(response)
}

export async function listSubApp(param, config) {
    const { appid,user_id, page, page_size } = param;
    let params= {
        "app_id": parseInt(appid),
        "count_num":true,
        "page": {
            page: parseInt(page) >= 0 ? (parseInt(page) + 1) : 1,
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 25
        }
    }
    if(user_id>0){
        params.user_id=parseInt(user_id)
    }
    let response = await appRest().post("list_sub_app",params, config);
    return restResult(response)
}

export async function listParentApp(param, config) {
    const { appid, page, page_size } = param;
    let response = await appRest().post("list_parent_app", {
        "app_id": parseInt(appid),
        "count_num":true,
        "page": {
            page: parseInt(page) >= 0 ? (parseInt(page) + 1) : 1,
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 25
        }
    }, config);
    return restResult(response)
}

export async function setParentApp(param, config) {
    const { appid ,parent_appid,sub_secret} = param;
    let response = await appRest().post("set_parent_app", {
        "app_id": parseInt(appid),
        "parent_app_id": parseInt(parent_appid),
        "sub_secret":sub_secret
    }, config);
    return restResult(response)
}
export async function delParentApp(param, config) {
    const { appid ,parent_appid} = param;
    let response = await appRest().post("del_parent_app", {
        "app_id": parseInt(appid),
        "parent_app_id": parseInt(parent_appid),
    }, config);
    return restResult(response)
}
