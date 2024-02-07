import config from '../../config.json';
import { failResult, globalRest, restResult, sessionRest } from "../utils/rest";

function docsRest() {
    return sessionRest('/api/docs')
};



export async function docsGitAdd(params, config) {
    const { name, url, max_try } = params;
    var errors = {};
    if (max_try <= 0) {
        errors.max_try = "重试次数必须大于0";
    }
    if (typeof name != 'string' || name == 0) {
        errors.name = "请输入名称";
    }
    if (typeof url !== "string" || (
        url.substr(0, 7) != 'http://' &&
        url.substr(0, 8) != 'https://' &&
        url.substr(0, 6) != 'git://'
    )) {
        errors.url = "URL请提供git地址";
    }
    if (Object.keys(errors).length) {
        return failResult(errors);
    }
    let response = await docsRest().post(`/setting/git_add`, {
        name: name,
        url: url,
        max_try: parseInt(max_try)
    }, config);
    return restResult(response)
}
export async function docsGitEdit(params, config) {
    const { doc_id, name, url, max_try } = params;
    var errors = {};
    if (doc_id <= 0) {
        errors.branch = "ID异常";
    }
    var errors = {};
    if (max_try <= 0) {
        errors.max_try = "重试次数必须大于0";
    }
    if (typeof name != 'string' || name == 0) {
        errors.name = "请输入名称";
    }
    if (typeof url != 'string' || url == 0) {
        errors.url = "请提供URL";
    }
    if (Object.keys(errors).length) {
        return failResult(errors);
    }
    let response = await docsRest().post(`/setting/git_edit`, {
        id: parseInt(doc_id),
        name: name,
        url: url,
        max_try: parseInt(max_try)
    }, config);
    return restResult(response)

}
export async function docsGitList(config) {
    let response = await docsRest().post(`setting/git_list`, {}, config);
    return restResult(response)
}

export async function docsDelGit(params, config) {
    const { git_id } = params;
    var errors = {};
    if (git_id <= 0) {
        errors.branch = "ID异常";
    }
    if (Object.keys(errors).length) {
        return failResult(errors);
    }
    let response = await docsRest().post(`/setting/git_del`, {
        id: parseInt(git_id),
    }, config);
    return restResult(response)
}



export async function docsGitDetail(param, config) {
    const { url } = param;
    var errors = {};
    if (typeof url !== "string" || (
        url.substr(0, 7) != 'http://' &&
        url.substr(0, 8) != 'https://' &&
        url.substr(0, 6) != 'git://'
    )) {
        errors.name = "URL请提供git地址";
    }
    if (Object.keys(errors).length) {
        return failResult(errors);
    }
    let response = await docsRest().post(`/setting/git_detail`, {
        url: url
    }, config);
    return restResult(response)
}


export const docCloneStatus = [
    { key: 1, val: '待克隆' },
    { key: 2, val: '已克隆' },
    { key: 3, val: '克隆失败' },
];

export const docTagStatus = [
    { key: 1, val: '待启用' },
    { key: 2, val: '已启用' },
];

export async function docsTagAdd(param, config) {
    const { version, git_id, tag, clear_rule } = param;
    var errors = {};
    if (git_id <= 0) {
        errors.git_id = "GIT ID异常";
    }
    if (typeof tag !== "string" || tag.length <= 0) {
        errors.tag = "请提供正确的版本信息";
    }
    if (typeof version !== "string" || version.length != 40) {
        errors.version = "请提供正确的版本信息";
    }
    if (Object.keys(errors).length) {
        return failResult(errors);
    }
    let response = await docsRest().post(`/setting/tag_add`, {
        git_id: git_id,
        tag: tag,
        build_version: version,
        clear_rule: clear_rule && clear_rule.length ? clear_rule : [],
    }, config);
    return restResult(response)
}
export async function docsTagDel(param, config) {
    const { tag_id } = param;
    var errors = {};
    if (tag_id <= 0) {
        errors.tag_id = "ID异常";
    }
    if (Object.keys(errors).length) {
        return failResult(errors);
    }
    let response = await docsRest().post(`/setting/tag_del`, { tag_id: tag_id }, config);
    return restResult(response)
}
export async function docsTagList(params, config) {
    const { key_word, page, page_size, git_id, status } = params;
    let param = {
        count_num: true,
        "page": {
            page: parseInt(page) >= 0 ? (parseInt(page) + 1) : 1,
            limit: parseInt(page_size) > 0 ? parseInt(page_size) : 25
        }
    };
    if (typeof key_word == 'string' && key_word.length > 0) {
        param.key_word = key_word
    }
    if (git_id > 0) {
        param.git_id = parseInt(git_id)
    }
    if (status > 0) {
        param.status = parseInt(status)
    }
    let response = await docsRest().post(`/setting/tag_list`, param, config);
    return restResult(response)
}
export async function docsTagCloneDel(param, config) {
    const { clone_id } = param;
    var errors = {};
    if (clone_id <= 0) {
        errors.clone_id = "ID异常";
    }
    if (Object.keys(errors).length) {
        return failResult(errors);
    }
    let response = await docsRest().post(`/setting/tag_clone_del`, {
        clone_id: parseInt(clone_id)
    }, config);
    return restResult(response)
}

export async function docsTagStatusSet(param, config) {
    const { tag_id, status } = param;
    var errors = {};
    if (tag_id <= 0) {
        errors.tag_id = "ID异常";
    }
    if (Object.keys(errors).length) {
        return failResult(errors);
    }
    let response = await docsRest().post(`/setting/tag_status`, {
        tag_id: parseInt(tag_id),
        status: parseInt(status)
    }, config);
    return restResult(response)
}

export async function docsTagDir(params, config) {
    const { tag_id, prefix } = params;
    var errors = {};
    if (tag_id <= 0) {
        errors.tag_id = "ID异常";
    }
    if (Object.keys(errors).length) {
        return failResult(errors);
    }
    let response = await docsRest().post(`/setting/tag_dir`, {
        tag_id: parseInt(tag_id),
        prefix: prefix ? prefix + '' : ''
    }, config);
    return restResult(response)
}
export async function docsTagLogs(params, config) {
    const { tag_id } = params;
    var errors = {};
    if (tag_id <= 0) {
        errors.tag_id = "ID异常";
    }
    if (Object.keys(errors).length) {
        return failResult(errors);
    }
    let response = await docsRest().post(`/setting/tag_logs`, {
        tag_id: parseInt(tag_id)
    }, config);
    return restResult(response)
}
export async function docsTagFileData(params, config) {
    const { tag_id, file_path } = params;
    var errors = {};
    if (tag_id <= 0) {
        errors.tag_id = "ID异常";
    }
    if (typeof file_path != 'string' || file_path <= 0) {
        errors.file_path = "请先提供查看路径";
    }
    if (Object.keys(errors).length) {
        return failResult(errors);
    }
    let response = await docsRest().post(`/setting/tag_file_data`, {
        tag_id: parseInt(tag_id),
        file_path: file_path + '',
    }, config);
    return restResult(response)
}
export async function docsMenuList(params, config) {
    const { tag_id } = params;
    var errors = {};
    if (tag_id <= 0) {
        errors.tag_id = "ID异常";
    }
    if (Object.keys(errors).length) {
        return failResult(errors);
    }
    let response = await docsRest().post(`/setting/menu_list`, {
        tag_id: parseInt(tag_id),
    }, config);
    return restResult(response)
}
export async function docsMenuAdd(params, config) {
    const { tag_id, menu_path } = params;
    var errors = {};
    if (tag_id <= 0) {
        errors.tag_id = "ID异常";
    }
    if (typeof menu_path != 'string' || menu_path <= 0) {
        errors.menu_path = "请输入目录路径";
    }
    if (Object.keys(errors).length) {
        return failResult(errors);
    }
    let response = await docsRest().post(`/setting/menu_add`, {
        tag_id: parseInt(tag_id),
        menu_path: menu_path + '',
    }, config);
    return restResult(response)
}
export async function docsMenuDel(params, config) {
    const { menu_id } = params;
    var errors = {};
    if (menu_id <= 0) {
        errors.menu_id = "ID异常";
    }
    if (Object.keys(errors).length) {
        return failResult(errors);
    }
    let response = await docsRest().post(`/setting/menu_del`, {
        menu_id: parseInt(menu_id),
    }, config);
    return restResult(response)
}


export async function docsMenu(config) {
    let response = await globalRest('/api/docs').post(`/read/menu`, {}, config);
    return restResult(response)
}

export async function docsMdReads(params, config) {
    const { url, menu_id } = params;
    let response = await globalRest('/api/docs').post(`/read/md`, {
        url: url,
        menu_id: parseInt(menu_id)
    }, config);
    return restResult(response)
}
