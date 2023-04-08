
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

