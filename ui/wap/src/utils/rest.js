import axios from "axios";
import config from '../../config.json';
const timeout = 10000;

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
        baseURL: config.serverURL + '/api/user/',
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


export function fialResult(field, message) {
    return {
        status: false,
        field: field,
        message: message ?? null
    }
};

export function restResult(res) {
    if (res?.data?.result?.code != 200) {
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