// setting config

import { globalRest, restResult, sessionRest } from "../utils/rest";

function settingRest() {
    return sessionRest('api/setting')
};

export async function loadLoginConfigGet(type, config) {
    let response = await settingRest().post(`/oauth/${type}`, {}, config);
    return restResult(response)
}
export async function loadLoginConfigSet(type, param, config) {
    let response = await settingRest().post(`/oauth/${type}`, param, config);
    return restResult(response)
}


export async function loadSiteConfigGet(type, config) {
    let response = await settingRest().post(`/system/${type}`, {}, config);
    return restResult(response)
}

export async function loadSiteConfigSet(type, param, config) {
    let response = await settingRest().post(`/system/${type}`, param, config);
    return restResult(response)
}


export async function loadSiteConfigInfo(config) {
    let response = await globalRest("api/site").get(`/info`, {}, config);
    return restResult(response)
}
