

import { default as AddCircleOutlineIcon } from '@mui/icons-material/AddCircleOutline';
import DatasetLinkedOutlinedIcon from '@mui/icons-material/DatasetLinkedOutlined';
import { Box, Button, FormControl, Paper, Stack, TextField } from '@mui/material';
import { LocalizationProvider } from '@mui/x-date-pickers';
import { AdapterDayjs } from '@mui/x-date-pickers/AdapterDayjs';
import React, { Fragment, useCallback, useContext, useEffect, useRef, useState } from 'react';
import isEmail from "validator/lib/isEmail";
import { UserSessionContext } from '../../../../common/context/session';
import { ClearTextField } from '../../../../library/input';
import { LoadingButton } from '../../../../library/loading';
import { mailAppSend, mailListTplConfig } from '../../../../common/rest/sender_setting';
import { TplSelect } from '../../common/sender/lib_tpl_select';
import { UserAppSendBox, UserAppSendParseBox } from '../../common/sender/send';
import LogoDevIcon from '@mui/icons-material/LogoDev';
import HelpOutlineOutlinedIcon from '@mui/icons-material/HelpOutlineOutlined';


export default function UserAppMailSendPage(props) {
    const { userData } = useContext(UserSessionContext)
    const scrollRef = useRef(null);
    const runIndex = useRef(-1);
    let [sendData, setSendData] = useState({
        data: [],
        disabled: false,
        send: false,
        reply: '',
        tpl_id: '',
        disabled: false,
    });
    const ajaxReq = useRef(null);
    const sendDo = useCallback(() => {
        if (parseInt(runIndex.current) + 1 >= sendData.data.length) {
            setSendData({
                ...sendData,
                send: false,
                disabled: false,
            })
            runIndex.current = -1;
            return;
        }
        ajaxReq.current = new AbortController();
        for (let i in sendData.data) {
            if (parseInt(runIndex.current) >= i) continue;
            runIndex.current = i
            let tmp = sendData.data[i];
            if (tmp.finish) continue;
            let tmpData = [...sendData.data];
            tmpData[i].loading = true;
            setSendData({
                ...sendData,
                data: tmpData
            })
            let sdata = {};
            for (let sval of tmp.tplData) {
                sdata[sval.name] = sval.val;
            }
            mailAppSend({
                tpl_id: sendData.tpl_id,
                data: sdata,
                to: tmp.mail,
                reply: sendData.reply,
                max_try: tmp.tryNum,
                send_time: tmp.nowSend ? null : tmp.sendTime.format('YYYY-MM-DD HH:mm:ss')
            }, {
                signal: ajaxReq.current.signal
            }).then((data) => {
                ajaxReq.current = null;
                let tmpData = [...sendData.data];
                tmpData[i].loading = false;
                if (!data.status) {
                    tmpData[i].error = data.message;
                    setSendData({
                        ...sendData,
                        data: tmpData
                    })
                } else {
                    tmpData[i].error = null;
                    tmpData[i].finish = true;
                    setSendData({
                        ...sendData,
                        data: tmpData
                    })
                }
                sendDo()
            })
            break
        }
    }, [sendData])
    useEffect(() => {
        runIndex.current = -1;
        if (ajaxReq.current) {
            ajaxReq.current.abort()
            ajaxReq.current = null;
        }
        sendData.send && sendDo();
    }, [sendData.send])

    useEffect(() => {
        //console.log(scrollRef.current)
        if (scrollRef.current && sendData.send) {
            scrollRef.current.scrollIntoView();
        }
    }, [scrollRef.current, runIndex.current])

    return <Fragment>
        <Paper
            sx={{ p: 2, display: 'flex', alignItems: 'center', marginBottom: 1, marginTop: 1 }}
        >
            <LocalizationProvider dateAdapter={AdapterDayjs}>
                <Stack spacing={2} >
                    <Stack spacing={2}
                        direction={"row"}
                        justifyContent="flex-start"
                        alignItems="flex-start"
                    >
                        <TplSelect
                            sx={{ width: 380 }}
                            MenuProps={{
                                anchorOrigin: {
                                    vertical: 'bottom',
                                    horizontal: 'left',
                                },
                                transformOrigin: {
                                    vertical: 'top',
                                    horizontal: 'left',
                                },
                            }}
                            tplId={sendData.tpl_id}
                            userId={parseInt(userData.user_data.user_id)}
                            onChange={(e) => {
                                setSendData({
                                    ...sendData,
                                    tpl_id: e.target.value
                                })
                            }}
                            onLoad={(data) => {
                                let reply;
                                if (data.config_data.reply_email) {
                                    reply = data.config_data.reply_email
                                }
                                if (data.config_data.from_email) {
                                    reply = data.config_data.from_email
                                }
                                if (reply) {
                                    setSendData({
                                        ...sendData,
                                        tpl_id: data.id,
                                        reply: reply,
                                        data: []
                                    })
                                }
                            }}
                            loadData={mailListTplConfig}
                        />
                        <Button
                            variant="outlined"
                            size="medium"
                            startIcon={<LogoDevIcon />}
                            endIcon={<HelpOutlineOutlinedIcon fontSize='small' />}
                            sx={{ mr: 1, p: "7px 15px", minWidth: 110 }}
                            onClick={() => {
                                window.open("https://github.com/shanliu/lsys/tree/main/sdk/go/examples/basic/mail_test.go", "_blank")
                            }}>
                            通过代码发送
                        </Button>
                    </Stack>
                    <Box sx={{ width: 380 }} >
                        <Stack spacing={2} >

                            {sendData.tpl_id > 0 ? <FormControl>
                                <ClearTextField
                                    disabled={sendData.disabled}
                                    variant="outlined"
                                    value={sendData.reply}
                                    label={`回复邮箱`}
                                    type="email"
                                    size="small"
                                    onChange={(event, nval) => {
                                        setSendData({
                                            ...sendData,
                                            reply: nval
                                        })
                                    }}
                                />
                            </FormControl> : null}
                        </Stack>
                        {sendData.tpl_id > 0 ? <Stack
                            direction="row"
                            spacing={1}
                            justifyContent="space-between"
                            flexWrap="wrap"
                            alignItems="center" sx={{ mt: 2, sx: 1 }}>
                            <Box sx={{ flexGrow: 1, }}>
                                <Button
                                    disabled={sendData.disabled}
                                    fullWidth
                                    variant="outlined"
                                    size="small"
                                    startIcon={<AddCircleOutlineIcon />}
                                    onClick={() => {
                                        setSendData({
                                            ...sendData,
                                            data: [...sendData.data, {
                                                tplData: [],
                                                mail: [],
                                                nowSend: true,
                                                sendTime: null,
                                                loading: false,
                                                error: null,
                                                finish: false
                                            }]
                                        })
                                    }}>
                                    添加发送
                                </Button>
                            </Box>

                            <UserAppSendParseBox
                                tips={`#注释,一行一条记录
#示例1:变量名:变量值,变量名:变量值;接收邮箱,接收邮箱
var1:111,var2:222;1@1.com,1@1.com
#示例2:变量名: 变量值,变量名:变量值;发送时间;接收邮箱,接收邮箱,
var1:111,var2:222;2023-11-11 11:11:11;1@1.com,1@1.com
#示例2:变量名: 变量值,变量名:变量值;发送时间;接收邮箱,接收邮箱;重试次数,
var1:111,var2:222;2023-11-11 11:11:11;1@1.com,1@1.com;3`}
                                sx={{ flexGrow: 1, }}
                                checkValue={(val) => {
                                    if (!isEmail(val)) {
                                        return `${val} 非邮箱格式`
                                    }
                                }}
                                onParse={(data) => {
                                    setSendData({
                                        ...sendData,
                                        data: data.map((tmp) => {
                                            return {
                                                tplData: tmp.tplData,
                                                mail: tmp.to,
                                                nowSend: tmp.nowSend,
                                                sendTime: tmp.sendTime,
                                                tryNum: tmp.tryNum,
                                                loading: false,
                                                error: null,
                                                finish: false
                                            }
                                        })
                                    })
                                }}>
                                <Button
                                    disabled={sendData.disabled}
                                    fullWidth
                                    variant="outlined"
                                    size="small"
                                    startIcon={<DatasetLinkedOutlinedIcon />}
                                >
                                    解析数据
                                </Button>
                            </UserAppSendParseBox>

                        </Stack> : null}

                    </Box>
                    {sendData.data.map((item, i) => {
                        //console.log(item)
                        return <UserAppSendBox
                            ref={item.loading ? scrollRef : null}
                            disabled={sendData.disabled}
                            onDelete={() => {
                                let data = [...sendData.data];
                                delete data[i];
                                data = data.filter(e => e);
                                setSendData({
                                    ...sendData,
                                    data: data
                                })
                            }}
                            key={`send-box-${i}`}
                            tplData={item.tplData}
                            nowSend={item.nowSend}
                            sendTime={item.sendTime}
                            tryNum={item.tryNum ?? 0}
                            onTplDataChange={(tplData, nowSend, sendTime, num) => {
                                //                                console.log("---", tplData, nowSend, sendTime);
                                let data = [...sendData.data];
                                data[i].tplData = tplData;
                                data[i].nowSend = nowSend;
                                data[i].sendTime = sendTime ? sendTime : null;
                                data[i].tryNum = num <= 0 ? null : num;
                                setSendData({
                                    ...sendData,
                                    data: data
                                })
                            }}
                            loading={item.loading}
                            error={item.error}
                            finish={item.finish}
                        >
                            <TextField
                                disabled={sendData.disabled || item.finish}
                                onChange={(e) => {
                                    let val = e.target.value.replaceAll("\r\n", "\n").split("\n")
                                    let data = [...sendData.data];
                                    data[i].mail = val;
                                    setSendData({
                                        ...sendData,
                                        data: data
                                    })
                                }}
                                value={item.mail.join("\n")}
                                size="small"
                                label={`接收邮箱`}
                                type='text'
                                fullWidth
                                multiline
                                minRows={1}
                            />
                        </UserAppSendBox>
                    })}
                    {sendData.data.length ? <Stack
                        direction="row"
                        spacing={1}
                        justifyContent="space-between"

                        flexWrap="wrap"
                        alignItems="center" sx={{ mt: 2, sx: 1 }}>
                        <LoadingButton sx={{ flexGrow: 1, }} variant="contained" type="submit"
                            onClick={() => {
                                setSendData({
                                    ...sendData,
                                    send: true,
                                    disabled: true
                                })
                            }}
                            disabled={sendData.disabled}  >发送</LoadingButton>
                        <LoadingButton sx={{ flexGrow: 1, }} variant="contained" type="submit" disabled={!sendData.disabled}
                            onClick={() => {
                                setSendData({
                                    ...sendData,
                                    send: false,
                                    disabled: false
                                })
                            }}>取消</LoadingButton>

                    </Stack> : null}


                </Stack>
            </LocalizationProvider>
        </Paper >
    </Fragment >
}


