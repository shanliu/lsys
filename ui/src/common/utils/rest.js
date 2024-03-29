import axios from "axios";
import randomString from "random-string";
import store from 'store2';
import config from 'siteConfig';
import JSONBig from 'json-bigint'
import { redirectLoginPage } from "./utils";
const timeout = 10000;



export function captchaSrc(type, rand = false) {
    let api_host = config.serverURL;
    if (!api_host) api_host = window.location.origin + '/';
    let url = `${api_host}captcha/${type}`;
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

export function globalRest(path) {
    let api_host = config.serverURL;
    if (!api_host) api_host = window.location.origin + '/';
    let ax = axios.create({
        baseURL: api_host + path,
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
    let api_host = config.serverURL;
    if (!api_host) api_host = window.location.origin + '/';
    let session = userSessionGet();
    if (!session) {
        redirectLoginPage()
        return axios.create({//防止报错。。。
            timeout: timeout,
            baseURL: api_host + path,
        })
    }
    let ax = axios.create({
        baseURL: api_host + path,
        timeout: timeout,
        transformResponse: [
            (data) => {
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

export function failResult(field, message) {
    if (!message) {
        message = Object.values(field).join(";")
    }
    return {
        status: false,
        field: field,
        message: message ?? null
    }
};

export function restResult(res, ignore) {
    if (typeof ignore == 'string') {
        ignore = [ignore]
    }
    if (!res?.data?.response) res.data.response = [];
    if (res?.data?.result?.code != 200) {

        if (ignore && ignore.length > 0 && ignore.includes(res?.data?.result?.state)) {
            return {
                status: true,
                ...res.data.response
            };
        }

        switch (res?.data?.result?.state) {
            case "not_login":
                userSessionClear()
            case "jwt_bad_token":
                userSessionClear()
                break
        }
        return {
            ...(res?.data?.result ?? {}),
            ...failResult(res?.data?.result?.state == "valid_code" ? {
                captcha: res.data.result.message
            } : {}, res?.data?.result?.message ?? "未知错误"),
            is_captcha: res.data.result.state == "need_captcha",
            data: res?.data?.response ?? null
        }
    }
    return {
        status: true,
        ...res.data.response
    };


    // return {
    //     ...res.data.result,
    //     ...failResult(res.data.result.state == "valid_code" ? {
    //         captcha: res.data.result.message
    //     } : {}, res.data.result.message),
    //     is_captcha: res.data.result.state == "need_captcha",
    //     data: res.data.response
    // }
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
