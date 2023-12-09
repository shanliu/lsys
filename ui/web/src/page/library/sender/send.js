

import CloseIcon from '@mui/icons-material/Close';
import DeleteIcon from '@mui/icons-material/Delete';
import SendOutlinedIcon from '@mui/icons-material/SendOutlined';

import { DesktopDateTimePicker } from '@mui/x-date-pickers';
import { ItemTooltip } from '../../../library/tips';
import AddCircleOutlineIcon from '@mui/icons-material/AddCircleOutline';
import { Alert, Box, Button, Dialog, DialogActions, DialogContent, DialogTitle, Divider, FormControl, FormControlLabel, IconButton, Paper, Stack, Switch, TextField, Typography } from '@mui/material';
import dayjs from 'dayjs';
import PropTypes from 'prop-types';
import React, { Fragment, forwardRef, useEffect, useState } from 'react';
import HelpOutlineOutlinedIcon from '@mui/icons-material/HelpOutlineOutlined';
function UserAppSendDataItem(props) {

    const { tplData, onDelete, onChange, disabled } = props
    return <Stack direction={"row"} spacing={1}>
        <TextField
            label="变量名"
            size="small"
            disabled={disabled}
            value={tplData.name}
            onChange={(e) => {
                let val = (e.target.value + '').trim()
                onChange({
                    ...tplData,
                    name: val,
                })
            }}
        />
        <TextField
            disabled={disabled}
            size="small"
            label={`变量数据`}
            fullWidth
            value={tplData.val}
            onChange={(e) => {
                let val = (e.target.value + '').trim()
                onChange({
                    ...tplData,
                    val: val,
                })
            }}
        />
        <IconButton disabled={disabled} onClick={() => { onDelete() }}>
            <DeleteIcon fontSize='small' />
        </IconButton>
    </Stack>
}

UserAppSendDataItem.propTypes = {
    disabled: PropTypes.bool.isRequired,
    tplData: PropTypes.object.isRequired,
    onDelete: PropTypes.func.isRequired,
    onChange: PropTypes.func.isRequired,
};



function UserAppSendDataTime(props) {
    const { nowSend, sendTime, tryNum, changeSendTime, changeTryNum, disabled } = props

    return <Stack direction="row">
        <FormControl fullWidth>
            <DesktopDateTimePicker
                disabled={nowSend || disabled}
                inputFormat="YYYY-MM-DD hh:mm:ss"
                label="发送时间"
                variant="outlined"
                size="small"
                minDateTime={dayjs(new Date())}
                value={(sendTime === null || sendTime === undefined) ? null : sendTime}
                onChange={(value) => {
                    changeSendTime(false, value)
                }}
                renderInput={(params) => <TextField
                    size="small" name="timeout" {...params} sx={{
                        width: 1
                    }} />}
            />
        </FormControl>

        <FormControlLabel

            label={<span style={{ fontSize: "0.75rem" }}>立即发送</span>
            }
            sx={{ color: "#666", width: 180, ml: 1 }}
            control={
                < Switch
                    checked={nowSend}
                    disabled={disabled}
                    size="small"
                    onChange={(e) => {
                        changeSendTime(e.target.checked, null)
                    }} />
            }
        />
        < FormControl
            sx={{ width: 220, ml: 1 }}
        >
            <TextField
                disabled={disabled}
                size="small"
                label={`失败重试(次)`}
                type='number'
                value={tryNum > 0 ? tryNum : ""}
                onChange={(e) => {
                    let num = parseInt(e.target.value);
                    num = num > 0 ? num : null
                    changeTryNum(num)
                }}
            />
        </FormControl >
    </Stack >
}

UserAppSendDataTime.propTypes = {
    nowSend: PropTypes.bool.isRequired,
    sendTime: PropTypes.object,
    disabled: PropTypes.bool.isRequired,
    changeSendTime: PropTypes.func.isRequired,
    tryNum: PropTypes.number.isRequired,
};

export const UserAppSendBox = forwardRef((props, ref) => {
    const { tplData, onTplDataChange, nowSend, sendTime, tryNum, error, finish, loading, onDelete, disabled } = props
    return <Paper sx={{ width: 500 }} ref={ref}>
        <Stack spacing={1} >
            <Stack spacing={1} sx={{ p: 2, pb: 0 }}
                direction="row"
                justifyContent="flex-start"
                alignItems="flex-start">
                <Box>
                    <Button
                        disabled={disabled || finish}
                        color="info"
                        size="medium"
                        startIcon={<AddCircleOutlineIcon />}
                        onClick={() => {
                            let setData = [...tplData];
                            setData.push({ name: '', val: '' });
                            onTplDataChange(setData, nowSend, sendTime, tryNum)
                        }}>
                        添加变量
                    </Button>
                </Box>
                <Box sx={{ flex: 1 }}>
                    {finish ? <Alert sx={{ pt: 0, pb: 0 }} severity="success">
                        已完成发送
                    </Alert> : (
                        loading ? <Alert sx={{ pt: 0, pb: 0 }} severity="info">
                            发送中
                        </Alert> : error ? <Alert sx={{ pt: 0, pb: 0 }} severity="error">
                            发送错误:{error}
                        </Alert> : <Alert sx={{ pt: 0, pb: 0 }} severity="info">
                            待发送
                        </Alert>
                    )}

                </Box>
                <IconButton onClick={onDelete} disabled={disabled}>
                    <CloseIcon disabled={disabled} fontSize="small" />
                </IconButton>
            </Stack>
            <Divider textAlign="left"><SendOutlinedIcon sx={{ pt: 1 }} color='disabled' fontSize='medium' /></Divider>
            <Stack spacing={2} sx={{ p: 2, pt: 0 }}  >
                {tplData.map((item, i) => {
                    return <UserAppSendDataItem key={`send-data-var-${i}`} disabled={disabled || finish} tplData={item} onChange={(dat) => {
                        let setData = [];
                        for (const ti in tplData) {
                            if (ti == i) setData.push(dat);
                            else setData.push(tplData[ti]);
                        }
                        onTplDataChange(setData, nowSend, sendTime, tryNum)
                    }} onDelete={() => {
                        let setData = [];
                        for (const ti in tplData) {
                            if (ti != i) setData.push(tplData[ti]);
                        }
                        onTplDataChange(setData, nowSend, sendTime, tryNum)
                    }} />
                })}
                {props.children}
                <UserAppSendDataTime
                    disabled={disabled || finish}
                    nowSend={nowSend}
                    sendTime={sendTime}
                    tryNum={tryNum}
                    changeTryNum={(num) => {
                        onTplDataChange(tplData, nowSend, sendTime, num)
                    }}
                    changeSendTime={(nowSend, sendTime) => {
                        onTplDataChange(tplData, nowSend, sendTime, tryNum)
                    }} />
            </Stack>
        </Stack>
    </Paper>

});
UserAppSendBox.propTypes = {
    finish: PropTypes.bool.isRequired,
    error: PropTypes.string,
    loading: PropTypes.bool.isRequired,
    disabled: PropTypes.bool.isRequired,
    tplData: PropTypes.array.isRequired,
    onTplDataChange: PropTypes.func.isRequired,
    onDelete: PropTypes.func.isRequired,
    nowSend: PropTypes.bool.isRequired,
    sendTime: PropTypes.object,
    tryNum: PropTypes.number.isRequired,
};

UserAppSendParseBox.propTypes = {
    checkValue: PropTypes.func.isRequired,
    onParse: PropTypes.func.isRequired,
};

export function UserAppSendParseBox(props) {
    const { children, onParse, checkValue, tips, ...other } = props;
    let [boxData, setBoxData] = useState({
        open: false,
        val: '',
        error: ""
    });
    let [tipOpen, setTipOpen] = useState(false);
    return <Fragment>
        <Dialog onClose={() => {
            setBoxData({
                ...boxData,
                open: false,
            })
        }} open={boxData.open} maxWidth={false} >
            <DialogTitle sx={{
                width: 800,
            }} >
                <span>请输入要发送的数据</span>
                <ItemTooltip leaveDelay={800} open={tipOpen} onClose={() => { setTipOpen(false) }} componentsProps={{
                    transition: {
                        sx: { maxWidth: 470, top: "-8px!important" }
                    }
                }} title={<Box sx={{ width: 470 }} onClick={() => {
                    if (boxData.val == '') {
                        setBoxData({
                            ...boxData,
                            val: tips,
                        })
                    }
                }}>
                    <pre>{tips}</pre>
                </Box>} placement="bottom" >
                    <IconButton onClick={() => { setTipOpen(!tipOpen) }} size='small' sx={{ position: "absolute", right: 10, top: 10 }}>
                        <HelpOutlineOutlinedIcon fontSize='small' />
                    </IconButton>
                </ItemTooltip>
            </DialogTitle>
            <DialogContent sx={{ pb: 1 }}>

                <TextField
                    InputProps={{
                        style: { padding: 4 },
                        inputProps: {
                            wrap: "off"
                        }
                    }}
                    value={boxData.val}
                    onChange={(e) => {
                        setBoxData({
                            ...boxData,
                            val: e.target.value
                        })
                    }}
                    onBlur={(e) => {
                        let val = e.target.value.trim()
                        setBoxData({
                            ...boxData,
                            val: val
                        })
                    }}
                    error={boxData.error && boxData.error.length ? true : false}
                    helperText={boxData.error}
                    fullWidth
                    multiline
                    minRows={15}
                />
            </DialogContent>
            <DialogActions sx={{ m: 2, mt: 0 }}>
                <Button variant="contained" type="submit" onClick={() => {
                    if (boxData.val == '') {
                        setBoxData({
                            ...boxData,
                            error: "请输入需要解析的数据"
                        })
                        return
                    }
                    let out = [];
                    let error;
                    let valdat = boxData.val.replaceAll("\r\n", "\n").split("\n")
                    for (let ind in valdat) {
                        ind = parseInt(ind);
                        if (!valdat[ind] || valdat[ind].trim() == '' || /^\s*#/.test(valdat[ind])) {
                            continue;
                        }
                        let items = valdat[ind].split(";");
                        if (items.length != 2 && items.length != 3 && items.length != 4) {
                            error = "解析第" + (ind + 1) + "行失败,仅能包含3个以内的;号"
                            break;
                        }
                        let numTry = null;
                        if (items.length == 4) {
                            let tmp = parseInt(items.pop())
                            if (tmp > 0 && tmp < 20) {
                                numTry = tmp
                            }
                        } else if (items.length == 3) {
                            if (/^\d+$/.test(items[2])) {
                                numTry = parseInt(items.pop())
                            }
                        }
                        let out_val = [];
                        let vals = (items.pop() + '').split(",")
                        for (let t in vals) {
                            if (vals[t].trim() == '') {
                                continue;
                            }
                            let valid = checkValue(vals[t])
                            if (valid) {
                                error = "解析第" + (ind + 1) + "行失败,错误信息:" + valid
                                break
                            }
                            out_val.push(vals[t])
                        }
                        if (error) break
                        let out_var = []
                        let varData = items[0].trim().split(",")
                        for (let t in varData) {
                            let vtmp = varData[t].split(":");
                            if (vtmp.length != 2) {
                                error = "解析第" + (ind + 1) + "行失败,变量需使用:连接"
                                break;
                            }
                            out_var.push({
                                name: vtmp[0],
                                val: vtmp[1],
                            })
                        }
                        let ns = true;
                        let st = null;
                        if (error) break
                        if (items.length == 2) {
                            let time = Date.parse(items[1])
                            if (isNaN(time)) {
                                error = "解析第" + (ind + 1) + "行日期失败,请提供类似:2023-01-01 01:01:01的格式"
                                break;
                            }
                            st = dayjs(time);
                            ns = false
                        }

                        out.push({
                            tplData: out_var,
                            to: out_val,
                            nowSend: ns,
                            sendTime: st,
                            tryNum: numTry
                        })
                    }
                    if (error) {
                        setBoxData({
                            ...boxData,
                            error: error
                        })
                        return
                    }
                    onParse(out)
                    setBoxData({
                        ...boxData,
                        open: false,
                        val: ''
                    })
                }} >解析数据</Button>
            </DialogActions>
        </Dialog>
        <Box {...other} onClick={() => {
            setBoxData({
                ...boxData,
                open: true
            })
        }}>
            {children}
        </Box>
    </Fragment>
}
