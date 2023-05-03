import { Box, List, ListItem, ListItemButton, ListItemIcon, ListItemText, MenuList } from '@mui/material';
import React, { Fragment } from 'react';
import ApiIcon from '@mui/icons-material/Api';
import KeyIcon from '@mui/icons-material/Key';
import SettingsIcon from '@mui/icons-material/Settings';
import { Breadcrumbs } from '@mui/material';
import { useParams } from 'react-router';
import ManageAccountsIcon from '@mui/icons-material/ManageAccounts';
import { Link } from '@mui/material';
import Typography from '@mui/material/Typography';
import { Link as RouteLink } from 'react-router-dom';

import MarkdownPreview from '@uiw/react-markdown-preview';

const source = '## 接口文档\n\n> 目前已提供 go 封装,可参考[示例](go/examples/)\n\n\n\n\n## 请求参数及加密秘钥生成说明(非OAUTH)\n\n> 当 Content-type 为 application/json 时 将尝试把请求内容[如:POST内容]做为JSON字符串并解析为接口参数\n\n> 当 Content-type 非 application/json 时 将尝试把GET参数中的 payload 做为JSON并解析为接口参数\n\n\n\n###  公共请求HEADER:\n\n| 参数            | 类型      | 是否必填 | 最大长度 | 描述             | 示例值        |\n|---------------|---------|------|------|----------------|------------|\n| X-Request-ID\t | String\t | 否\t\t  | 32\t\t | 请求ID,每次请求不同\t\t\t | xxxxx12456 |\n\n\n### 公共参数:\n\n> 以下参数通过GET形式传递\n\n> 当 Content-type:application/json 时,POST参数为JSON字符串,并作为 `GET payload` 参数代替\n\n| 参数         | 类型      | 是否必填 | 最大长度  | 描述                               | 示例值                                  |\n|------------|---------|------|-------|----------------------------------|--------------------------------------|\n| app_id     | String\t | 是\t\t  | 32\t\t  | 应用ID                             | test1                                |\n| version\t   | String\t | 是\t\t  | 3\t\t   | 调用的接口版本                          | 固定为：2.0                              |\n| timestamp\t | String\t | 是\t\t  | 19\t\t  | 发送请求的时间,格式"yyyy-MM-dd HH:mm:ss"\t | 2014-07-24 03:07:50                  |\n| sign\t\t     | String\t | 是\t\t  | 32\t\t  | 请求参数的签名串\t                        | 生成方式参见`签名生成`                         |\n| request_ip | String\t | 否\t\t  | 40\t\t  | 客户端IP,存在时加入签名                    | 127.0.0.1                            |\n| method\t\t   | String\t | 否\t\t  | 128\t\t | 接口名称,可以放到URL中,存在时加入签名            | product.detail                       |\n| token\t     | String\t | 否\t\t  | 无\t\t   | OAUTH登录后获取TOKEN,存在时需加入签名              | MTQtSldES1RIUVVZT0NGUkVNUEdJQlpBTlhM |\n| payload\t     | String\t | 否\t\t  | 无\t\t   | Content-type 非 application/json 时的作为接口参数,内容为JSON字符串      | {"id":1} |\n\n####  签名生成方法及示例:\n\n##### 签名涉及参数\n\n> 使用 app,version,timestamp,request_ip,method,token 及 请求 [`如POST中JSON`或`GET中payload`] 生成秘钥.\n> 其中 request_ip,method,token 及 [`如POST中JSON`或`GET中payload`] 为可选值\n\n##### 签名生成示例\n\n>  假设签名秘钥\n```\n3f95638a1e07b87df2b64e09c2541dac\n```\n\n---\n\n> 假设Content-type 为 application/json的POST请求:\n```\nGET:app_id=1212f&version=2.0&timestamp=2023-04-24 15:36:20&method=view&request_ip=fe80::e1bd:c78d:610f:3d03\nPOST:{"client_id":"1212f"}\n```\n\n> 把GET参数排序加payload跟秘钥,生成签名\n```\napp_id=1212f&method=view&request_ip=fe80%3A%3Ae1bd%3Ac78d%3A610f%3A3d03&timestamp=2023-04-24+15%3A36%3A20&version=2.0{"client_id":"1212f"}3f95638a1e07b87df2b64e09c2541dac\n```\n\n> 生成签名(转为小写)\n```\nd5d21befc41d017064e28a807ecd65b6\n```\n\n> 实际请求参数(POST&&Content-type:application/json)\n```\nGET:app_id=1212f&version=2.0&timestamp=2023-04-24+15%3A36%3A20&method=view&request_ip=fe80%3A%3Ae1bd%3Ac78d%3A610f%3A3d03&sign=d5d21befc41d017064e28a807ecd65b6\nPOST:{"client_id":"1212f"}\n```\n---\n\n\n\n> 假设Content-type 非 application/json的GET请求(其中GET存在可选参数 request_ip,payload):\n```\nGET:app_id=1212f&payload={"client_id":"1212f"}&request_ip=fe80::e1bd:c78d:610f:3d03&timestamp=2023-04-24:15:45:22&version=2.0\n```\n\n> 把GET参数排序加payload跟秘钥,生成签名\n```\napp_id=1212f&request_ip=fe80%3A%3Ae1bd%3Ac78d%3A610f%3A3d03&timestamp=2023-04-24+15%3A45%3A22&version=2.0{"client_id":"1212f"}3f95638a1e07b87df2b64e09c2541dac\n```\n\n> 生成签名(转为小写)\n\n```\n8fea66dc4b9928fa0664cbe06947e630\n```\n\n> 实际请求参数(GET)\n```\nGET:app_id=1212f&payload=%7B%22client_id%22%3A%221212f%22%7D&request_ip=fe80%3A%3Ae1bd%3Ac78d%3A610f%3A3d03&timestamp=2023-04-24+15%3A45%3A22&version=2.0&sign=8fea66dc4b9928fa0664cbe06947e630\n```\n---\n\n\n\n\n## OAUTH 请求参数\n\n### 登录地址获取\n\n> 请求路径为: ${OAUTH_HOST}/oauth.html\n\n> 重定向到生成的路径\n\n| 参数            | 类型      | 是否必填 | 最大长度 | 描述             | 示例值        |\n|---------------|---------|------|------|----------------|------------|\n| client_id  \t | String\t | 是\t\t  | 32\t\t | 应用ID,跟非OAUTH的参数app_id相同| xxxxx12456 |\n| redirect_uri  \t | String\t | 是\t\t  | 32\t\t | 登录后重定向地址,必须跟后台配置的域名匹配 | https://127.0.0.1:8080 |\n| response_type  \t | String\t | 是\t\t  | 4\t\t | 传入 code| code |\n| state  \t | String\t | 否\t\t  | 32\t\t | 随机值,完成登录原样返回| 23dfa |\n| scope  \t | String\t | 是\t\t  | 64\t\t | 授权功能,多个逗号分割:user_info 用户信息 \tuser_email 用户邮箱 user_mobile用户手机号  | user_info |\n\n\n####  登录地址获取示例:\n\n>  生成授权地址\n```\nhttp://175.178.90.181/ui/oauth.html?client_id=1212f&redirect_uri=http%3A%2F%2F127.0.0.1%3A8080%2F&response_type=code&scope=user_info&state=aa\n```\n\n> 授权完成后返回\n```\nhttp://127.0.0.1:8080/?code=27b5591cb788309dfee63da4fc264a10&state=aa\n```\n\n\n### 通过code获取授权token\n\n> 请求路径为: ${APP_HOST}/oauth/token\n\n> 无需签名,请求方式为GET\n\n| 参数            | 类型      | 是否必填 | 最大长度 | 描述             | 示例值        |\n|---------------|---------|------|------|----------------|------------|\n| client_id  \t | String\t | 是\t\t  | 32\t\t | 应用ID,跟非OAUTH的参数app_id相同| xxxxx12456 |\n| client_secret  \t | String\t | 是\t\t  | 32\t\t |OAuthSecret,从后台获取,注意:不是AppSecret | 2a97bf1b4f075b0ca7467e7c6b223f89 |\n| code  \t | String\t | 是\t\t  | 4\t\t | 登录后返回的code| 2a97bf1b4f075b0ca7467e7c6b223f89 |\n\n####  获取token示例:\n\n>  请求示例\n```\nGET:http://175.178.90.181/oauth/token?client_id=1212f&client_secret=2a97bf1b4f075b0ca7467e7c6b223f89&code=27b5591cb788309dfee63da4fc264a10\n```\n\n>  返回示例\n```\n{\n\t"response": {\n\t\t"access_token": "a4985f6747962b0ceb1533a0e28dd1fc",\n\t\t"expires_in": "1682929356",\n\t\t"openid": "1",\n\t\t"refresh_token": null,\n\t\t"scope": "user_info"\n\t},\n\t"result": {\n\t\t"code": "200",\n\t\t"message": "ok",\n\t\t"state": "ok"\n\t}\n}\n```\n\n\n### 刷新授权token\n\n> 请求路径为: ${APP_HOST}/oauth/refresh_token\n\n> 无需签名,请求方式为GET\n\n| 参数            | 类型      | 是否必填 | 最大长度 | 描述             | 示例值        |\n|---------------|---------|------|------|----------------|------------|\n| client_id  \t | String\t | 是\t\t  | 32\t\t | 应用ID,跟非OAUTH的参数app_id相同| xxxxx12456 |\n| client_secret  \t | String\t |是\t\t  | 32\t\t |OAuthSecret,从后台获取,注意:不是AppSecret | 2a97bf1b4f075b0ca7467e7c6b223f89 |\n| refresh_token  \t | String\t | 是\t\t  | 4\t\t | 通过oauth/token获取的TOKEN| 2a97bf1b4f075b0ca7467e7c6b223f89 |\n\n####  获取token示例:\n\n>  请求示例\n```\nGET:http://175.178.90.181/oauth/refresh_token?client_id=1212f&client_secret=2a97bf1b4f075b0ca7467e7c6b223f89&refresh_token=1cbefd9bf60598a17523042eca74836d\n```\n\n>  返回示例\n```\n{\n\t"response": {\n\t\t"access_token": "1cbefd9bf60598a17523042eca74836d",\n\t\t"expires_in": "1682931269",\n\t\t"openid": "1",\n\t\t"refresh_token": "a4985f6747962b0ceb1533a0e28dd1fc",\n\t\t"scope": "user_info"\n\t},\n\t"result": {\n\t\t"code": "200",\n\t\t"message": "ok",\n\t\t"state": "ok"\n\t}\n}\n```\n\n### 获取登录用户信息\n\n> 请求路径为: ${APP_HOST}/oauth/user\n\n> 需签名,参考`请求参数及加密秘钥生成说明(非OAUTH)`\n\n> 请求方式为POST,设置Content-Type:application/json\n\n#### 接口参数说明\n\n> GET参数 method 为 `info` ,其他参数查看`公共参数`说明\n\n> POST参数[JSON序列化]\n\n| 参数            | 类型      | 是否必填 | 最大长度 | 描述             | 示例值        |\n|---------------|---------|------|------|----------------|------------|\n| user  \t | bool\t | 否\t\t  | 1\t\t | 用户iD等基本资料|true |\n| name  \t | bool\t | 否\t\t  | 1\t\t | 用户登录名 需要 scope:user_info 授权|true |\n| info  \t | bool\t | 否\t\t  | 1\t\t |用户资料 需要 scope:user_info 授权 | true |\n| address  \t | bool\t | 否\t\t  | 1\t\t |用户收货地址 需要 scope:user_address 授权 | true |\n| email  \t | bool\t | 否\t\t  | 1\t\t |用户邮箱 需要 scope:user_email 授权 | true |\n| mobile  \t | bool\t | 否\t\t  | 1\t\t |用户手机号 需要 scope:user_mobile 授权   | true |\n\n\n####  获取登录用户示例:\n\n>  请求示例\n```\nGET:http://175.178.90.181/oauth/user?app_id=1212f&method=info&request_ip=fe80%3A%3Ae1bd%3Ac78d%3A610f%3A3d03&sign=8cdd52847cf6d5ce808c37cfc3d816c3&timestamp=2023-04-24+16%3A46%3A45&token=a4985f6747962b0ceb1533a0e28dd1fc&version=2.0\nPOST:{"address":false,"email":false,"info":false,"mobile":false,"name":true,"user":true}\n```\n\n>  返回示例\n```\n{\n\t"response": {\n\t\t"user_data": {\n\t\t\t"address": null,\n\t\t\t"email": null,\n\t\t\t"info": null,\n\t\t\t"mobile": null,\n\t\t\t"name": {\n\t\t\t\t"change_time": 1682318126,\n\t\t\t\t"id": 1,\n\t\t\t\t"user_id": 1,\n\t\t\t\t"username": "aaaaa"\n\t\t\t},\n\t\t\t"user": {\n\t\t\t\t"add_time": 1667904484,\n\t\t\t\t"address_count": 0,\n\t\t\t\t"change_time": 1682318140,\n\t\t\t\t"confirm_time": 0,\n\t\t\t\t"email_count": 15,\n\t\t\t\t"external_count": 5,\n\t\t\t\t"id": 1,\n\t\t\t\t"mobile_count": 4,\n\t\t\t\t"nickname": "测试用户-已开所有权限",\n\t\t\t\t"password_id": 40,\n\t\t\t\t"status": 2,\n\t\t\t\t"use_name": 1\n\t\t\t}\n\t\t}\n\t},\n\t"result": {\n\t\t"code": "200",\n\t\t"message": "ok",\n\t\t"state": "ok"\n\t}\n}\n```\n\n\n\n\n## 返回数据说明\n\n> OAUTH跟非OAUTH的请求格式返回一致\n\n###  公共返回HEADER:\n\n| 参数            | 类型      | 是否必填 | 最大长度 | 描述                      | 示例值        |\n|---------------|---------|------|------|-------------------------|------------|\n| X-Request-ID\t | String\t | 否\t\t  | 32\t\t | 如果请求时存在原样返回,否则系统内部会生成一个 | xxxxx12456 |\n\n\n### 公共返回参数说明:\n\n> 当code为200时系统正常,当state为ok时业务无异常\n\n| 参数              | 类型            | 是否必返回 | 最大长度 | 描述                    |\n|-----------------|---------------|-------|------|-----------------------|\n| result.code\t    | String\t       | 是\t\t\t  | 12\t\t | 系统状态码,除200外均为异常       |\n| result.state\t   | String\t       | 是\t\t\t  | 12\t\t | 业务状态,ok 正常,其他可能参见业务说明 |\n| result.message\t | String\t       | 是\t\t\t  | 256  | 相关消息                  |\n| response\t\t      | Array,Object\t | 是\t\t\t  | 无    | 接口数据,可能为:{}或[],参见具体接口说明     |\n\n#### 返回参数示例:\n\n```\n{\n    "result": {\n        "code": "200",\n        "state": "ok",\n        "message": "add success"\n    },\n    "response": {\n\t\t"product":{\n\t\t\t"name":"Iphone",\n\t\t\t"sku":"111_11"\n\t\t}\n\t}\n}\n```';


function PageNav() {
    let param = useParams()//从请求url中获取数据
    let baeadTip = Menus.find((e) => {
        if (param["*"] != '' && e.url.indexOf(param["*"]) != -1) return true
    });
    return baeadTip ? (
        <Breadcrumbs >
            <Link component={RouteLink}
                underline="hover"
                sx={{ display: 'flex', alignItems: 'center' }}
                color="inherit"
                to=""
            >
                <PersonIcon sx={{ mr: 0.5 }} fontSize="inherit" />
                用户中心
            </Link>
            <Typography
                sx={{ display: 'flex', alignItems: 'center', color: '#999' }}

            >
                {baeadTip.text}
            </Typography></Breadcrumbs>
    ) : (<Breadcrumbs >
        <Typography
            sx={{ display: 'flex', alignItems: 'center', color: '#999' }}

        >
            用户中心
        </Typography></Breadcrumbs>)

}



export const Menus = [
    {
        url: "/system/app",
        icon: ApiIcon,
        text: "应用审核",
        rbac: [{
            name: "admin-app"
        }]
    },
];



export default function DocPage() {
    return <Box sx={{ display: 'flex' }}>
        <Box sx={{ minWidth: 200, minHeight: "calc(100vh - 69px)", borderRight: " 1px solid #eee" }}>
            <List>
                {Menus.map((item) => {
                    const Icon = item.icon;
                    return <ListItem key={`system-${item.url}`} disablePadding>
                        <ListItemButton prefix="/system/" to={item.url}>
                            <ListItemIcon>
                                <Icon />
                            </ListItemIcon>
                            <ListItemText primary={item.text} />
                        </ListItemButton>
                    </ListItem>;
                })}
            </List>
        </Box><Box sx={{ flex: 1 }}>
            <PageNav />
            <MarkdownPreview
                style={{ width: "100vh", margin: "24px" }}
                wrapperElement={{
                    "data-color-mode": "light"
                }}
                source={source} />
        </Box>
    </Box >
        ;
}


