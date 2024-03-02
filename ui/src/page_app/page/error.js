
import { Box } from '@mui/material';
import React, { useEffect, useState } from 'react';
import { useRouteError } from 'react-router-dom';
export function ErrorPage() {
    const error = useRouteError();//错误信息
    const [go, setGo] = useState(10)
    useEffect(() => {
        let a = setInterval(() => {
            if (go - 1 <= 0) {
                window.location.reload();
                return;
            }
            setGo(go - 1)
        }, 1000);
        return () => {
            clearInterval(a)
        }
    })
    return (
        <Box id="error-page">
            <h1>发生错误!</h1>
            <p>你请求的页码当前不可用,请稍后再尝试.错误信息如下:</p>
            <p>
                <i>{error.statusText || error.message}</i>
            </p>
            {<Box>将在 {go} 秒后刷新本页面</Box>}
        </Box>
    );
}
