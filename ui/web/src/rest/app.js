import { fialResult, restResult,sessionRest } from "../utils/rest";
function appRest() {
    return sessionRest('/app')
};


export async function appList(param, config) {
    const { app_id,status, user_id, client_id, page, page_size } = param;
    let params = {
        "count_num": true,
        "page": {
            "page": parseInt(page) + 1,
            "limit": parseInt(page_size)
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
    if (app_id && app_id>0) {
        params.app_id = [parseInt(app_id)];
    }
    let response = await appRest().post("list", params, config);
    return restResult(response)
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
        if (!/^[\d]{1,3}\.[\d]{1,3}\.[\d]{1,3}\.[\d]{1,3}(:[\d]{1,5})?$/.test(domain)
            && !/^[0-9a-zA-Z]{0,1}[0-9a-zA-Z-]*(\.[0-9a-zA-Z-]*)*(\.[0-9a-zA-Z]*)+(:[\d]{1,5})?$/.test(domain)) {
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

export async function disableApp(param, config) {
    const { appid } = param;
    let response = await appRest().post("disable", {
        "app_id": appid,
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
