import axios from "axios";
import randomString from "random-string";
import store from 'store2';
import config from '../../config.json';
import JSONBig from 'json-bigint'
const timeout = 10000;



export function captchaSrc(type, rand = false) {
    let url = `${config.serverURL}/captcha/${type}`;
    if (rand) return url + "?" + randomString();
    else return url;
}



function dataHandle(data) {
    if (typeof data.data != "object") {
        data.data = {
            result: {
                code: 500,
                message: data.data,
                state: "server"
            },
        }
    } else {
        if (!data.data?.result?.code) {
            data.data.result.code = 500
        }
        if (!data.data.result.message) {
            data.data.result.message = "server output unkown"
        }
        if (data.data?.result?.code == 200 && !data.data?.response) {
            data.data.response = {}
        }
    }
    return data;
}


function errorHandler(error) {
    error.data = {
        result: {
            code: 500,
            message: '请求错误:' + error.message,
            state: "http"
        },
    }
    return Promise.resolve(error);
}

export function globalRest() {

    let ax = axios.create({
        baseURL: config.serverURL + config.restPath + '/user/',
        timeout: timeout,
        validateStatus: function (status) {
            return status >= 200 && status < 600;
        },
        headers: {
            'content-type': 'application/json'
        }
    })
    ax.interceptors.response.use(dataHandle, errorHandler);
    return ax
};


export function sessionRest(path) {
    let session = userSessionGet();
    if (!session) throw new Error("not login can't call user rest");
    let ax = axios.create({
        baseURL: config.serverURL + config.restPath + path,
        timeout: timeout,
        transformResponse: [
            (data)=>{
              try {
                return JSONBig.parse(data)
              } catch (err) {
                return data
              }
            }
        ],
        validateStatus: function (status) {
            return status >= 200 && status < 600;
        },
        headers: {
            'content-type': 'application/json',
            'Authorization': 'Bearer ' + session.jwt_header
        }
    })
    ax.interceptors.response.use(dataHandle, errorHandler);
    return ax
}

export function fialResult(field, message) {
    if (!message){
        message=Object.values(field).join(";")
    }
    return {
        status: false,
        field: field,
        message: message ?? null
    }
};

export function restResult(res) {
    if (res?.data?.result?.code != 200) {
        return {
            ...(res?.data?.result ?? {}),
            ...fialResult({}, res?.data?.result?.message ?? "未知错误"),
            data: res?.data?.response ?? null
        }
    }
    if (res.data.result.state != "ok") {
        if (res.data.result.state == "not_login") {
            userSessionClear()
        }
        return {
            ...res.data.result,
            ...fialResult(res.data.result.state == "valid_code" ? {
                captcha: res.data.result.message
            } : {}, res.data.result.message),
            is_captcha: res.data.result.state == "need_captcha",
            data: res.data.response
        }
    }
    return {
        status: true,
        ...res.data.response
    };
};

export function userSessionGet(session) {
    let data = store.get("session");
    if (!data) {
        data = store.session("session");
    }
    if (data?.user_data?.time_out) {
        if (data.user_data.time_out > Date.now() / 1000) {
            return data;
        }
    }
    userSessionClear();
    return null;
};
export function userSessionClear() {
    store.remove("session")
    store.session.remove("session")
};
export function userSessionSet(data) {
    if (data.session) {
        store.session({ session: data })
    } else {
        store.set("session", data);
    }
};
