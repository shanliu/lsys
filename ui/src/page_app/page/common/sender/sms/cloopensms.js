

import AddCircleOutlineIcon from '@mui/icons-material/AddCircleOutline';
import DeleteIcon from '@mui/icons-material/Delete';
import EditIcon from '@mui/icons-material/Edit';
import SearchIcon from '@mui/icons-material/Search';
import { Alert, Button, Divider, Drawer, FormControl, Grid, IconButton, InputLabel, MenuItem, Paper, Select, Table, TableContainer, TextField, Typography } from '@mui/material';
import Box from '@mui/material/Box';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { Form } from 'react-router-dom';
import { UserSessionContext } from '../../../../../common/context/session';
import { ToastContext } from '../../../../../common/context/toast';
import { ConfirmButton } from '../../../../../common/ui/dialog';
import { ClearTextField } from '../../../../../library/input';
import { LoadingButton } from '../../../../../library/loading';
import { BaseTableBody, BaseTableHead } from '../../../../../library/table_page';
import { smsAddCloOpenConfig, smsDelCloOpenConfig, smsEditCloOpenConfig, smsAddAppCloOpenConfig, smsListCloOpenConfig } from '../../../../../common/rest/sender_setting';
import { useSearchChange } from '../../../../../common/utils/hook';
import { showTime } from '../../../../../common/utils/utils';



//短信配置


// 配置编辑

function SystemAppSmsSettingCloOpensmsBox(props) {
    const {
        rowData,
        onFinish
    } = props;

    const { toast } = useContext(ToastContext);

    let [addData, setAddData] = useState({
        name: rowData ? rowData.name : '',
        account_sid: rowData ? rowData.account_sid : '',
        account_token: rowData ? rowData.account_token : '',
        sms_app_id: rowData ? rowData.sms_app_id : '',
        callback_key: rowData ? rowData.callback_key : '',
        limit: rowData ? rowData.limit : 50,
        loading: false,
    });
    const [addError, setAddError] = useState({
        name: '',
        account_sid: '',
        account_token: '',
        sms_app_id: '',
        callback_key: '',
        limit: '',
    });

    let onSubmit = () => {
        setAddData({
            ...addData,
            loading: true
        })
        if (rowData && rowData.id) {
            smsEditCloOpenConfig({
                id: rowData.id,
                name: addData.name,
                account_sid: addData.account_sid,
                account_token: addData.account_token,
                sms_app_id: addData.sms_app_id,
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
                        account_sid: '',
                        account_token: '',
                        callback_key: '',
                        sms_app_id: '',
                        limit: '1',
                    })
                    setAddData({
                        ...addData,
                        loading: false
                    })
                    onFinish(rowData.id);
                }
            })
        } else {
            smsAddCloOpenConfig({
                name: addData.name,
                account_sid: addData.account_sid,
                account_token: addData.account_token,
                sms_app_id: addData.sms_app_id,
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
                        account_sid: '',
                        account_token: '',
                        callback_key: '',
                        sms_app_id: '',
                        limit: '1',
                    })
                    setAddData({
                        name: '',
                        account_sid: '',
                        account_token: '',
                        loading: false,
                        callback_key: '',
                        sms_app_id: '',
                        limit: '1',
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
                阿里短信配置
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
                            label="account sid"
                            type="text"
                            name="name"
                            size="small"
                            onChange={(e) => {
                                setAddData({
                                    ...addData,
                                    account_sid: e.target.value
                                })
                            }}
                            value={addData.account_sid}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            disabled={addData.loading}
                            error={!!addError.account_sid}
                            helperText={addError.account_sid}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label=" account token"
                            type="text"
                            name="name"
                            size="small"
                            onChange={(e) => {
                                setAddData({
                                    ...addData,
                                    account_token: e.target.value
                                })
                            }}
                            value={addData.account_token}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            disabled={addData.loading}
                            error={!!addError.account_token}
                            helperText={addError.account_token}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label="应用ID"
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

export default function SystemAppSmsSettingCloOpensmsPage(props) {
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
            field: 'account_sid',
            style: { width: 120 },
            label: 'account sid',
        },
        {
            field: 'account_token',
            style: { width: 120 },
            label: 'account token',
        },
        {
            field: 'sms_app_id',
            style: { width: 120 },
            label: '应用ID',
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
            style: { width: 180 },
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
                    return smsDelCloOpenConfig({ id: row.id }).then((data) => {
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
        return smsListCloOpenConfig({
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
            showBox = <SystemAppSmsSettingCloOpensmsBox
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
            showBox = <SystemAppSmsSettingCloOpensmsBox
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





// 发送关联配置

export function AppSmsTplConfigCloOpenShowDetail(row) {
    return <Box>
        <Box>{`短信端口:${row.setting_name}`}</Box>
        <Box>{`融连云模板:${row.config_data?.template_id} `}</Box>
        {row.config_data?.template_map ? <Box>{`key映射:${row.config_data?.template_map}`}</Box> : null}
    </Box>;
}
export function AppSmsTplConfigCloOpenAddBox(props) {
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


        template_id: '',
        template_map: '',
        loading: false,
    });
    const [addError, setAddError] = useState({
        config_id: '',
        name: '',
        tpl_id: '',


        template_id: '',
        template_map: '',
    });

    useEffect(() => {
        setTplConfigData({
            ...tplConfigData,
            loading: true
        })
        smsListCloOpenConfig({}).then((data) => {
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
        smsAddAppCloOpenConfig({
            user_id: userId,
            app_id: appId,
            name: configData.name,
            tpl_id: configData.tpl_id,
            config_id: configData.config_id,

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
            <InputLabel size="small" id="res-select-label">选择短信端口配置</InputLabel>
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
                label="选择短信端口配置">
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
            label={`融连云模板`}
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