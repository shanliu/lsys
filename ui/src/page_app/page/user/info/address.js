import AddCircleOutlineIcon from '@mui/icons-material/AddCircleOutline';
import DeleteIcon from '@mui/icons-material/Delete';
import EditIcon from '@mui/icons-material/Edit';
import { Alert, Button, Divider, Drawer, IconButton, Paper, Stack, TextField, Typography } from '@mui/material';
import Box from '@mui/material/Box';

import React, { Fragment, useContext, useEffect, useState } from 'react';
import { ToastContext } from '../../../../common/context/toast';
import { ConfirmButton } from '../../../../common/ui/dialog';
import { DataPaginationTablePage } from '../../../../library/table_page';
import { AddressAdd, AddressDelete, AddressEdit, AddressList } from '../../../../common/rest/user';
import { useSearchChange } from '../../../../common/utils/hook';
import { AddressSelect } from '../../common/address';
import { Form } from 'react-router-dom';
import { LoadingButton } from '../../../../library/loading';
import { showTime } from '../../../../common/utils/utils';
import { ItemTooltip } from '../../../../library/tips';


function AddressBox(props) {
    const { toast } = useContext(ToastContext);
    const { onFinish, row } = props;
    // add
    const [sendData, setSendData] = useState({
        name: '',
        code: '',
        info: '',
        detail: '',
        mobile: '',
        loading: false,
    });
    useEffect(() => {
        if (row && row.id) {
            setSendData({
                name: row.name,
                code: row.address_code,
                info: row.address_info,
                detail: row.address_detail,
                mobile: row.mobile,
            })
        } else {
            setSendData({
                name: '',
                code: '',
                info: '',
                detail: '',
                mobile: '',
            })
        }
    }, [row])
    const [sendError, setSendError] = useState({
        name: '',
        code: '',
        info: '',
        detail: '',
        mobile: '',
    });
    const doSave = function () {
        setSendData({
            ...sendData,
            loading: true
        })
        let param = {
            code: sendData.code,
            name: sendData.name,
            info: sendData.info,
            detail: sendData.detail,
            mobile: sendData.mobile,
        }
        if (row && row.id) {
            param.id = row.id;
            AddressEdit(param).then((data) => {
                if (!data.status) {
                    toast(data.message)
                    setSendData({
                        ...sendData,
                        loading: false,
                    })
                    setSendError({
                        ...sendError,
                        ...data.field
                    })
                } else {
                    setSendData({
                        ...sendData,
                        loading: false,
                    })
                    setSendError({
                        name: '',
                        code: '',
                        info: '',
                        detail: '',
                        mobile: '',
                    })
                    onFinish()
                }
            })
        } else {
            AddressAdd(param).then((data) => {
                if (!data.status) {
                    toast(data.message)
                    setSendData({
                        ...sendData,
                        loading: false,
                    })
                    setSendError({
                        ...sendError,
                        ...data.field
                    })
                } else {
                    setSendData({
                        ...sendData,
                        name: '',
                        code: '',
                        info: '',
                        detail: '',
                        mobile: '',
                        loading: false,
                    })
                    setSendError({
                        name: '',
                        code: '',
                        info: '',
                        detail: '',
                        mobile: '',
                    })
                    onFinish()
                }
            })
        }
    };
    return (
        <Fragment>
            <Typography
                align="center"
                variant="subtitle1"
                noWrap
                sx={{
                    mt: 5,
                    mb: 2,
                    fontWeight: 100,
                    alignItems: "center",
                    letterSpacing: '.3rem',
                    color: 'inherit',
                    textDecoration: 'none',
                }}
            >
                {row ? "编辑" : "新增"}地址
            </Typography>
            <Divider variant="middle" />
            <Form method="post" onSubmit={(e) => {
                e.preventDefault();
                doSave();
            }}>
                <Stack
                    justifyContent="center"
                    alignItems="center"
                    spacing={1}
                    sx={{
                        m: 3,
                    }}
                >
                    <AddressSelect
                        onChange={(code, info) => {
                            setSendData({
                                ...sendData,
                                info: info,
                                code: code
                            })
                        }}
                        label={"选择地址"}
                        type="text"
                        addrCode={sendData.code + ''}
                        addrInfo={sendData.info + ''}
                        size={"small"}
                        sx={{
                            width: 1,
                            mb: 1
                        }} />
                    <TextField
                        variant="outlined"
                        label={"输入详细地址"}
                        type="text"
                        name="name"
                        size="small"
                        onChange={(e) => {
                            let val = e.target.value ?? '';
                            val = val.replace(/^\s+/, '');
                            val = val.replace(/\s+$/, '');
                            setSendData({
                                ...sendData,
                                detail: val,
                            })
                        }}
                        value={sendData.detail}
                        sx={{
                            width: 1,
                            paddingBottom: 1
                        }}
                        required
                        disabled={sendData.loading}
                        error={!!sendError.detail}
                        helperText={sendError.detail}
                    />
                    <TextField
                        variant="outlined"
                        label={"收货人姓名"}
                        type="text"
                        name="name"
                        size="small"
                        onChange={(e) => {
                            let val = e.target.value ?? '';
                            val = val.replace(/^\s+/, '');
                            val = val.replace(/\s+$/, '');
                            setSendData({
                                ...sendData,
                                name: val,
                            })
                        }}
                        value={sendData.name}
                        sx={{
                            width: 1,
                            paddingBottom: 1
                        }}
                        required
                        disabled={sendData.loading}
                        error={!!sendError.name}
                        helperText={sendError.name}
                    />
                    <TextField
                        variant="outlined"
                        label={"收货人电话"}
                        type="number"
                        name="name"
                        size="small"
                        onChange={(e) => {
                            let val = e.target.value ?? '';
                            val = val.replace(/^\s+/, '');
                            val = val.replace(/\s+$/, '');
                            setSendData({
                                ...sendData,
                                mobile: val,
                            })
                        }}
                        value={sendData.mobile}
                        sx={{
                            width: 1,
                            paddingBottom: 1
                        }}
                        required
                        disabled={sendData.loading}
                        error={!!sendError.mobile}
                        helperText={sendError.mobile}
                    />
                    <LoadingButton sx={{
                        width: 1,
                    }} variant="contained" type="submit" loading={sendData.loading} disabled={sendData.loading} > 保存</LoadingButton>
                </Stack>
            </Form ></Fragment>)
}

export default function UserAddressPage(props) {
    //列表数据
    let [loadData, setLoadData] = useState({
        loading: true,
        status: false,
        data: [],
        message: null,
    });
    const [param, setParam] = useSearchChange({
        page: 0,
        page_size: 25
    });
    const loadAddressData = () => {
        setLoadData({
            ...loadData,
            loading: true
        })
        window.scrollTo({ top: 0, left: 0, behavior: 'smooth' });
        AddressList().then((data) => {
            setLoadData({
                ...loadData,
                ...data,
                data: data.status ? data.data : [],
                loading: false
            })
        })
    };
    useEffect(loadAddressData, [])

    //添加跟更新
    const [changeBoxState, setChangeBox] = useState({
        key: null,
        row: null
    });
    const { toast } = useContext(ToastContext);

    const columns = [
        {
            field: 'id',
            label: 'ID',
            style: { width: 90 },
            align: "right",
        },
        {
            style: { width: 240 },
            label: '收货地区',
            render: (row) => {
                return <ItemTooltip title={`地区编码:${row.address_code}`} placement="top">
                    <span>{row.address_info}</span>
                </ItemTooltip>;
            }
        },
        {
            field: 'address_detail',
            style: { width: 180 },
            label: '地址详细',
        },
        {
            style: { width: 180 },
            label: '收货人信息',
            render: (params) => {
                return `${params.name}[${params.mobile}]`
            }
        },
        {

            style: { width: 180 },
            label: '更改时间',
            render: (row) => {
                return showTime(row.change_time, "未知")
            }
        },
        {
            label: '操作',
            align: "center",
            render: (params) => {
                return <Fragment>
                    <IconButton title="修改" onClick={() => {
                        setChangeBox({
                            key: "edit",
                            row: params
                        })
                    }} size='small'>
                        <EditIcon fontSize='small' />
                    </IconButton>
                    <ConfirmButton key={`${params.id}-del`}
                        message={`确定要删除此收货地址?`}
                        onAction={() => {
                            return AddressDelete(params.id).then((res) => {
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
                                    loadAddressData()
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
                        }} />
                </Fragment>

            }
        },
    ];

    let showBox
    switch (changeBoxState.key) {
        case "add":
            showBox = <AddressBox
                onFinish={() => {
                    setParam({
                        page: 0
                    }, loadAddressData)
                }}
            />;
            break
        case "edit":
            showBox = <AddressBox
                row={changeBoxState.row}
                onFinish={() => {
                    loadAddressData()
                }}
            />;
            break
    };

    return <Fragment>
        <Drawer
            sx={{ zIndex: (theme) => theme.zIndex.drawer + 3 }}
            anchor={"right"}
            open={!!changeBoxState.key}
            onClose={() => {
                setChangeBox({
                    key: null,
                    row: null
                })
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
            <Button
                variant="outlined"
                size="medium"
                startIcon={<AddCircleOutlineIcon />}
                sx={{ mr: 1, p: "7px 15px" }}
                onClick={() => {
                    setChangeBox({
                        key: "add",
                        row: null
                    })
                }}>
                绑定新地址
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
                        }, loadAddressData)
                    }}
                    rowsPerPage={param.get("page_size") || 25}
                    onRowsPerPageChange={(e) => {
                        setParam({
                            page_size: e.target.value,
                            page: 0
                        }, loadAddressData)
                    }}
                    loading={loadData.loading}
                />
            </Box> : <Alert severity="error">{loadData.message}</Alert>}
    </Fragment>
}