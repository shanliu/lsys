// setting config

import { fialResult, globalRest, restResult, sessionRest } from "../utils/rest";

function docsRest() {
    return sessionRest('/api/docs')
};

export async function docsGitDetail(param, config) {
    const { url } = param;
    var errors = {};
    if (typeof url !== "string" ||(
        url.substr(0,7)!='http://'&&
        url.substr(0,8)!='https://'&&
        url.substr(0,6)!='git://'
    )) {
        errors.name = "URL请提供git地址";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await docsRest().post(`/setting/git_detail`, {
        url:url
    }, config);
    return restResult(response)
}
export async function docsAdd(param, config) {
    const { version } = param;
    var errors = {};
    if (typeof version !== "string" ||version.length!=40) {
        errors.name = "请提供正确的版本信息";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await docsRest().post(`/setting/add`, {}, config);
    return restResult(response)
}
export async function docsEdit(type, config) {
    let response = await docsRest().post(`/setting/edit`, {}, config);
    return restResult(response)
}
export async function docsList(type, config) {
    let response = await docsRest().post(`/setting/list`, {}, config);
    return restResult(response)
}
export async function docsLogs(type, config) {
    let response = await docsRest().post(`/setting/logs`, {}, config);
    return restResult(response)
}


export async function docsMenu(type, config) {
    let response = await docsRest().post(`/read/menu`, {}, config);
    return restResult(response)
}

export async function docsMdReads(type, config) {
    let response = await docsRest().post(`/read/md`, {}, config);
    return restResult(response)
}
