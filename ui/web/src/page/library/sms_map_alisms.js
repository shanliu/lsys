

import { default as AddCircleOutlineIcon } from '@mui/icons-material/AddCircleOutline';
import DeleteIcon from '@mui/icons-material/Delete';
import SearchIcon from '@mui/icons-material/Search';
import { Alert, Button, Divider, Drawer, FormControl, Grid, IconButton, InputLabel, MenuItem, Paper, Select, TextField, Typography } from '@mui/material';
import Box from '@mui/material/Box';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { Form } from 'react-router-dom';
import { ToastContext } from '../../context/toast';
import { ConfirmButton } from '../../library/dialog';
import { ClearTextField } from '../../library/input';
import { LoadingButton } from '../../library/loading';
import { BaseTablePage } from '../../library/table_page';
import { addAppAliConfig, delAppAliConfig, listAliConfig, listAppAliConfig } from '../../rest/sms_setting';
import { showTime } from '../../utils/utils';


function AddBox(props) {
    const { onAdd, appId, appName, userId } = props;
    const { toast } = useContext(ToastContext);
    const [aliConfigData, setAliConfigData] = useState({
        data: [],
        loading: false,
    });
    const [configData, setConfigData] = useState({
        aliconfig_id: '',
        name: '',
        sms_tpl: '',
        aliyun_sign_name: '',
        aliyun_sms_tpl: '',
        max_try_num: 1,
        loading: false,
    });
    const [addError, setAddError] = useState({
        aliconfig_id: '',
        name: '',
        sms_tpl: '',
        aliyun_sign_name: '',
        aliyun_sms_tpl: '',
        max_try_num: '',
    });

    useEffect(() => {
        setAliConfigData({
            ...aliConfigData,
            loading: true
        })
        listAliConfig({}).then((data) => {
            if (!data.status) {
                setAliConfigData({
                    ...aliConfigData,
                    loading: false
                })
                toast(data.message);
                return
            }
            setAliConfigData({
                ...aliConfigData,
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
        addAppAliConfig({
            user_id: userId,
            app_id: appId,
            name: configData.name,
            aliconfig_id: configData.aliconfig_id,
            sms_tpl: configData.sms_tpl,
            aliyun_sign_name: configData.aliyun_sign_name,
            aliyun_sms_tpl: configData.aliyun_sms_tpl,
            max_try_num: configData.max_try_num
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
                    sms_tpl: '',
                    aliyun_sign_name: '',
                    aliyun_sms_tpl: '',
                    max_try_num: 1,
                    loading: false,
                })
                onAdd(data.id)
            }
        })
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
                添加{appName}模板关联
            </Typography>
            <Divider variant="middle" />
            <Form method="post" onSubmit={(e) => {
                e.preventDefault();
                doAdd()
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
                        <FormControl fullWidth sx={{
                            width: 1,
                            paddingBottom: 2
                        }}>
                            <InputLabel size="small" id="res-select-label">选择阿里云端口</InputLabel>
                            <Select
                                fullWidth
                                size='small'
                                value={configData.aliconfig_id}
                                onChange={
                                    (e) => {
                                        setConfigData({
                                            ...configData,
                                            aliconfig_id: e.target.value
                                        })
                                    }
                                }
                                labelId="config-select-small"
                                id="config-select-small"
                                label="选择阿里云端口">
                                {aliConfigData.data.map((item) => {
                                    return <MenuItem value={item.id}>{item.name}[{item.app_id}]</MenuItem>
                                })}
                            </Select>
                        </FormControl>
                    </Grid>
                    <Grid item xs={10}>
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
                    </Grid>
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label={`模板名`}
                            type="text"
                            size="small"
                            onChange={(e) => {
                                setConfigData({
                                    ...configData,
                                    sms_tpl: e.target.value
                                })
                            }}
                            value={configData.sms_tpl}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            error={!!addError.sms_tpl}
                            helperText={addError.sms_tpl}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label={`阿里云签名`}
                            type="text"
                            size="small"
                            onChange={(e) => {
                                setConfigData({
                                    ...configData,
                                    aliyun_sign_name: e.target.value
                                })
                            }}
                            value={configData.aliyun_sign_name}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            error={!!addError.aliyun_sign_name}
                            helperText={addError.aliyun_sign_name}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label={`阿里云模板`}
                            type="text"
                            size="small"
                            onChange={(e) => {
                                setConfigData({
                                    ...configData,
                                    aliyun_sms_tpl: e.target.value
                                })
                            }}
                            value={configData.aliyun_sms_tpl}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            error={!!addError.aliyun_sms_tpl}
                            helperText={addError.aliyun_sms_tpl}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label={`失败尝试次数`}
                            type="number"
                            size="small"
                            onChange={(e) => {
                                let val = parseInt(e.target.value)
                                if (val <= 0) val = 1;
                                setConfigData({
                                    ...configData,
                                    max_try_num: val
                                })
                            }}
                            value={configData.max_try_num}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            error={!!addError.max_try_num}
                            helperText={addError.max_try_num}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <LoadingButton sx={{
                            width: 1,
                        }} variant="contained" type="submit">添加</LoadingButton>
                    </Grid>
                </Grid>
            </Form >
        </Fragment>)
}



export default function AppSmsAliSmsMap(props) {
    const {
        userId,
        appId,
        mapId,
        appName,
        page,
        pageSize,
        onSearchChange,
    } = props;
    let [loadData, setLoadData] = useState({
        status: false,
        message: null,
        loading: true,
        data: [],
        total: 0,
    });
    let columns = [
        {
            label: 'ID',
            align: "right",
            style: { width: 80 },
            render: (row) => {
                return row.config.id
            }
        },
        {
            field: "app_id",
            style: { width: 120 },
            label: '应用ID',
            render: (row) => {
                return row.config.app_id
            }
        },
        {
            style: { width: 120 },
            label: '名称',
            render: (row) => {
                return row.config.name
            }
        },
        {

            style: { width: 160 },
            label: '模板名',
            render: (row) => {
                return row.config.sms_tpl
            }
        },
        {
            style: { width: 160 },
            label: '阿里云端口',
            render: (row) => {
                return row.aliyun_name + ' (id:' + row.aliyun_id + ')'
            }
        },
        {

            style: { width: 160 },
            label: '阿里云签名',
            render: (row) => {
                return row.config.aliyun_sign_name
            }
        },
        {

            style: { width: 160 },
            label: '阿里云模板名',
            render: (row) => {
                return row.config.aliyun_sms_tpl
            }
        },
        {
            align: "center",
            style: { width: 100 },
            label: '尝试次数',
            render: (row) => {
                return row.config.max_try_num
            }
        },
        {
            style: { width: 180 },
            label: '添加时间',
            render: (row) => {
                return showTime(row.config.add_time, "未知")
            }
        },
        {

            label: '操作',
            render: (row) => {
                let delAction = () => {
                    return delAppAliConfig({ id: row.config.id }).then((data) => {
                        if (!data.status) return data;
                        let rows = loadData.data.filter((item) => {
                            if (item.config.id == row.config.id) return;
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
                    <ConfirmButton
                        message={`确定删除关系 [${row.config.id}] 吗?`}
                        onAction={delAction}
                        renderButton={(props) => {
                            return <IconButton  {...props} size='small' ><DeleteIcon fontSize="small" /></IconButton>
                        }} />
                </Fragment>
            }
        }
    ];

    if (!props.children) {
        columns = columns.filter((e) => { return e.field != 'app_id' })
    }

    const [filterData, setfilterData] = useState({
        ...{
            id: mapId,
        }, ...props.children ? { app_id: appId } : {}
    })
    const loadAppData = () => {
        setLoadData({
            ...loadData,
            loading: true
        })
        return listAppAliConfig({
            id: mapId,
            user_id: parseInt(userId),
            app_id: (props.children && !appId) ? -1 : appId,
            page: page || 0,
            page_size: pageSize || 10
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
            ...{
                ...filterData,
                id: mapId,
            }, ...props.children ? { app_id: appId } : {}
        })
        loadAppData()
    }, [props])
    const [changeBoxState, setChangeBox] = useState(0);
    let showBox
    switch (changeBoxState) {
        case 1:
            showBox = <AddBox
                userId={parseInt(userId)}
                appId={(props.children && !appId) ? -1 : appId}
                appName={appName}
                onAdd={(id) => {
                    onSearchChange({
                        id: id,
                        app_id: appId,
                        page: 0
                    }, loadAppData)
                    setChangeBox(0)
                }} />;
            break;
    };

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
            sx={{ p: 2, display: 'flex', alignItems: 'center', marginBottom: 1, marginTop: 1 }}
        >
            {props.children}
            <FormControl sx={{ minWidth: 120, mr: 1 }} size="small"  >
                <ClearTextField
                    sx={{ mr: 1 }}
                    variant="outlined"
                    label={`配置ID`}
                    type="text"
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
                    onSearchChange({
                        ...filterData,
                        page: 0
                    }, loadAppData)
                }}
                variant="outlined"
                size="medium"
                startIcon={<SearchIcon />}
                sx={{ mr: 1, p: "7px 15px" }}
                loading={loadData.loading}
                disabled={loadData.loading}
            >
                过滤
            </LoadingButton>
            {appName ? <Button
                variant="outlined"
                size="medium"
                startIcon={<AddCircleOutlineIcon />}
                sx={{ mr: 1, p: "7px 15px" }}
                onClick={() => {
                    setChangeBox(1)
                }}>
                添加
            </Button> : null}
        </Paper>

        {(loadData.status || loadData.loading)
            ? <Box sx={{ height: 1, width: '100%' }}>
                <BaseTablePage
                    rows={loadData.data}
                    columns={columns}
                    count={loadData.total}
                    page={page || 0}
                    onPageChange={(e, newPage) => {
                        onSearchChange({
                            page: newPage
                        }, loadAppData)
                    }}
                    rowsPerPage={pageSize || 10}
                    onRowsPerPageChange={(e) => {
                        onSearchChange({
                            page_size: e.target.value,
                            page: 0
                        }, loadAppData)
                    }}
                    loading={loadData.loading}
                />
            </Box> : <Alert severity="error">{loadData.message}</Alert>}
    </Fragment>
}


