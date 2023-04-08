import { restResult, sessionRest } from "../utils/rest";

function accessRest() {
  return sessionRest('/api/user')
};


export async function resAll(param, config) {
  let response = await accessRest().post("/res/all", param, config);
  return restResult(response, ['not_found'])
}


export async function resAdd(params, config) {
  let {
    user_id,
    key,
    name,
    ops,
    tags,
  } = params;
  tags = tags.map((e) => {
    return e
      .replace(/^\s+/)
      .replace(/\s+$/)
  }).filter((e) => {
    return e.length > 0
  })
  ops = ops.map((e) => {
    if (!e.key || e.key.length <= 0 || !e.name || e.name.length <= 0) return;
    return e;
  }).filter((e) => { return e })
  let param = {
    user_id: parseInt(user_id),
    name: name,
    key: key,
    ops: ops,
    tags: tags,
  };
  let response = await accessRest().post("/res/add", param, config);
  return restResult(response)
}

export async function resEdit(params, config) {
  let { res_id, name, ops, tags } = params;
  tags = tags.map((e) => {
    return e
      .replace(/^\s+/)
      .replace(/\s+$/)
  }).filter((e) => {
    return e.length > 0
  })
  ops = ops.map((e) => {
    if (!e.key || e.key.length <= 0 || !e.name || e.name.length <= 0) return;
    return e;
  }).filter((e) => { return e })
  let param = {
    res_id: parseInt(res_id),
    name: name,
    ops: ops,
    tags: tags,
  };
  let response = await accessRest().post("/res/edit", param, config);
  return restResult(response)
}

export async function resListData(params, config) {
  let {
    user_id, tag, res_id, res_name, page, page_size
  } = params;
  let param = {
    user_id: parseInt(user_id),
    tags: params.tags ? true : false,
    ops: params.ops ? true : false,
    count_num: params.count_num ? true : false,
    page: {
      page: parseInt(page) >= 0 ? (parseInt(page) + 1) : 1,
      limit: parseInt(page_size) > 0 ? parseInt(page_size) : 10
    }
  };
  if (typeof tag == 'string' && tag.length > 0) {
    param.tags_filter = [tag];
  }
  if (typeof res_name == 'string' && res_name.length > 0) {
    param.res_name = res_name;
  }
  if (typeof res_id == 'string') {
    res_id = res_id.split(",").map((e) => parseInt(e));
    res_id = res_id.filter((e) => !isNaN(e))
    if (res_id.length > 0) {
      param.res_id = res_id;
    }
  }
  if (typeof res_id == 'number') {
    param.res_id = [res_id];
  }
  let response = await accessRest().post("/res/list_data", param, config);
  return restResult(response, ['not_found'])
}

export async function resTags(param, config) {
  let response = await accessRest().post("/res/tags", param, config);
  return restResult(response, ['not_found'])
}


export async function resDelete(param, config) {
  let response = await accessRest().post("/res/delete", param, config);
  return restResult(response)
}

export async function roleAdd(params, config) {
  let { user_id,
    name,
    user_range,
    role_op_range,
    priority,
    relation_key,
    tags,
    role_ops } = params;
  user_id = parseInt(user_id)
  user_range = parseInt(user_range)
  role_op_range = parseInt(role_op_range)
  priority = parseInt(priority)
  tags = tags.map((e) => {
    return e
      .replace(/^\s+/)
      .replace(/\s+$/)
  }).filter((e) => {
    return e.length > 0
  })
  role_ops = role_ops.map((e) => {
    let op_id = parseInt(e.op_id);
    let op_positivity = parseInt(e.op_positivity);
    if (isNaN(op_id) || isNaN(op_positivity)) return;
    return {
      op_id: op_id,
      op_positivity: op_positivity
    }
  }).filter((e) => { return e })
  let param = {
    user_id: user_id,
    name: name,
    user_range: user_range,
    role_op_range: role_op_range,
    priority: priority,
    relation_key: relation_key + '',
    tags: tags,
    role_user: [],
  };
  if (role_op_range == 1) {
    param.role_ops = role_ops;
  }
  let response = await accessRest().post("/role/add", param, config);
  return restResult(response)
}



export async function roleEdit(params, config) {
  let {
    role_id,
    name,
    user_range,
    role_op_range,
    priority,
    relation_key,
    tags,
    role_ops
  } = params;

  role_id = parseInt(role_id)
  user_range = parseInt(user_range)
  role_op_range = parseInt(role_op_range)
  priority = parseInt(priority)
  tags = tags.map((e) => {
    return e
      .replace(/^\s+/)
      .replace(/\s+$/)
  }).filter((e) => {
    return e.length > 0
  })
  role_ops = role_ops.map((e) => {
    let op_id = parseInt(e.op_id);
    let op_positivity = parseInt(e.op_positivity);
    if (isNaN(op_id) || isNaN(op_positivity)) return;
    return {
      op_id: op_id,
      op_positivity: op_positivity
    }
  }).filter((e) => { return e })
  let param = {
    role_id: role_id,
    name: name,
    user_range: user_range,
    role_op_range: role_op_range,
    priority: priority,
    relation_key: relation_key,
    tags: tags,
  };
  if (role_op_range == 1) {
    param.role_ops = role_ops;
  }


  let response = await accessRest().post("/role/edit", param, config);
  return restResult(response)
}


export async function roleListUser(params, config) {
  let { op_user_id, page, page_size, role_id } = params;
  op_user_id = parseInt(op_user_id)
  let param = {
    role_id: [parseInt(role_id)],
    count_num: true,
    page: {
      page: parseInt(page) >= 0 ? (parseInt(page) + 1) : 1,
      limit: parseInt(page_size) > 0 ? parseInt(page_size) : 10
    },
  };
  if (op_user_id > 0) {
    param.user_id = [op_user_id]
  }
  let response = await accessRest().post("/role/list_user", param, config);
  return restResult(response, ['not_found'])
}

export async function roleAddUser(params, config) {
  let { roleId, op_user_id, timeout } = params;
  op_user_id = parseInt(op_user_id)
  if (isNaN(op_user_id) || op_user_id < 0) return;
  let user_param = {
    role_id: roleId,
    user_vec: [
      {
        "user_id": op_user_id,
        "timeout": timeout
      }
    ]
  };
  let response = await accessRest().post("/role/add_user", user_param, config);
  return restResult(response)
}



export async function roleDeleteUser(params, config) {
  let { op_user_id, roleId } = params;
  op_user_id = parseInt(op_user_id)
  if (isNaN(op_user_id) || op_user_id < 0) return;
  let user_param = {
    role_id: roleId,
    user_vec: [
      op_user_id
    ]
  };
  let response = await accessRest().post("/role/delete_user", user_param, config);
  return restResult(response)
}


//todo 待使用
export async function roleRelationData(params, config) {
  let {
    user_id,
    relation_prefix,
    page,
    page_size
  } = params;
  let param = {
    count_num: true,
    user_id: parseInt(user_id),
    page: {
      page: parseInt(page) >= 0 ? (parseInt(page) + 1) : 1,
      limit: parseInt(page_size) > 0 ? parseInt(page_size) : 10
    },
  };
  if (typeof relation_prefix == 'string' && relation_prefix.length > 0) {
    param.relation_prefix = relation_prefix;
  }
  let response = await accessRest().post("/role/relation", param, config);
  return restResult(response, ['not_found'])
}
export async function roleListData(params, config) {
  let {
    user_id,
    user_range,
    res_range,
    relation_prefix,
    tag,
    role_id,
    role_name,
    page,
    page_size
  } = params;
  let param = {
    count_num: true,
    user_id: parseInt(user_id),
    tags: true,
    user_data: false,
    ops: 2,
    page: {
      page: parseInt(page) >= 0 ? (parseInt(page) + 1) : 1,
      limit: parseInt(page_size) > 0 ? parseInt(page_size) : 10
    },
    user_data_group: 2,
    user_data_page: { page: 0, limit: 0 }
  };
  user_range = parseInt(user_range)
  if (user_range > 0) {
    param.user_range = [user_range];
  }
  res_range = parseInt(res_range)
  if (res_range > 0) {
    param.res_range = [res_range];
  }
  if (typeof tag == 'string' && tag.length > 0) {
    param.tags_filter = [tag];
  }
  if (typeof role_name == 'string' && role_name.length > 0) {
    param.role_name = role_name;
  }
  if (typeof relation_prefix == 'string' && relation_prefix.length > 0) {
    param.relation_prefix = relation_prefix;
  }
  
  if (typeof role_id == 'string') {
    role_id = role_id.split(",").map((e) => parseInt(e));
    role_id = role_id.filter((e) => !isNaN(e))
    if (role_id.length > 0) {
      param.role_id = role_id;
    }
  }
  let response = await accessRest().post("/role/list_data", param, config);
  return restResult(response, ['not_found'])
}


export async function roleTags(param, config) {
  let response = await accessRest().post("/role/tags", param, config);
  return restResult(response, ['not_found'])
}


export async function roleOptions(params, config) {
  let {
    user_id
  } = params;
  //@todo 关系应该不能全部已知,所以应该是搜索得到部分在查询
  // 关系应该要分组
  const user_access_keys = [
    {
      key: "vip1",
      name: "等级1",
      is_use: false,
      time: null
    },
    {
      key: "vip2",
      name: "等级2",
      is_use: false,
      time: null
    },
  ]
  let param = {
    user_id: user_id,
    user_range: true,
    res_range: true,
  };
  param.relation_key = user_access_keys.map((e) => {
    return e.key
  });
  let response = await accessRest().post("/role/options", param, config);
  let data= restResult(response, ['not_found'])
    if (!data.status) return data;
    return {
      ...data,
      user_access_keys: user_access_keys.map((item) => {
        if (!data.exist_relation_role) {
          return item;
        } else {
          let out = { ...item };
          let sout = data.exist_relation_role.find((sitem) => {
            return sitem.relation_key == item.key
          });
          if (sout) {
            out.is_use = true;
            out.time = sout.change_time;
          }
          return out;
        }
      })
    }
}

export async function roleDelete(params, config) {
  let {
    role_id
  } = params;
  role_id = parseInt(role_id)
  if (isNaN(role_id) || role_id <= 0) return fialResult({ id: "缺少ID" });
  let param = {
    role_id: role_id
  };
  let response = await accessRest().post("/role/delete", param, config);
  return restResult(response)
}


export async function accessMenu(menus, config) {
  let out = [];
  let param = [];
  menus.map((item) => {
    if (!item.rbac || item.rbac.length == 0) {
      delete item.rbac;
      out.push(item);
      return;
    }
    item.rbac.map((e) => {
      if (!e.data) e.data = null;
      if (!param.find((t) => {
        return t.name == e.name && t.data == e.data;
      })) {
        param.push(e)
      }
    })
  })
  if (param.length == 0) {
    return Promise.resolve(out)
  }
  let response = await accessRest().post("/access/menu", {
    check_res: param
  }, config);
  let data = restResult(response, ['not_found'])
  if (data.status && data.data && data.data.length > 0) {

    menus.map((item) => {
      if (!item.rbac || item.rbac.length == 0) {
        return;
      }
      if (item.rbac.find((e) => {
        let find = false;
        if (data.data.find((t) => {
          return t.name == e.name && t.status
        })) {
          find = true;
        }
        return find
      })) {
        out.push(item);
      }
    })
  }
  return out;
}

