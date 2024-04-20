
const crypto = require('crypto');
function getCurrentDateTime() {
    const now = new Date();
    const year = now.getFullYear();
    const month = (now.getMonth() + 1).toString().padStart(2, '0');
    const day = now.getDate().toString().padStart(2, '0');
    const hours = now.getHours().toString().padStart(2, '0');
    const minutes = now.getMinutes().toString().padStart(2, '0');
    const seconds = now.getSeconds().toString().padStart(2, '0');
    return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
}

function payloadSort(payload) {
    let param = {};
    for (let key of Object.keys(payload).sort()) {
        if (!Array.isArray(payload[key]) && typeof payload[key] == 'object') {
            param[key] = payloadSort(payload[key]);
        } else {
            param[key] = payload[key];
        }
    }
    return param
}




function addRequestSign(request, yac) {
    const query = new URLSearchParams(request.url.replace(/^.*\?/, ''));
    const timestamp = getCurrentDateTime();
    let payload = request.body;
    let contType = request.contentType?.contentType ?? "";
    for (key in ['content-type', 'Content-Type', 'Content-type', 'content-Type']) {
        if (request.headers[key] ?? null) {
            contType = request.headers[key];
            break;
        }
    }
    if (contType.indexOf("application/json") == -1) {
        payload = query.get('payload');
    }
    let body = "";
    if (payload && payload != '') {
        body = JSON.stringify(payloadSort(JSON.parse(payload)));
    }



    const secret = query.get('secret') ? query.get('secret') : yac.variables.APP_SECRET ?? '';
    const lang = query.get('lang') ? query.get('lang') : 'zh_CN';
    let params = {
        timestamp
    };
    let default_params = Object.assign({
        lang
    }, params);
    for (let opt of [{
        name: 'app_id',
        default: yac.variables.APP_KEY ?? '',
    }, {
        name: 'version',
        default: '2.0',
    }]) {
        if (query.get(opt.name)) {
            params[opt.name] = query.get(opt.name);
        } else {
            params[opt.name] = opt.default;
            default_params[opt.name] = opt.default;
        }
    }
    for (let opt of ['request_ip', 'token', 'method']) {
        if (query.get(opt) && query.get(opt) != '') {
            params[opt] = query.get(opt);
        }
    }
    let signData = [];
    for (key of Object.keys(params).sort()) {
        signData.push(`${key}=${encodeURIComponent(params[key]).replace(/%20/g, '+')}`);
    }
    const signStr = signData.join('&') + body + secret;
    console.log(signStr)
    const md5 = crypto.createHash('md5');
    md5.update(signStr);
    let url = []
    for (let tmp in default_params) {
        url.push(`${tmp}=${encodeURIComponent(default_params[tmp]).replace(/%20/g, '+')}`);
    }
    url.push(`sign=${md5.digest('hex')}`);
    if (request.url.indexOf('?') == -1) {
        request.url += '?' + url.join('&');
    } else {
        request.url += '&' + url.join('&');
    }
    if (request.url.indexOf("/") == 0 && (yac.variables.APP_HOST)) {
        request.url = yac.variables.APP_HOST + request.url
    }
}

module.exports = {
    configureHooks: function (api) {
        api.hooks.onRequest.addHook('addSign', addRequestSign);
    }
}