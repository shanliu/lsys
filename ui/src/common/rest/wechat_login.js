import { globalRest, restResult } from "../utils/rest";


export async function wechatLogin(code, callback_state, config) {
    let response = await globalRest('api/user').post("/external_state_callback", {
        "code": code,
        "login_type": "wechat",
        "callback_state": callback_state
    }, config)
    return restResult(response)
}