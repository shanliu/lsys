import { restResult,sessionRest } from "../utils/rest";

function accessRest() {
    return sessionRest('/user')
};


export async function resAll(param, config) {
    let response = await accessRest().post("/res/all", param, config);
    return restResult(response)
}

export async function resAdd(param, config) {
    let response = await accessRest().post("/res/add", param, config);
    return restResult(response)
}

export async function resEdit(param, config) {
    let response = await accessRest().post("/res/edit", param, config);
    return restResult(response)
}

export async function resListData(param, config) {
    let response = await accessRest().post("/res/list_data", param, config);
    return restResult(response)
}

export async function resTags(param, config) {
    let response = await accessRest().post("/res/tags", param, config);
    return restResult(response)
}


export async function resDelete(param, config) {
    let response = await accessRest().post("/res/delete", param, config);
    return restResult(response)
}

export async function roleAdd(param, config) {
    let response = await accessRest().post("/role/add", param, config);
    return restResult(response)
}



export async function roleEdit(param, config) {
    let response = await accessRest().post("/role/edit", param, config);
    return restResult(response)
}


export async function roleListUser(param, config) {
    let response = await accessRest().post("/role/list_user", param, config);
    return restResult(response)
}

export async function roleAddUser(param, config) {
    let response = await accessRest().post("/role/add_user", param, config);
    return restResult(response)
}



export async function roleDeleteUser(param, config) {
    let response = await accessRest().post("/role/delete_user", param, config);
    return restResult(response)
}



export async function roleListData(param, config) {
    let response = await accessRest().post("/role/list_data", param, config);
    return restResult(response)
}


export async function roleTags(param, config) {
    let response = await accessRest().post("/role/tags", param, config);
    return restResult(response)
}


export async function roleOptions(param, config) {
    let response = await accessRest().post("/role/options", param, config);
    return restResult(response)
}

export async function roleDelete(param, config) {
    let response = await accessRest().post("/role/delete", param, config);
    return restResult(response)
}


export async function accessMenu(menus, config) {
    let out = [];
    let param = menus.map((item) => {
        if (!item.rbac || item.rbac.length == 0) {
            out.push(item);
            return;
        }
        let access_check = item.rbac.map((res_group) => {
            return res_group.map((res) => {
                res.user_id = 0;
                return res;
            })
        })
        return {
            name: item.text,
            access_check: access_check
        }
    }).filter((e) => { return e })
    if (param.length == 0) {
        return Promise.resolve(out)
    }
    let response = await accessRest().post("/access/menu", {
        check_res: param
    }, config);
    let data = restResult(response)
    if (data.status && data.data && data.data.length > 0) {
        data.data.filter((e) => e.status).map((e) => {
            menus.map((item) => {
                if (item.text == e.name) {
                    out.push(item);
                }
            })
        })
    }
    return out;
}


