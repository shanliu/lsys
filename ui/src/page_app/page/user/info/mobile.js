import AddCircleOutlineIcon from '@mui/icons-material/AddCircleOutline';
import DeleteIcon from '@mui/icons-material/Delete';
import SearchIcon from '@mui/icons-material/Search';
import SendIcon from '@mui/icons-material/Send';
import { Alert, Button, Drawer, FormControl, IconButton, InputLabel, MenuItem, Paper, Select } from '@mui/material';
import Box from '@mui/material/Box';
import randomString from 'random-string';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { ToastContext } from '../../../../common/context/toast';
import { ConfirmButton } from '../../../../common/ui/dialog';
import { DataPaginationTablePage } from '../../../../library/table_page';
import { mobileAdd, mobileComfirmStatus, mobileConfirm, mobileDelete, mobileList, mobileSendCode } from '../../../../common/rest/user';
import { useSearchChange } from '../../../../common/utils/hook';
import { captchaSrc } from '../../../../common/utils/rest';
import { showTime } from '../../../../common/utils/utils';
import { AddBox, CodeBox, ConfirmBox } from './valid_box';
import { LoadingButton } from '../../../../library/loading';


export default function UserMobilePage(props) {
    const comfirmStatus = mobileComfirmStatus;
    //列表数据
    let [loadData, setLoadData] = useState({
        loading: true,
        status: false,
        data: [],
        message: null,
    });

    const [param, setParam] = useSearchChange({
        status: "",
        page: 0,
        page_size: 25
    });
    const loadMobileData = () => {
        setLoadData({
            ...loadData,
            loading: true
        })
        window.scrollTo({ top: 0, left: 0, behavior: 'smooth' });
        mobileList().then((data) => {

            setLoadData({
                ...loadData,
                ...data,
                data: data.status ? data.data : [],
                loading: false
            })
        })
    };

    useEffect(loadMobileData, [])



    //添加跟更新
    const [changeBoxState, setChangeBox] = useState(0);
    const { toast } = useContext(ToastContext);
    const captchaType = 'add-sms'

    const showCodeBox = (id, name) => {
        const captchaKey = randomString()
        setCodeData({
            ...codeData,
            id: id,
            name: name,
            captcha_key: captchaKey,
            captcha_src: captchaSrc(captchaType + '/' + captchaKey, true),
        })
        setChangeBox(2)
    }

    const showConfirmBox = (id, name) => {
        setConfrimData({
            ...confrimData,
            id: id,
            name: name,
        })
        setChangeBox(3)
    }

    // add
    const [addData, setAddData] = useState({
        name: '',
        loading: false,
    });
    const [addError, setAddError] = useState({
        name: '',
    });
    const doAdd = function () {
        setAddData({
            ...addData,
            loading: true
        })
        mobileAdd({ mobile: addData.name }).then((data) => {

            if (!data.status) {
                toast(data.message)
                setAddData({
                    ...addData,
                    loading: false,
                })
                setAddError({
                    ...addError,
                    ...data.field
                })
            } else {
                showCodeBox(data.id, addData.name)
                setAddData({
                    ...addData,
                    name: '',
                    loading: false,
                })
                loadMobileData()
                setParam({
                    page: 0,
                    status: comfirmStatus[0].key
                })
            }
        })
    };

    //send code
    const [codeData, setCodeData] = useState({
        name: '',
        id: 0,
        captcha_val: '',
        captcha_key: '',
        captcha_src: '',
        loading: false,
    });
    const [codeError, setCodeError] = useState({
        captcha: '',
    });
    const doCode = function () {
        setCodeData({
            ...codeData,
            loading: true
        })
        mobileSendCode({
            mobile: codeData.name,
            captcha_code: codeData.captcha_val,
            captcha_key: codeData.captcha_key
        }).then((data) => {
            if (!data.status) {
                toast(data.message)
                setCodeError({
                    ...codeError,
                    ...data.field
                })
                let catpcha = {};
                if (data.field.captcha) {
                    catpcha = {
                        captcha_src: captchaSrc(captchaType + "/" + codeData.captcha_key, true)
                    }
                }
                setCodeData({
                    ...codeData,
                    loading: false,
                    ...catpcha
                })
            } else {
                setCodeData({
                    ...codeData,
                    loading: false,
                    captcha_val: '',
                    captcha_src: captchaSrc(captchaType + "/" + codeData.captcha_key, true)
                })
                showConfirmBox(codeData.id, codeData.name)
            }
        })
    };

    //confirm code
    const [confrimData, setConfrimData] = useState({
        name: '',
        id: 0,
        code: '',
        loading: false,
    });
    const [confrimError, setConfrimError] = useState({
        code: '',
    });
    const doConfirm = function () {
        setConfrimData({
            ...confrimData,
            loading: true
        })
        mobileConfirm({
            id: confrimData.id,
            code: confrimData.code,
        }).then((data) => {
            if (!data.status) {
                toast(data.message)
                setConfrimError({
                    ...confrimError,
                    ...data.field
                })
                setConfrimData({
                    ...confrimData,
                    loading: false
                })
            } else {
                setConfrimData({
                    ...confrimData,
                    loading: false,
                    code: ''
                })
                toast("完成绑定")
                loadMobileData()
                setParam({
                    status: comfirmStatus[1].key
                })
                setChangeBox(0)
            }
        })
    };


    const columns = [
        {
            field: 'id',
            label: 'ID',
            style: { width: 90 },
            align: "right",
        },
        {
            field: 'mobile',

            style: { width: 250 },
            label: '手机号',


        },
        {
            field: 'status',

            label: '状态',

            render: (params) => {
                return comfirmStatus.find((e) => { return e.key == params.status })?.val ?? "未知";
            }
        },
        {
            field: 'confirm_time',

            style: { width: 180 },
            label: '确认时间',

            render: (params) => {
                return showTime(params.confirm_time, "未确认")
            }
        },
        {
            field: 'change_time',

            style: { width: 180 },
            label: '添加时间',

            render: (params) => {
                return showTime(params.change_time, "未知")
            }
        },
        {
            label: '操作',

            align: "center",
            render: (params) => {
                let icon = [];
                if (params.status != 2) {
                    icon.push(
                        <IconButton title="发送验证短信" key={`${params.id}-send`} onClick={() => {
                            showCodeBox(params.id, params.mobile)
                        }} size='small'>
                            <SendIcon fontSize='small' />
                        </IconButton>)
                }
                icon.push(<ConfirmButton key={`${params.id}-del`}
                    message={`确定要删除手机号 [${params.mobile}] ?`}
                    onAction={() => {
                        return mobileDelete(params.id).then((res) => {
                            if (!res.status) return res;
                            let rows = loadData.data.filter((item) => {
                                if (item.id != params.id) return item;
                            })
                            setLoadData({
                                ...loadData,
                                data: rows
                            })
                            toast("删除完成");
                            if (rows.length == 0) {
                                loadMobileData()
                                setParam({
                                    page: param.get("page") - 1 >= 0 ? param.get("page") : 0,
                                });
                            }
                            return res;
                        });
                    }}
                    renderButton={(props) => {
                        return <IconButton title="删除" key={`${params.id}-mail`} {...props} size='small'>
                            <DeleteIcon fontSize='small' />
                        </IconButton>
                    }} />)
                return icon;
            }
        },
    ];

    let showBox
    switch (changeBoxState) {
        case 1:
            showBox = <AddBox
                title="添加手机号"
                type="text"
                label="手机号"
                placeholder="输入新手机号"
                button="添加"
                onSubmit={doAdd}
                onChange={(e) => {
                    setAddData({
                        ...addData,
                        name: e.target.value
                    })
                    setAddError({
                        ...addError,
                        name: ''
                    })
                }}
                name={addData.name}
                nameError={addError.name}
                loading={addData.loading}
            />;
            break
        case 2:
            showBox = <CodeBox
                label="手机号"
                title="添加手机号"
                button="添加"
                onSubmit={doCode}
                onChange={(e) => {
                    setCodeData({
                        ...codeData,
                        captcha_val: e.target.value
                    })
                    setCodeError({
                        ...codeError,
                        captcha: ''
                    })
                }}
                name={codeData.name}
                codeError={codeError.captcha}
                code={codeData.captcha_val}
                loading={codeData.loading}
                captchaSrc={codeData.captcha_src}
            />
            break
        case 3:
            showBox = <ConfirmBox
                label="手机号"
                title="确认手机验证码"
                button="确认"
                onSubmit={doConfirm}
                onChange={(e) => {
                    setConfrimData({
                        ...confrimData,
                        code: e.target.value
                    })
                    setConfrimError({
                        ...confrimError,
                        code: ''
                    })
                }}
                name={confrimData.name}
                codeError={confrimError.code}
                code={confrimData.code}
                loading={confrimData.loading}
                onBack={() => {
                    showCodeBox(confrimData.id, confrimData.name)
                }}
            />
            break
    };


    const [mobileStatus, setMobileStatus] = useState('');
    useEffect(() => {
        setMobileStatus(param.get("status"))
    }, [param])


    return <Fragment>
        <Drawer
            sx={{ zIndex: (theme) => theme.zIndex.drawer + 3 }}
            anchor={"right"}
            open={changeBoxState != 0}
            onClose={() => {
                setChangeBox(0)
            }}
        >
            <Box
                sx={{ width: 450 }}
                role="presentation"
            >
                {showBox}
            </Box>
        </Drawer>
        <Paper
            component="form"
            sx={{ p: 2, display: 'flex', alignItems: 'center', marginBottom: 1, marginTop: 1 }}
        >
            <FormControl sx={{ minWidth: 120, mr: 1 }} size="small"  >
                <InputLabel id="select-small">状态</InputLabel>
                <Select
                    labelId="select-small"
                    id="select-small"
                    label="状态"
                    onChange={(event) => {
                        setMobileStatus(event.target.value);
                    }}
                    value={mobileStatus ?? ''}
                >
                    <MenuItem value="">
                        全部
                    </MenuItem>
                    {
                        comfirmStatus.map((status) => {
                            return <MenuItem key={`status_${status.key}`} value={status.key}>{status.val}</MenuItem>
                        })
                    }
                </Select>
            </FormControl>
            <LoadingButton
                onClick={() => {
                    setParam({
                        page: 0,
                        status: mobileStatus ?? ''
                    }, loadMobileData)
                }}
                variant="outlined"
                size="medium"
                startIcon={<SearchIcon />}
                sx={{ mr: 1, p: "7px 15px", minWidth: 110 }}
                loading={loadData.loading}
                disabled={loadData.loading}
            >
                过滤
            </LoadingButton>
            <Button
                variant="outlined"
                size="medium"
                startIcon={<AddCircleOutlineIcon />}
                sx={{ mr: 1, p: "7px 15px" }}
                onClick={() => {
                    setChangeBox(1)
                }}>
                绑定新手机
            </Button>
        </Paper>

        {(loadData.status || loadData.loading)
            ? <Box sx={{ height: 1, width: '100%' }}>
                <DataPaginationTablePage
                    rows={(loadData.data ?? []).filter((item) => {
                        if (!param.get('status') || item.status == param.get('status')) return item;
                    })}
                    columns={columns}
                    page={param.get("page") || 0}
                    onPageChange={(e, newPage) => {
                        setParam({
                            page: newPage
                        }, loadMobileData)
                    }}
                    rowsPerPage={param.get("page_size") || 25}
                    onRowsPerPageChange={(e) => {
                        setParam({
                            page_size: e.target.value,
                            page: 0
                        }, loadMobileData)
                    }}
                    loading={loadData.loading}
                />
            </Box> : <Alert severity="error">{loadData.message}</Alert>}
    </Fragment>
}