

import { Drawer, FormControl, Grid, InputLabel, MenuItem, Paper, Select, TextField, Button, TableContainer, Table, Alert, Typography, Divider, IconButton } from '@mui/material';
import Box from '@mui/material/Box';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { Form } from 'react-router-dom';
import { ToastContext } from '../../../../../common/context/toast';
import { LoadingButton } from '../../../../../library/loading';
import { smsAddAppTenConfig, smsAddTenConfig, smsDelTenConfig, smsEditTenConfig, smsListTenConfig } from '../../../../../common/rest/sender_setting';
import { UserSessionContext } from '../../../../../common/context/session';
import { useSearchChange } from '../../../../../common/utils/hook';
import { ClearTextField } from '../../../../../library/input';
import DeleteIcon from '@mui/icons-material/Delete';
import EditIcon from '@mui/icons-material/Edit';
import SearchIcon from '@mui/icons-material/Search';
import AddCircleOutlineIcon from '@mui/icons-material/AddCircleOutline';
import { BaseTableBody, BaseTableHead } from '../../../../../library/table_page';
import { showTime } from '../../../../../common/utils/utils';
import { ConfirmButton } from '../../../../../common/ui/dialog';
//腾讯云短信配置页面

function SystemAppSmsSettingTensmsBox(props) {
    const {
        rowData,
        onFinish
    } = props;

    const { toast } = useContext(ToastContext);

    let [addData, setAddData] = useState({
        name: rowData ? rowData.name : '',
        secret_id: rowData ? rowData.secret_id : '',
        secret_key: rowData ? rowData.secret_key : '',
        region: rowData ? rowData.region : '',
        sms_app_id: rowData ? rowData.sms_app_id : '',
        callback_key: rowData ? rowData.callback_key : '',
        limit: rowData ? rowData.limit : 100,
        loading: false,
    });
    const [addError, setAddError] = useState({
        name: '',
        secret_id: '',
        secret_key: '',
        sms_app_id: '',
        region: '',
        callback_key: '',
        limit: '',
    });

    let onSubmit = () => {
        setAddData({
            ...addData,
            loading: true
        })
        if (rowData && rowData.id) {
            smsEditTenConfig({
                id: rowData.id,
                name: addData.name,
                secret_id: addData.secret_id,
                secret_key: addData.secret_key,
                sms_app_id: addData.sms_app_id,
                region: addData.region,
                callback_key: addData.callback_key,
                limit: parseInt(addData.limit),
            }).then((data) => {
                if (!data.status) {
                    toast(data.message)
                    setAddError({
                        ...addError,
                        ...data.field
                    })
                    setAddData({
                        ...addData,
                        loading: false
                    })
                } else {
                    setAddError({
                        name: '',
                        secret_id: '',
                        secret_key: '',
                        callback_key: '',
                        sms_app_id: '',
                        region: '',
                        limit: '100',
                    })
                    setAddData({
                        ...addData,
                        loading: false
                    })
                    onFinish(rowData.id);
                }
            })
        } else {
            smsAddTenConfig({
                name: addData.name,
                secret_id: addData.secret_id,
                secret_key: addData.secret_key,
                sms_app_id: addData.sms_app_id,
                region: addData.region,
                callback_key: addData.callback_key,
                limit: parseInt(addData.limit),
            }).then((data) => {
                if (!data.status) {
                    toast(data.message)
                    setAddError({
                        ...addError,
                        ...data.field
                    })
                    setAddData({
                        ...addData,
                        loading: false
                    })
                } else {
                    setAddError({
                        name: '',
                        callback_key: '',
                        region: '',
                        limit: '100',
                        sms_app_id: '',
                        secret_id: '',
                        secret_key: '',
                    })
                    setAddData({
                        name: '',
                        callback_key: '',
                        region: '',
                        sms_app_id: '',
                        limit: '100',
                        secret_id: '',
                        secret_key: '',
                        loading: false
                    })
                    onFinish(data.id);
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
                腾讯云云短信配置
            </Typography>
            <Divider variant="middle" />
            <Form method="post" onSubmit={(e) => {
                e.preventDefault();
                onSubmit()
            }}>
                <Grid
                    sx={{
                        mt: 5,
                    }}
                    container
                    justifyContent="center"
                    alignItems="center"
                >
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label="配置名"
                            type="text"
                            name="name"
                            size="small"
                            onChange={(e) => {
                                setAddData({
                                    ...addData,
                                    name: e.target.value
                                })
                            }}
                            value={addData.name}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            disabled={addData.loading}
                            error={!!addError.name}
                            helperText={addError.name}
                        />
                    </Grid>

                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label="腾讯云 secret id"
                            type="text"
                            name="name"
                            size="small"
                            onChange={(e) => {
                                setAddData({
                                    ...addData,
                                    secret_id: e.target.value
                                })
                            }}
                            value={addData.secret_id}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            disabled={addData.loading}
                            error={!!addError.secret_id}
                            helperText={addError.secret_id}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label="腾讯云 secret key"
                            type="text"
                            name="name"
                            size="small"
                            onChange={(e) => {
                                setAddData({
                                    ...addData,
                                    secret_key: e.target.value
                                })
                            }}
                            value={addData.secret_key}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            disabled={addData.loading}
                            error={!!addError.secret_key}
                            helperText={addError.secret_key}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label="短信应用ID"
                            type="text"
                            name="sms_app_id"
                            size="small"
                            onChange={(e) => {
                                setAddData({
                                    ...addData,
                                    sms_app_id: e.target.value
                                })
                            }}
                            value={addData.sms_app_id}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            disabled={addData.loading}
                            error={!!addError.sms_app_id}
                            helperText={addError.sms_app_id}
                        />
                    </Grid>

                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label="区域"
                            type="text"
                            name="region"
                            size="small"
                            onChange={(e) => {
                                setAddData({
                                    ...addData,
                                    region: e.target.value
                                })
                            }}
                            value={addData.region}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            disabled={addData.loading}
                            error={!!addError.region}
                            helperText={addError.region}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label="回调KEY,为空不校验"
                            type="text"
                            name="callback_key"
                            size="small"
                            onChange={(e) => {
                                setAddData({
                                    ...addData,
                                    callback_key: e.target.value
                                })
                            }}
                            value={addData.callback_key}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}

                            disabled={addData.loading}
                            error={!!addError.callback_key}
                            helperText={addError.callback_key}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label="单次发送量"
                            type="number"
                            name="limit"
                            size="small"
                            onChange={(e) => {
                                setAddData({
                                    ...addData,
                                    limit: e.target.value
                                })
                            }}
                            value={addData.limit}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            disabled={addData.loading}
                            error={!!addError.limit}
                            helperText={addError.limit}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <LoadingButton sx={{
                            width: 1,
                        }} variant="contained" type="submit" loading={addData.loading} disabled={addData.loading} >{rowData ? "修改" : "添加"}</LoadingButton>
                    </Grid>
                </Grid>
            </Form ></Fragment>)
}


export default function SystemAppSmsSettingTensmsPage(props) {
    const { userData } = useContext(UserSessionContext)
    let [loadData, setLoadData] = useState({
        status: false,
        message: null,
        loading: true,
        data: [],
        total: 0,
    });

    const columns = [
        {
            field: 'id',
            label: 'ID',
            align: "right",
            style: { width: 90 }
        },
        {
            field: 'name',
            style: { width: 100 },
            label: '配置名',
        },
        {
            field: 'secret_id',
            style: { width: 120 },
            label: 'secret id',
        },
        {
            field: 'secret_key',
            style: { width: 120 },
            label: 'secret key',
        },
        {
            field: 'region',
            style: { width: 120 },
            label: '区域',
        },
        {
            field: 'sms_app_id',
            style: { width: 120 },
            label: '短信应用ID',
        },

        {
            field: 'callback_url',
            style: { width: 120 },
            label: '回调地址',
        },
        {
            field: 'limit',
            style: { width: 100 },
            label: '单次发送量',
        },
        {

            style: { width: 100 },
            label: '用户ID',
            align: "center",
            render: (row) => {
                return row.change_user_id
            }
        },
        {
            style: { width: 120 },
            label: '更新时间',
            render: (row) => {
                return showTime(row.change_time, "未知")
            }
        },
        {
            label: '操作',
            align: "center",
            render: (row) => {
                let delAction = () => {
                    return smsDelTenConfig({ id: row.id }).then((data) => {
                        if (!data.status) return data;
                        let rows = loadData.data.filter((item) => {
                            if (item.id == row.id) return null;
                            return item;
                        })
                        setLoadData({
                            ...loadData,
                            data: rows
                        })
                        return data;
                    })
                };
                return <Fragment>
                    <IconButton size='small' onClick={() => {
                        setChangeBox({ data: row, show: 2 })
                    }}>
                        <EditIcon fontSize="small" />
                    </IconButton>
                    <ConfirmButton
                        message={`确定删除配置 [${row.name}] 吗?`}
                        onAction={delAction}
                        renderButton={(props) => {
                            return <IconButton  {...props} size='small' ><DeleteIcon fontSize="small" /></IconButton>
                        }} />
                </Fragment>
            }
        },
    ];
    const [searchParam, setSearchParam] = useSearchChange({
        id: "",
    });
    const [filterData, setfilterData] = useState({
        id: searchParam.get("id")
    })
    const loadConfigData = () => {
        setLoadData({
            ...loadData,
            loading: true
        })
        window.scrollTo({ top: 0, left: 0, behavior: 'smooth' });
        return smsListTenConfig({
            id: searchParam.get("id"),
            full_data: true
        }).then((data) => {

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
            id: searchParam.get("id")
        })
        loadConfigData()
    }, [searchParam])


    //添加跟更新
    const [changeBoxState, setChangeBox] = useState({
        show: 0,
        data: {}
    });
    let showBox
    switch (changeBoxState.show) {
        case 1:
            showBox = <SystemAppSmsSettingTensmsBox
                onFinish={(id) => {
                    setChangeBox({ data: {}, show: 0 })
                    setSearchParam({
                        ...filterData,
                        id: id
                    }, loadConfigData)
                }}
            />;
            break
        case 2:
            showBox = <SystemAppSmsSettingTensmsBox
                rowData={changeBoxState.data}
                onFinish={(id) => {
                    setChangeBox({ data: {}, show: 0 })
                    setSearchParam({
                        ...filterData,
                        id: id
                    }, loadConfigData)
                }}
            />;
            break
    };



    return <Fragment>
        <Drawer
            sx={{ zIndex: (theme) => theme.zIndex.drawer + 3 }}
            anchor={"right"}
            open={changeBoxState.show != 0}
            onClose={() => {
                setChangeBox({ data: {}, show: 0 })
            }}
        >
            <Box
                sx={{ width: 450 }}
                role="presentation"
            >
                {showBox}
            </Box>
        </Drawer>
        <Box
            sx={{ m: 2 }}
        >
            <Paper
                sx={{ p: 2, display: 'flex', alignItems: 'center', marginBottom: 1 }}
            >
                <FormControl sx={{ minWidth: 80, mr: 1 }} size="small"  >
                    <ClearTextField
                        sx={{ mr: 1 }}
                        variant="outlined"
                        label={`ID`}
                        type="text"
                        name="code"
                        value={filterData.id}
                        size="small"
                        disabled={loadData.loading}
                        onChange={(event, nval) => {
                            setfilterData({
                                ...filterData,
                                id: nval
                            })
                        }}
                    />
                </FormControl>
                <LoadingButton
                    onClick={() => {
                        setSearchParam({
                            ...filterData,
                        }, loadConfigData)
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
                    sx={{ mr: 1, p: "7px 15px", minWidth: 115 }}
                    onClick={() => {
                        setChangeBox({ data: {}, show: 1 })
                    }}>
                    新增配置
                </Button>
            </Paper>

            {(loadData.status || loadData.loading)
                ?
                <TableContainer component={Paper}>
                    <Table>
                        <BaseTableHead
                            columns={columns}
                        />
                        <BaseTableBody
                            columns={columns}
                            loading={loadData.loading}
                            rows={loadData.data ?? []}
                        />
                    </Table>
                </TableContainer>
                : <Alert severity="error">{loadData.message}</Alert>}
        </Box>
    </Fragment>
}


//发送关联配置


export function AppSmsTplConfigTenShowDetail(row) {
    return <Box>
        <Box>{`腾讯云短信端口:${row.setting_name}`}</Box>
        <Box>{`腾讯云模板:${row.config_data?.template_id} 腾讯云签名:${row.config_data?.sign_name} `}</Box>
        {row.config_data?.template_map ? <Box>{`key映射:${row.config_data?.template_map}`}</Box> : null}
    </Box>;
}
export function AppSmsTplConfigTenAddBox(props) {
    const { onAdd, appId, userId } = props;
    const { toast } = useContext(ToastContext);
    const [tplConfigData, setTplConfigData] = useState({
        data: [],
        loading: false,
    });
    const [configData, setConfigData] = useState({
        config_id: '',
        name: '',
        tpl_id: '',
        sign_name: '',

        template_id: '',
        template_map: '',
        loading: false,
    });
    const [addError, setAddError] = useState({
        config_id: '',
        name: '',
        tpl_id: '',
        sign_name: '',

        template_id: '',
        template_map: '',
    });

    useEffect(() => {
        setTplConfigData({
            ...tplConfigData,
            loading: true
        })
        smsListTenConfig({}).then((data) => {
            if (!data.status) {
                setTplConfigData({
                    ...tplConfigData,
                    loading: false
                })
                toast(data.message);
                return
            }

            setTplConfigData({
                ...tplConfigData,
                data: data.data,
                loading: false
            })
        })
    }, []);



    const doAdd = function () {
        setConfigData({
            ...configData,
            loading: true
        })
        smsAddAppTenConfig({
            user_id: userId,
            app_id: appId,
            name: configData.name,
            tpl_id: configData.tpl_id,
            config_id: configData.config_id,
            sign_name: configData.sign_name,
            template_id: configData.template_id,
            template_map: configData.template_map,

        }).then((data) => {
            if (!data.status) {
                toast(data.message)
                setAddError({
                    ...addError,
                    ...data.field
                })
                setConfigData({
                    ...configData,
                    loading: false,
                })
            } else {
                setConfigData({
                    ...configData,
                    tpl_id: '',
                    sign_name: '',

                    template_id: '',
                    template_map: '',
                    loading: false,
                })
                onAdd(data.id)
            }
        })
    };

    return (<Grid item xs={10} ><Form method="post" onSubmit={(e) => {
        e.preventDefault();
        doAdd()
    }}>
        <FormControl fullWidth sx={{
            width: 1,
            paddingBottom: 2
        }}>
            <InputLabel size="small" id="res-select-label">选择腾讯云短信端口配置</InputLabel>
            <Select
                fullWidth
                size='small'
                value={configData.config_id ?? ''}
                onChange={
                    (e) => {
                        setConfigData({
                            ...configData,
                            config_id: e.target.value
                        })
                    }
                }
                labelId="config-select-small"
                id="config-select-small"
                label="选择腾讯云短信端口配置">
                {tplConfigData.data.map((item) => {
                    return <MenuItem key={`s-${item.app_id}`} value={item.id}>{item.name}[{item.app_id}]</MenuItem>
                })}
            </Select>
        </FormControl>

        <TextField
            variant="outlined"
            label={`名称`}
            type="text"
            size="small"
            onChange={(e) => {
                setConfigData({
                    ...configData,
                    name: e.target.value
                })
            }}
            value={configData.name}
            sx={{
                width: 1,
                paddingBottom: 2
            }}
            required
            error={!!addError.name}
            helperText={addError.name}
        />

        <TextField
            variant="outlined"
            label={`模板名`}
            type="text"
            size="small"
            onChange={(e) => {
                setConfigData({
                    ...configData,
                    tpl_id: e.target.value
                })
            }}
            value={configData.tpl_id}
            sx={{
                width: 1,
                paddingBottom: 2
            }}
            required
            error={!!addError.tpl_id}
            helperText={addError.tpl_id}
        />

        <TextField
            variant="outlined"
            label={`腾讯云签名`}
            type="text"
            size="small"
            onChange={(e) => {
                setConfigData({
                    ...configData,
                    sign_name: e.target.value
                })
            }}
            value={configData.sign_name ?? ''}
            sx={{
                width: 1,
                paddingBottom: 2
            }}
            required
            error={!!addError.sign_name}
            helperText={addError.sign_name}
        />

        <TextField
            variant="outlined"
            label={`腾讯云模板`}
            type="text"
            size="small"
            onChange={(e) => {
                setConfigData({
                    ...configData,
                    template_id: e.target.value
                })
            }}
            value={configData.template_id ?? ''}
            sx={{
                width: 1,
                paddingBottom: 2
            }}
            required
            error={!!addError.template_id}
            helperText={addError.template_id}
        />

        <TextField
            variant="outlined"
            label={`模板位置映射(逗号分割)`}
            type="text"
            size="small"
            onChange={(e) => {
                setConfigData({
                    ...configData,
                    template_map: e.target.value
                })
            }}
            value={configData.template_map ?? ''}
            sx={{
                width: 1,
                paddingBottom: 2
            }}

            error={!!addError.template_map}
            helperText={addError.template_map}
        />

        <LoadingButton sx={{
            width: 1,
            mb: 4
        }} variant="contained" type="submit">添加</LoadingButton>
    </Form >
    </Grid >

    )
}