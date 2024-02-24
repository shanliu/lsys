import { globalRest, restResult } from "../utils/rest";

function areaRest() {
    return globalRest('api/area')
};



export async function areaList(params, config) {
    const { code } = params;
    let response = await areaRest().post(`/list`, {
        code: code + '',
    }, config);
    return restResult(response)
}
export async function areaSearch(params, config) {
    const { key_word } = params;
    let response = await areaRest().post(`/search`, {
        key_word: key_word + '',
    }, config);
    return restResult(response)
}
export async function areaDetail(params, config) {
    const { code } = params;
    let response = await areaRest().post(`/detail`, {
        code: code + '',
    }, config);
    return restResult(response)
}