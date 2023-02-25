
import AddCircleOutlineIcon from '@mui/icons-material/AddCircleOutline';
import DeleteIcon from '@mui/icons-material/Delete';
import SearchIcon from '@mui/icons-material/Search';
import { Alert, Button, Dialog, DialogActions, DialogContent, DialogContentText, DialogTitle, FormControl, InputLabel, MenuItem, Paper, Select } from '@mui/material';
import Box from '@mui/material/Box';
import { DataGrid, GridActionsCellItem } from '@mui/x-data-grid';
import { toDataURL } from 'qrcode';
import randomString from 'random-string';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { ToastContext } from '../../../context/toast';
import { ConfirmButton } from '../../../library/dialog';
import { LoadingButton, Progress } from '../../../library/loading';
import { oauthAdd, oauthCheck, OauthConfig, oauthDelete, oauthList, OauthType } from '../../../rest/user';
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
            headerName: 'ID',
            width: 90,
            type: 'number',
        },
        {
            field: 'external_type',
            width: 80,
            headerName: '类型',

            valueGetter: (params) => {
                let item = OauthType.find((e) => { return e.key == params.row.external_type });
                return item ? item.val : params.row.external_type
            }
        },
        {
            field: 'external_id',
            width: 140,
            headerName: '外部ID',
            sortable: false
        },
        {
            field: 'external_name',
            width: 140,
            headerName: '外部账号',

            valueGetter: (params) => {
                if (params.row.external_nikename.length > 0) {
                    return params.row.external_name + '(' + params.row.external_nikename + ')';
                }
                return params.row.external_name
            }
        },
        {
            field: 'add_time',
            width: 180,
            headerName: '添加时间',
            sortable: false,
            valueGetter: (params) => {
                return showTime(params.row.add_time, "未知")
            }
        },
        {
            field: 'change_time',
            width: 180,
            headerName: '最后登陆时间',
            sortable: false,
            valueGetter: (params) => {
                return showTime(params.row.change_time, "未知")
            }
        },
        {
            field: 'token_data',
            width: 100,
            sortable: false,
            headerName: '登陆Token',
            align: "left"
        },
        {
            headerName: '操作',
            type: 'actions',
            field: "actions",
            align: "center",
            getActions: (params) => {
                return [
                    <ConfirmButton
                        message={`确定要解除账号 ${params.row.external_name}[${params.row.external_id}] 的绑定?`}
                        onAction={() => {
                            return oauthDelete(params.row.id).then((res) => {
                                if (!res.status) return res;
                                let rows = loadData.data.filter((item) => {
                                    if (item.id != params.row.id) return item;
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
                            return <GridActionsCellItem {...props} icon={<DeleteIcon />} label="解除绑定" />
                        }
                        } />]

            }
        },
    ];

    //添加跟更新
    const { toast } = useContext(ToastContext);

    const [searchParam, setSearchParam] = useSearchChange({
        oauth_type: "",
    });
    const [filterData, setfilterData] = useState({
        oauth_type: searchParam.get("oauth_type"),
    })

    const LoadOauthData = () => {
        setLoadData({
            ...loadData,
            loading: true
        })
        oauthList(searchParam.get("oauth_type")).then((data) => {
            setLoadData({
                ...loadData,
                ...data,
                loading: false
            })
        })
    }
    useEffect(() => {
        setfilterData({
            ...filterData,
            oauth_type: searchParam.get("oauth_type"),
        })
        LoadOauthData();
    }, [searchParam])


    const execFilterData = () => {
        setSearchParam({
            ...filterData
        }, LoadOauthData)
    }



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
                    execFilterData()
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
                        }}>请稍后...</span>}
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
                onClick={execFilterData}
                variant="outlined"
                size="medium"
                startIcon={<SearchIcon />}
                sx={{ mr: 1, p: "7px 15px" }}
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
                < DataGrid
                    sx={{
                        "&.MuiDataGrid-root .MuiDataGrid-cell:focus-within": {
                            outline: "none !important",
                        },
                        "&.MuiDataGrid-root .MuiDataGrid-columnHeader:focus-within": {
                            outline: "none !important",
                        },
                    }}
                    rows={loadData.data}
                    columns={columns}
                    autoHeight={true}
                    autoPageSize={true}
                    pageSize={10}
                    disableColumnFilter={true}
                    disableColumnMenu={true}
                    disableSelectionOnClick={true}
                    rowCount={loadData.data.length}
                    editMode="cell"
                    loading={loadData.loading}
                />
            </Box> : <Alert severity="error">{loadData.message}</Alert>}
    </Fragment>
}