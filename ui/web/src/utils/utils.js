import { userSessionClear } from "./rest";

export function showTime(unixTime, defText) {
    if (!unixTime || unixTime <= 0 || unixTime == '') {
        return defText + '';
    }
    var date = new Date(unixTime * 1000);
    var datetime = "";
    datetime += date.getFullYear() + "-";
    datetime += (date.getMonth() + 1 < 10 ? "0" + (date.getMonth() + 1) : date.getMonth() + 1) + "-";
    datetime += (date.getDate() < 10 ? "0" + (date.getDate()) : date.getDate()) + " ";
    datetime += " " + (date.getHours() < 10 ? "0" + (date.getHours()) : date.getHours()) + ":";
    datetime += (date.getMinutes() < 10 ? "0" + (date.getMinutes()) : date.getMinutes()) + ":";
    datetime += (date.getSeconds() < 10 ? "0" + (date.getSeconds()) : date.getSeconds());
    return datetime;
}


export function redirectLoginPage() {
    userSessionClear()
    let url = window.location.href.replace(/#\/.*$/, "");
    url += "#/login/name?redirect_uri=" + encodeURIComponent(window.location.href);
    window.location.href = url
}


export function isDomain(domain, allow_ip = true) {
    if (allow_ip && /^[\d]{1,3}\.[\d]{1,3}\.[\d]{1,3}\.[\d]{1,3}(:[\d]{1,5})?$/.test(domain)) {
        return true
    }
    if (/^[0-9a-zA-Z]{0,1}[0-9a-zA-Z-]*(\.[0-9a-zA-Z-]*)*(\.[0-9a-zA-Z]*)+(:[\d]{1,5})?$/.test(domain)) {
        return true
    }
    return false
}