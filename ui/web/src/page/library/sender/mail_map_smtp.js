

import { default as AddCircleOutlineIcon } from '@mui/icons-material/AddCircleOutline';
import DeleteIcon from '@mui/icons-material/Delete';
import SearchIcon from '@mui/icons-material/Search';
import { Alert, Button, Divider, Drawer, FormControl, Grid, IconButton, InputLabel, MenuItem, Paper, Select, TextField, Typography } from '@mui/material';
import Box from '@mui/material/Box';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { Form } from 'react-router-dom';
import { ToastContext } from '../../../context/toast';
import { ConfirmButton } from '../../../library/dialog';
import { ClearTextField, LoadSelect } from '../../../library/input';
import { LoadingButton } from '../../../library/loading';
import { SimpleTablePage } from '../../../library/table_page';
import { SenderTypeMail, mailAddAppSmtpConfig, mailAddSmtpConfig, mailDelAppSmtpConfig, mailListAppSmtpConfig, mailListSmtpConfig, } from '../../../rest/sender_setting';
import { showTime } from '../../../utils/utils';
import { tplsListConfig } from '../../../rest/sender_setting';


function AddBox(props) {
    const { onAdd, appId, appName, userId } = props;
    const { toast } = useContext(ToastContext);
    const [smtpConfigData, setsmtpConfigData] = useState({
        data: [],
        loading: false,
    });
    const [configData, setConfigData] = useState({
        smtp_config_id: '',
        name: '',
        tpl_id: '',
        from_email: '',
        subject_tpl_id: '',
        body_tpl_id: '',
        try_num: 1,
        loading: false,
    });
    const [addError, setAddError] = useState({
        smtp_config_id: '',
        name: '',
        tpl_id: '',
        from_email: '',
        subject_tpl_id: '',
        body_tpl_id: '',
        try_num: '',
    });



    const [tplData, setTplData] = useState({
        loading: false,
        next: true,
        page: 0,
        show: 25,
        items: [],
        item_ops: [],
        item_ops_cache: {},
        error: null
    });


    useEffect(() => {
        setsmtpConfigData({
            ...smtpConfigData,
            loading: true
        })
        mailListSmtpConfig({}).then((data) => {
            if (!data.status) {
                setsmtpConfigData({
                    ...smtpConfigData,
                    loading: false
                })
                toast(data.message);
                return
            }
            setsmtpConfigData({
                ...smtpConfigData,
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
        mailAddAppSmtpConfig({
            user_id: userId,
            app_id: appId,
            name: configData.name,
            smtp_config_id: configData.smtp_config_id,
            tpl_id: configData.tpl_id,
            from_email: configData.from_email,
            subject_tpl_id: configData.subject_tpl_id,
            body_tpl_id: configData.body_tpl_id,
            try_num: configData.try_num
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
                    from_email: '',
                    subject_tpl_id: '',
                    body_tpl_id: '',
                    try_num: 1,
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
                            <InputLabel size="small" id="res-select-label">选择SMTP服务器</InputLabel>
                            <Select
                                fullWidth
                                size='small'
                                value={configData.smtp_config_id}
                                onChange={
                                    (e) => {
                                        let ifind = smtpConfigData.data.find((item) => item.id == e.target.value)
                                        if (!ifind) return
                                        if ((!configData.from_email || (configData.from_email + '').length <= 0) && ifind.email) {
                                            setConfigData({
                                                ...configData,
                                                smtp_config_id: e.target.value,
                                                from_email: ifind.email
                                            })
                                        } else {
                                            setConfigData({
                                                ...configData,
                                                smtp_config_id: e.target.value
                                            })
                                        }
                                    }
                                }
                                labelId="config-select-small"
                                id="config-select-small"
                                label="选择SMTP服务器">
                                {smtpConfigData.data.map((item) => {
                                    return <MenuItem key={`smtp-config-${item.id}`} value={item.id}>{item.name}[{item.user}]</MenuItem>
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
                    </Grid>
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label={`来源邮箱`}
                            type="text"
                            size="small"
                            onChange={(e) => {
                                setConfigData({
                                    ...configData,
                                    from_email: e.target.value
                                })
                            }}
                            value={configData.from_email}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            error={!!addError.from_email}
                            helperText={addError.from_email}
                        />
                    </Grid>
                    <Grid item xs={10}>


                        <FormControl fullWidth sx={{
                            width: 1,
                            paddingBottom: 2
                        }}>
                            <InputLabel size="small" >主题模板</InputLabel>
                            <LoadSelect
                                label="主题模板"
                                size="small"
                                loading={tplData.loading}
                                next={tplData.next}
                                value={configData.subject_tpl_id}
                                error={tplData.error}
                                onChange={(e) => {
                                    let val = e.target.value;
                                    let res = tplData.items.find((e) => {
                                        return e.tpl_id == val
                                    })
                                    if (!res) {
                                        setConfigData({
                                            ...configData,
                                            subject_tpl_id: val
                                        })
                                        setTplData({ ...tplData, loading: false })
                                        return
                                    }
                                    let cache_item = tplData.item_ops_cache[res.id] ?? null;
                                    if (cache_item) {
                                        setTplData({
                                            ...tplData,
                                            loading: false,
                                            item_ops: cache_item.ops,
                                        })
                                        setConfigData({
                                            ...configData,
                                            subject_tpl_id: val
                                        })
                                        return;
                                    }
                                    tplsListConfig({
                                        user_id: userId,
                                        sender_type: SenderTypeMail,
                                        page: 0,
                                        page_size: 1
                                    }).then((data) => {
                                        if (!data.status) {
                                            setTplData({
                                                ...tplData,
                                                loading: false,
                                                error: tplData.items.length > 0 ? null : data.message
                                            })
                                            return;
                                        }
                                        let items = (data.data ?? [])[0];
                                        if (!items) return;
                                        let cache = { ...tplData.item_ops_cache };
                                        cache[items.id] = items;
                                        setTplData({
                                            ...tplData,
                                            item_ops: items.ops ?? [],
                                            item_ops_cache: cache,
                                            loading: false,
                                        })
                                        setConfigData({
                                            ...configData,
                                            subject_tpl_id: val
                                        })
                                    })
                                }}
                                onLoad={() => {
                                    setTplData({ ...tplData, loading: true })
                                    tplsListConfig({
                                        user_id: userId,
                                        sender_type: SenderTypeMail,
                                        page: tplData.page,
                                        page_size: tplData.show
                                    }).then((data) => {
                                        if (!data.status) {
                                            setTplData({
                                                ...tplData,
                                                loading: false,
                                                next: false,
                                                error: tplData.items.length > 0 ? null : data.message
                                            })
                                            return;
                                        }
                                        let items = (data.data ?? []).map((e) => {
                                            return e
                                        });
                                        setTplData({
                                            ...tplData,
                                            items: [...tplData.items, ...items],
                                            loading: false,
                                            page: tplData.page + 1,
                                            next: tplData.page * tplData.show < data.total
                                        })
                                    })
                                }}
                            >
                                {tplData.items.map((item) => {
                                    return <MenuItem key={`res-subtid-${item.tpl_id}`} value={item.tpl_id}>{item.tpl_id}</MenuItem>
                                })}
                            </LoadSelect>
                        </FormControl>
                        <FormControl fullWidth sx={{
                            width: 1,
                            paddingBottom: 2
                        }}>
                            <InputLabel size="small" >内容模板</InputLabel>
                            <LoadSelect
                                label="内容模板"
                                size="small"
                                loading={tplData.loading}
                                next={tplData.next}
                                value={configData.body_tpl_id}
                                error={tplData.error}
                                onChange={(e) => {
                                    let val = e.target.value;
                                    let res = tplData.items.find((e) => {
                                        return e.tpl_id == val
                                    })
                                    if (!res) {
                                        setConfigData({
                                            ...configData,
                                            body_tpl_id: val
                                        })
                                        setTplData({ ...tplData, loading: false })
                                        return
                                    }
                                    let cache_item = tplData.item_ops_cache[res.id] ?? null;
                                    if (cache_item) {
                                        setTplData({
                                            ...tplData,
                                            loading: false,
                                            item_ops: cache_item.ops,
                                        })
                                        setConfigData({
                                            ...configData,
                                            body_tpl_id: val
                                        })
                                        return;
                                    }
                                    tplsListConfig({
                                        user_id: userId,
                                        sender_type: SenderTypeMail,
                                        page: 0,
                                        page_size: 1
                                    }).then((data) => {
                                        if (!data.status) {
                                            setTplData({
                                                ...tplData,
                                                loading: false,
                                                error: tplData.items.length > 0 ? null : data.message
                                            })
                                            return;
                                        }
                                        let items = (data.data ?? [])[0];
                                        if (!items) return;
                                        let cache = { ...tplData.item_ops_cache };
                                        cache[items.id] = items;
                                        setTplData({
                                            ...tplData,
                                            item_ops: items.ops ?? [],
                                            item_ops_cache: cache,
                                            loading: false,
                                        })
                                        setConfigData({
                                            ...configData,
                                            body_tpl_id: val
                                        })
                                    })
                                }}
                                onLoad={() => {
                                    setTplData({ ...tplData, loading: true })
                                    tplsListConfig({
                                        user_id: userId,
                                        sender_type: SenderTypeMail,
                                        page: tplData.page,
                                        page_size: tplData.show
                                    }).then((data) => {
                                        if (!data.status) {
                                            setTplData({
                                                ...tplData,
                                                loading: false,
                                                next: false,
                                                error: tplData.items.length > 0 ? null : data.message
                                            })
                                            return;
                                        }
                                        let items = (data.data ?? []).map((e) => {
                                            return e
                                        });
                                        setTplData({
                                            ...tplData,
                                            items: [...tplData.items, ...items],
                                            loading: false,
                                            page: tplData.page + 1,
                                            next: tplData.page * tplData.show < data.total
                                        })
                                    })
                                }}
                            >
                                {tplData.items.map((item) => {
                                    return <MenuItem key={`body-tid-${item.tpl_id}`} value={item.tpl_id}>{item.tpl_id}</MenuItem>
                                })}
                            </LoadSelect>
                        </FormControl>
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
                                    try_num: val
                                })
                            }}
                            value={configData.try_num}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            error={!!addError.try_num}
                            helperText={addError.try_num}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <LoadingButton sx={{
                            width: 1,
                        }} variant="contained" type="submit">添加</LoadingButton>
                    </Grid>
                </Grid>
            </Form >
        </Fragment >)
}



export default function AppMailSmtpMap(props) {
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
                return row.config.tpl_id
            }
        },
        {
            style: { width: 160 },
            label: 'SMTP服务器',
            render: (row) => {
                return row.name + ' [' + row.user + ']'
            }
        },
        {

            style: { width: 160 },
            label: '来源邮箱',
            render: (row) => {
                return row.config.from_email;
            }
        },
        {

            style: { width: 120 },
            label: '主题模板',
            render: (row) => {
                return row.config.subject_tpl_id
            }
        },
        {

            style: { width: 120 },
            label: '内容模板',
            render: (row) => {
                return row.config.body_tpl_id
            }
        },
        {
            align: "center",
            style: { width: 120 },
            label: '尝试次数',
            render: (row) => {
                return row.config.max_try_num
            }
        },
        {
            style: { width: 180 },
            label: '更新时间',
            render: (row) => {
                return showTime(row.config.change_time, "未知")
            }
        },
        {

            label: '操作',
            render: (row) => {
                let delAction = () => {
                    return mailDelAppSmtpConfig({ id: row.config.id }).then((data) => {
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
        window.scrollTo({ top: 0, left: 0, behavior: 'smooth' });
        return mailListAppSmtpConfig({
            id: mapId,
            user_id: parseInt(userId),
            app_id: (props.children && !appId) ? -1 : appId,
            page: page || 0,
            page_size: pageSize || 25
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
                sx={{ mr: 1, p: "7px 15px", minWidth: 85 }}
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
                <SimpleTablePage
                    rows={loadData.data}
                    columns={columns}
                    count={loadData.total}
                    page={page || 0}
                    onPageChange={(e, newPage) => {
                        onSearchChange({
                            page: newPage
                        }, loadAppData)
                    }}
                    rowsPerPage={pageSize || 25}
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


