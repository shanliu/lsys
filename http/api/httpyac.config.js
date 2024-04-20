
const crypto = require('crypto');
function addRequestSign(request, yac) {
    // let jwt_header = '';
    // for (key of ['Authorization', 'authorization']) {
    //     if (request.headers[key] ?? null) {
    //         jwt_header = request.headers[key];
    //         break;
    //     }
    // }
    // console.log(yac);
    // if (/^Bearer\s+JWT\s*$/ig.test(jwt_header)) {

    // }
    if (request.url.indexOf("/") == 0 && (yac.variables.APP_HOST)) {
        request.url = yac.variables.APP_HOST + request.url
    }
}

module.exports = {
    configureHooks: function (api) {
        api.hooks.onRequest.addHook('addSign', addRequestSign);
    }
}