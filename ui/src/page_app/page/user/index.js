import { Box } from '@mui/system';
import React from 'react';

export default function UserIndexPage() {
    return <Box>
        <p>
            这是一个服务端以 rust(actix-web)语言, 网页端以reactjs 为基础开发的项目。
        </p>
        <p>
            目标：作为 内部应用管理 及 轻量级开放平台 的解决方案
        </p>
        <p>
            目前项目还在<b>迭代开发</b>中，将不定期更新。
        </p>
        <p>
            请不要恶意使用本示例项目,如有需要可以 克隆本项目代码 自行部署测试。
        </p>
        <p>
            发现bug,可发送邮件:<a href="mailto:shan.liu@msn.com">shan.liu#msn.com</a>(把#改为@)
        </p>
        <p>
            <b>本项目代码协议: Apache-2.0 license </b>
        </p>
    </Box >

}

