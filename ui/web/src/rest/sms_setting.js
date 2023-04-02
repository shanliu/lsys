import { fialResult, restResult,sessionRest } from "../utils/rest";

export const LimitStatusMap = [
    {   
        key: 1, 
        val: '发送频率限制',
        show:(d)=>{
           let data= JSON.parse(d);
           if(!data)return "数据解析失败";
           return `${data.range_time} 秒内最多发送 ${data.max_send} 条信息`;
        } 
    },
    {   
        key: 2, 
        val: '屏蔽指定号码',
        show:(data)=>{
            return "号码["+data+"]已被屏蔽";
        } 
    },
    {   
        key: 3, 
        val: '关闭发送功能',
        show:(data)=>{
            return "已禁止发送短信";
        } 
    },
    {   
        key: 4, 
        val: '指定模板忽略限制',
        show:(data)=>{
            return "短信模板["+data+"]不限制发送频率";
        } 
    },
    {   
        key: 5, 
        val: '批量发送量限制',
        show:(data)=>{
            return "单次最大发送量:"+data+"条";
        } 
    }
];
function smsSettingRest() {
    return sessionRest('/api/sender/smser')
};
export async function addSmsConfig(param, config) {
    const {app_id,priority,user_id, config_type,config_data} = param;
    let data={};
    var errors = {};
    if (app_id <0) {
        errors.app_id = "未提供app";
    }
    if (user_id>=0){
        data.user_id=parseInt(user_id);
    }
    data.app_id=parseInt(app_id);
    data.priority=parseInt(priority);
    if (config_type<=0){
        errors.config_type = "类型未知";
    }
    data.config_type=parseInt(config_type);
    switch (data.config_type){
        case 1:
            let range_time=parseInt( config_data.range_time);
            let max_send=parseInt( config_data.max_send);
            if (range_time<=0||max_send<=0){
                errors.config_data = "请设置限制数量跟时间";
            }else{
                data.config_data={
                    range_time,
                    max_send
                };
            }
            break;
        case 2:
            let mobile=config_data+'';
            if (/^\s*$/.test(mobile)){
                errors.config_data = "屏蔽号码不能为空";
            }else{
                data.config_data=mobile;
            }
            break;
        case 3:
            data.config_data='';
            break;
        case 4:
            let tpl=config_data+'';
            if (/^\s*$/.test(tpl)){
                errors.config_data = "模板不能为空";
            }else{
                data.config_data=tpl;
            }
            break;
        case 5:
            let num=parseInt(config_data);
            if (num<=0){
                errors.config_data = "数量必须大于0";
            }else{
                data.config_data=num;
            }
            break;
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await smsSettingRest().post("config_add", data, config);
    return restResult(response)
}
export async function listSmsConfig(param, config) {
    const {id,app_id,user_id ,page,page_size} = param;
    let data={
        "page": {
            "page": parseInt(page) + 1,
            "limit": parseInt(page_size)
        }
    };
    if (id>0){
        data.id=parseInt(id);
    }
    if (app_id>=0){
        data.app_id=parseInt(app_id);
    }
    if (user_id>0){
        data.user_id=parseInt(user_id);
    }
    let response = await smsSettingRest().post("config_list", data, config);
    return restResult(response,['not_found'])
}
export async function delSmsConfig(param, config) {
    const {config_id} = param;
    let data={};
    if (config_id>0){
        data.config_id=parseInt(config_id);
    }
    let response = await smsSettingRest().post("config_del", data, config);
    return restResult(response)
}
export async function listAliConfig(param, config) {
    const { id, full_data} = param;
    let data={};
    if (id>0){
        data.ids=[parseInt(id)];
    }
    if ( typeof full_data =='boolean'){
        data.full_data=full_data;
    }
    let response = await smsSettingRest().post("ali_config_list", data, config);
    return restResult(response,['not_found'])
}
export async function addAliConfig(param, config) {
    const { name, access_id, access_secret } = param;
    var errors = {};
    if (typeof name !== "string" || name.length < 1) {
        errors.name = "应用名不能为空";
    }
    if (typeof access_id !== "string" || access_id.length < 12) {
        errors.access_id = "access id 错误";
    }
    if (typeof access_secret !== "string" || access_secret.length < 12) {
        errors.access_secret = "access secret 错误";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await smsSettingRest().post("/ali_config_add", param, config);
    return restResult(response)
}
export async function editAliConfig(param, config) {
    const { id,name, access_id, access_secret } = param;
    var errors = {};
    if (parseInt(id)<=0) {
        errors.name = "id未知";
    }
    if (typeof name !== "string" || name.length < 1) {
        errors.name = "应用名不能为空";
    }
    if (typeof access_id !== "string" || access_id.length < 12) {
        errors.access_id = "access id 错误";
    }
    if (typeof access_secret !== "string" || access_secret.length < 12) {
        errors.access_secret = "access secret 错误";
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await smsSettingRest().post("/ali_config_edit", {
        ...param,
        id:parseInt(id),
    }, config);
    return restResult(response)
}
export async function delAliConfig(param, config) {
    const { id} = param;
    let response = await smsSettingRest().post("/ali_config_del", {
        "id":parseInt(id),
    }, config);
    return restResult(response)
}


export async function listAppAliConfig(param, config) {
    const { id,app_id,tpl,user_id,page,page_size} = param;
    let data={
        "page": {
            "page": parseInt(page) + 1,
            "limit": parseInt(page_size)
        }
    };
    let errors={};
    if (parseInt(user_id)>0) {
        data.user_id=parseInt(user_id);
    }
    if (parseInt(app_id)>=0) {
        data.app_id=parseInt(app_id);
    }
    if (parseInt(id)>0) {
        data.id=parseInt(id);
    }
    if (typeof tpl !== "string" || tpl.length > 1) {
        data.tpl=tpl;
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await smsSettingRest().post("/ali_app_config_list", data, config);
    return restResult(response,['not_found'])
}


export async function addAppAliConfig(param, config) {
    const { app_id,user_id,aliconfig_id,name,sms_tpl,aliyun_sms_tpl,aliyun_sign_name,max_try_num} = param;
    let errors={};
    let data={
        name:name,
        ali_config_id:parseInt(aliconfig_id),
        sms_tpl:sms_tpl,
        aliyun_sms_tpl: aliyun_sms_tpl,
        aliyun_sign_name: aliyun_sign_name,
        try_num: max_try>0?max_try:1
    };
    if (user_id>=0){
        data.user_id=parseInt(user_id);
    }
    if (app_id<0){
        errors.app_id = "请提供操作应用";
    }else{
        data.app_id=parseInt(app_id);
    }
    if (typeof name !== "string" || name.length <1) {
        errors.name = "名称必须大于等于2个字符";
    }
    if (typeof sms_tpl !== "string" || sms_tpl.length <=2) {
        errors.sms_tpl = "模板名必须大于等于3个字符";
    }
    if (typeof aliyun_sms_tpl !== "string" || aliyun_sms_tpl.length <=2) {
        errors.aliyun_sms_tpl = "阿里云模板错误";
    }
    if (typeof aliyun_sign_name !== "string" || aliyun_sign_name.length <2) {
        errors.aliyun_sign_name = "阿里云签名错误";
    }
    let max_try=parseInt(max_try_num);
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await smsSettingRest().post("/ali_app_config_add",data , config);
    return restResult(response)
}

export async function delAppAliConfig(param, config) {
    const { id} = param;
    let response = await smsSettingRest().post("/ali_app_config_del", {
        "app_config_id":parseInt(id),
    }, config);
    return restResult(response)
}




export const MessageStatus = [
    {   
        key: 1, 
        val: '待发送'
    },
    {   
        key: 2, 
        val: '发送完成'
    },
    {   
        key: 3, 
        val: '发送失败'
    },
    {   
        key: 4, 
        val: '已取消'
    }
];

export const MessageLogType = [
   
    {   
        key: 1, 
        val: '发送操作'
    },
    {   
        key: 2, 
        val: '取消发送'
    }
];
export const MessageLogStatus = [
   
    {   
        key: 2, 
        val: '成功'
    },
    {   
        key: 3, 
        val: '失败'
    },
    {   
        key: 4, 
        val: '接口取消'
    },
    {   
        key: 5, 
        val: '后台取消'
    }
];



export async function listAppMessage(param, config) {
    const { app_id,user_id,tpl_id,mobile,status,page,page_size} = param;
    let data={
        count_num:true,
        page: {
            page: parseInt(page) + 1,
            limit: parseInt(page_size)
        }
    };
    let errors={};
    if (parseInt(user_id)>0) {
        data.user_id=parseInt(user_id);
    }
    if (parseInt(status)>0) {
        data.status=parseInt(status);
    }
    if (parseInt(app_id)>=0) {
        data.app_id=parseInt(app_id);
    }
    if (typeof tpl_id == "string" && tpl_id.length >0) {
        data.tpl_id=tpl_id;
    }
    if (typeof mobile == "string" && mobile.length >0) {
        data.mobile=mobile;
    }
    if (Object.keys(errors).length) {
        return fialResult(errors);
    }
    let response = await smsSettingRest().post("/message_list", data, config);
    return restResult(response,['not_found'])
}


export async function listAppMessageHistory(param, config) {
    const { message_id,page,page_size} = param;
    let data={
        count_num:true,
        message_id:message_id.toString(),
        page: {
            page: parseInt(page) + 1,
            limit: parseInt(page_size)
        }
    };
    let response = await smsSettingRest().post("/message_history", data, config);
    return restResult(response,['not_found'])
}



export async function cancelAppMessage(param, config) {
    const { message_id} = param;
    if (message_id<=0){
        return fialResult({message_id:"缺少ID"});
    }
    let response = await smsSettingRest().post("/message_cancel", {
        message_id:message_id.toString(),
    }, config);
    return restResult(response)
}

export async function viewAppMessage(param, config) {
    const { message_id} = param;
    if (message_id<=0){
        return fialResult({message_id:"缺少ID"});
    }
    let response = await smsSettingRest().post("/message_body", {
        message_id:message_id.toString(),
    }, config);
    return restResult(response)
}

