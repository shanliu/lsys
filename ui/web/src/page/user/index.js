import { Box } from '@mui/system';
import React from 'react';

export default function UserIndexPage() {
    return <Box>
        <p>
            欢迎,这是一个服务端以 rust语言, 网页端以reactjs框架 为基础开发的项目。
        </p>
        <p>
            目标：作为 小微企业的 内部公共服务、内部应用管理 及 轻量级开放平台 的解决方案
        </p>
        <p>
            目前项目还在<b>迭代开发</b>中，将不定期更新.
        </p>
        <p>
            请不要恶意使用本示例项目,如有需要可以clone github.com/shanliu/lsys 自行部署
        </p>
        <p>
            发现bug,可发送邮件:<a href="mailto:shan.liu@msn.com">shan.liu#msn.com</a>(把#改为@)
        </p>
        <p>
            <b>本项目代码协议: Apache-2.0 license </b>
        </p>
    </Box >

}

