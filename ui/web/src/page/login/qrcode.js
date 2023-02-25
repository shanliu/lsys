import { Stack, Typography } from "@mui/material";
import { Box } from "@mui/system";
import { toDataURL } from 'qrcode';
import randomString from "random-string";
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { SessionSetData, UserSessionContext } from "../../context/session";
import { ToastContext } from "../../context/toast";
import { Progress } from "../../library/loading";
import { QrcodeLogin, QrcodeLoginCheck } from "../../rest/login";
export function QrCodeLoginPage(props) {
    const { type, label, onLogged } = props;
    const { toast } = useContext(ToastContext);
    const { dispatch } = useContext(UserSessionContext);
    const [qrData, setQrData] = useState({
        src: null,
        state: randomString(),
        loading: true
    })
    const [qrCheck, setQrCheck] = useState({
        check: false,
        hand: null,
        checking: false
    })
    useEffect(() => {
        return () => {
            qrCheck.hand && clearTimeout(qrCheck.hand);
            setQrCheck({
                checking: false,
                hand: null,
                check: false,
            })
        }
    }, [props.type])
    useEffect(() => {
        if (qrCheck.hand || !qrCheck.check) return
        let hand = setTimeout(() => {
            setQrCheck({
                check: true,
                hand: hand,
                checking: true
            })
            QrcodeLoginCheck(type, qrData.state).then((data) => {
                if (data.auth_data && data.token) {
                    dispatch(SessionSetData(data, true))
                    onLogged()
                    return
                }
                if (!data.status) {
                    toast(data.message);
                }
                if (data.reload || !data.status) {
                    setQrCheck({
                        check: false,
                        hand: null,
                        checking: false,
                    })
                    setQrData({
                        ...qrData,
                        state: randomString()
                    })
                } else {
                    setQrCheck({
                        check: true,
                        hand: null,
                        checking: false,
                    })
                }
            })
        }, 2000);
        setQrCheck({
            check: true,
            checking: false,
            hand: hand
        })
    }, [qrCheck.hand, qrData.state, props.type, qrCheck.check])

    useEffect(() => {
        setQrData({
            ...qrData,
            loading: true
        })
        QrcodeLogin(type, qrData.state).then((data) => {
            if (!data.status) {
                setQrData({
                    ...qrData,
                    loading: false
                })
                toast(data.message);
                return
            }
            toDataURL(data.url, function (err, url) {
                if (err) {
                    toast('二维码生成错误:' + err + '')
                } else {
                    setQrData({
                        ...qrData,
                        src: url,
                        loading: false
                    })
                    setQrCheck({
                        check: true,
                        hand: null,
                        checking: false,
                    })
                }
            })
        })
    }, [qrData.state, props.type])

    return <Fragment>
        <Stack sx={{
            alignItems: "center",
        }}>
            <Box sx={{
                width: "250px",
                height: "250px"
            }}>
                {
                    qrData.loading ? <Progress /> : <img style={{
                        width: '100%',
                        height: '100%'
                    }} src={qrData.src} />
                }
            </Box>
            <Typography
                align="center"
                variant="subtitle1"
                noWrap
                sx={{
                    mt: "4px",
                    mb: "4px",
                    fontWeight: 100,
                    alignItems: "center",
                    letterSpacing: '.3rem',
                    color: 'inherit',
                    fontSize: ".5em",
                    textDecoration: 'none',
                }}
            >
                请使用{label}扫码二维码完成登录
            </Typography>
            <Box sx={{
                height: "15px",
                lineHeight: "15px",
                alignItems: "center",
            }}>
                {qrCheck.checking ? <span style={{
                    display: "block",
                    fontSize: "0.5em"
                }}>检测登录中</span> : ''}
            </Box>
        </Stack>
    </Fragment>
}