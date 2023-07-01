
import AddCircleOutlineIcon from '@mui/icons-material/AddCircleOutline';
import DeleteIcon from '@mui/icons-material/Delete';
import SearchIcon from '@mui/icons-material/Search';
import { Alert, Button, Dialog, DialogActions, DialogContent, DialogContentText, DialogTitle, FormControl, IconButton, InputLabel, MenuItem, Paper, Select } from '@mui/material';
import Box from '@mui/material/Box';
import { toDataURL } from 'qrcode';
import randomString from 'random-string';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { ToastContext } from '../../../context/toast';
import { ConfirmButton } from '../../../library/dialog';
import { LoadingButton, Progress } from '../../../library/loading';
import { DataPaginationTablePage } from '../../../library/table_page';
import { OauthConfig, OauthType, oauthAdd, oauthCheck, oauthDelete, oauthList } from '../../../rest/user';
import { useSearchChange } from '../../../utils/hook';
import { showTime } from '../../../utils/utils';
export default function UserOauthPage(props) {
    //列表数据
    let [loadData, setLoadData] = useState({
        loading: true,
        status: false,
        data: [],
        message: null,
    });

    const columns = [
        {
            field: 'id',
            label: 'ID',

            style: { width: 90 },
            align: "right"
        },
        {

            style: { width: 90 },
            label: '类型',

            render: (params) => {
                let item = OauthType.find((e) => { return e.key == params.external_type });
                return item ? item.val : params.external_type
            }
        },
        {
            field: 'external_id',
            style: { width: 140 },
            label: '外部ID',

        },
        {

            style: { width: 140 },
            label: '外部账号',

            render: (params) => {
                if (params.external_nikename.length > 0) {
                    return params.external_name + '(' + params.external_nikename + ')';
                }
                return params.external_name
            }
        },
        {
            style: { width: 180 },
            label: '最后登陆时间',

            render: (params) => {
                return showTime(params.change_time, "未知")
            }
        },
        {
            field: 'token_data',
            width: 100,
            style: { width: 100 },
            label: '登陆Token',
            align: "left"
        },
        {
            label: '操作',
            align: "center",
            render: (params) => {
                return [
                    <ConfirmButton key={`${params.id}-del`}
                        message={`确定要解除账号 ${params.external_name}[${params.external_id}] 的绑定?`}
                        onAction={() => {
                            return oauthDelete(params.id).then((res) => {
                                if (!res.status) return res;
                                let rows = loadData.data.filter((item) => {
                                    if (item.id != params.id) return item;
                                })
                                setLoadData({
                                    ...loadData,
                                    data: rows
                                })
                                toast("解除完成");
                                if (rows.length == 0) {
                                    LoadOauthData();
                                }
                                return res;
                            });
                        }}
                        renderButton={(props) => {
                            return <IconButton title="解除绑定" key={`${params.id}-oauth`} {...props} size='small'>
                                <DeleteIcon fontSize='small' />
                            </IconButton>

                        }
                        } />]

            }
        },
    ];

    //添加跟更新
    const { toast } = useContext(ToastContext);

    const [searchParam, setSearchParam] = useSearchChange({
        oauth_type: "",
        page: 0,
        page_size: 25,
    });
    const [filterData, setfilterData] = useState({
        oauth_type: searchParam.get("oauth_type"),
    })

    const LoadOauthData = () => {
        setLoadData({
            ...loadData,
            loading: true
        })
        window.scrollTo({ top: 0, left: 0, behavior: 'smooth' });
        oauthList(searchParam.get("oauth_type")).then((data) => {

            setLoadData({
                ...loadData,
                ...data,
                data: data.status ? data.data : [],
                loading: false
            })
        })
    }
    useEffect(() => {
        setfilterData({
            ...filterData,
            oauth_type: searchParam.get("oauth_type"),
        })

    }, [searchParam])
    useEffect(() => {
        LoadOauthData();
    }, [])




    const [addOauth, setAddOauth] = useState({
        loading: false,
        load_type: null,
    })
    const addOauthOpen = (item) => {
        setAddOauth({
            load_type: item.key,
            loading: true
        })
        if (item.type == 'qrcode') {
            setQrData({
                ...qrData,
                open: true,
                title: item.val,
                state: randomString()
            })
        } else {
            oauthAdd(key).then((data) => {
                setAddOauth({
                    load_type: '',
                    loading: false
                })
                if (!data.status) {
                    toast(data.message);
                    return
                }
                open(data.url, "_blank");
            })
        }
    }
    const [qrData, setQrData] = useState({
        open: false,
        title: '',
        src: null,
        state: null,
        loading: false
    })
    const [qrCheck, setQrCheck] = useState({
        hand: null,
        checking: false
    })
    useEffect(() => {
        if (!qrData.open) {
            qrCheck.hand && clearTimeout(qrCheck.hand);
            return
        }
        if (qrCheck.hand) return
        let hand = setTimeout(() => {
            setQrCheck({
                hand: hand,
                checking: true
            })
            oauthCheck(addOauth.load_type, qrData.state).then((data) => {
                if (data.id) {
                    setAddOauth({
                        load_type: '',
                        loading: false
                    })
                    setQrCheck({
                        hand: null,
                        checking: false
                    })
                    setQrData({
                        ...qrData,
                        open: false,
                    })
                    setfilterData({
                        ...filterData,
                        oauth_type: addOauth.load_type
                    })
                    LoadOauthData()
                    setSearchParam({
                        ...filterData,
                        page: 0,
                    })
                    return
                }
                if (!data.status) {
                    toast(data.message);
                }
                if (data.reload || !data.status) {
                    setQrCheck({
                        hand: null,
                        checking: false,
                    })
                    setQrData({
                        ...qrData,
                        state: randomString()
                    })
                } else {
                    setQrCheck({
                        hand: null,
                        checking: false,
                    })
                }
            })
        }, 2000);
        setQrCheck({
            checking: false,
            hand: hand
        })
    }, [qrData.open, qrCheck.hand, qrData.state, addOauth.load_type])


    useEffect(() => {
        if (!qrData.open) return
        setQrData({
            ...qrData,
            loading: true
        })
        oauthAdd(addOauth.load_type, qrData.state).then((data) => {
            if (!data.status) {
                setQrData({
                    ...qrData,
                    open: false,
                    loading: false
                })
                toast(data.message);
                return
            }
            toDataURL(data.url, function (err, url) {
                if (err) {
                    toast('二维码生成错误:' + err + '')
                    return;
                } else {
                    setQrData({
                        ...qrData,
                        src: url,
                        loading: false
                    })
                }
            })
        })
    }, [qrData.open, qrData.state, addOauth.load_type])


    return <Fragment>
        <Dialog
            open={qrData.open}
        >
            {
                qrData.loading ? <DialogTitle> 登陆二维码加载中</DialogTitle> :
                    <DialogTitle> 使用 {qrData.title} 扫描二维码完成绑定</DialogTitle>
            }
            <DialogContent sx={{
                minWidth: 450
            }}>
                {
                    qrData.loading ? <Progress /> : <DialogContentText sx={{ textAlign: "center" }}>
                        <img src={qrData.src} />
                        {qrCheck.checking ? <span style={{
                            display: "block",
                            fontSize: "0.5em"
                        }}>检测登录中</span> : <span style={{
                            display: "block",
                            fontSize: "0.5em"
                        }}>请用微信扫码...</span>}
                    </DialogContentText>
                }
            </DialogContent>
            <DialogActions>
                <Button onClick={() => {
                    setQrData({
                        ...qrData,
                        open: false,
                        src: null,
                        loading: false
                    })
                    setAddOauth({
                        load_type: '',
                        loading: false
                    })
                }} >
                    取消登陆
                </Button>
            </DialogActions>
        </Dialog>
        <Paper
            component="form"
            sx={{ p: 2, display: 'flex', alignItems: 'center', marginBottom: 1, marginTop: 1 }}
        >
            <FormControl sx={{ minWidth: 120, mr: 1 }} size="small"  >
                <InputLabel id="select-small">登陆类型</InputLabel>
                <Select
                    labelId="select-small"
                    id="select-small"
                    label="登陆类型"
                    onChange={(event) => {
                        setfilterData({
                            ...filterData,
                            oauth_type: event.target.value
                        })
                    }}
                    value={filterData.oauth_type ?? ''}
                >
                    <MenuItem value="">
                        全部
                    </MenuItem>
                    {
                        OauthType.map((status) => {
                            return <MenuItem key={`status_${status.key}`} value={status.key}>{status.val}</MenuItem>
                        })
                    }
                </Select>
            </FormControl>
            <Button
                onClick={() => {
                    setSearchParam({
                        ...filterData
                    }, LoadOauthData)
                }}
                variant="outlined"
                size="medium"
                startIcon={<SearchIcon />}
                sx={{ mr: 1, p: "7px 15px", minWidth: 110 }}
            >
                过滤
            </Button>
            {
                OauthConfig.map((item) => {
                    return <LoadingButton
                        variant="outlined"
                        size="medium"
                        loading={addOauth.load_type == item.key}
                        disabled={addOauth.loading}
                        key={`add${item.key}`}
                        startIcon={<AddCircleOutlineIcon />}
                        sx={{ mr: 1, p: "7px 15px" }}
                        onClick={() => {
                            addOauthOpen(item)
                        }}>
                        绑定{item.val}
                    </LoadingButton>
                })
            }

        </Paper>

        {(loadData.status || loadData.loading)
            ? <Box sx={{ height: 1, width: '100%' }}>
                <DataPaginationTablePage
                    rows={(loadData.data ?? []).filter((item) => {
                        if (!searchParam.get('status') || item.status == searchParam.get('status')) return item;
                    })}
                    columns={columns}
                    page={searchParam.get("page") || 0}
                    onPageChange={(e, newPage) => {
                        setSearchParam({
                            page: newPage
                        }, LoadOauthData)
                    }}
                    rowsPerPage={searchParam.get("page_size") || 25}
                    onRowsPerPageChange={(e) => {
                        setSearchParam({
                            page_size: e.target.value,
                            page: 0
                        }, LoadOauthData)
                    }}
                    loading={loadData.loading}
                />
            </Box> : <Alert severity="error">{loadData.message}</Alert>}
    </Fragment>
}