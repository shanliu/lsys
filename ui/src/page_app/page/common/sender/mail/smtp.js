

import { default as AddCircleOutlineIcon } from '@mui/icons-material/AddCircleOutline';
import DeleteIcon from '@mui/icons-material/Delete';
import EditIcon from '@mui/icons-material/Edit';
import SearchIcon from '@mui/icons-material/Search';
import { Alert, Button, Divider, Drawer, FormControl, Grid, IconButton, InputLabel, MenuItem, Paper, Select, Table, TableContainer, TextField, Typography } from '@mui/material';
import Box from '@mui/material/Box';
import { Fragment, default as React, useContext, useEffect, useState } from 'react';
import { Form } from 'react-router-dom';
import { UserSessionContext } from '../../../../../common/context/session';
import { BaseTableBody, BaseTableHead } from '../../../../../library/table_page';
import { mailAddSmtpConfig, mailDelSmtpConfig, mailEditSmtpConfig } from '../../../../../common/rest/sender_setting';
import { useSearchChange } from '../../../../../common/utils/hook';
import { ToastContext } from '../../../../../common/context/toast';
import { ConfirmButton } from '../../../../../common/ui/dialog';
import { ClearTextField, LoadSelect } from '../../../../../library/input';
import { LoadingButton } from '../../../../../library/loading';
import { SenderTypeMail, mailAddAppSmtpConfig, mailListSmtpConfig, tplsListConfig } from '../../../../../common/rest/sender_setting';
import { showTime } from '../../../../../common/utils/utils';


//smtp 配置

function SystemAppMailSettingSmtpBox(props) {
    const {
        rowData,
        onFinish
    } = props;

    const { toast } = useContext(ToastContext);

    let [addData, setAddData] = useState({
        name: rowData ? rowData.name : '',
        host: rowData ? rowData.host : '',
        port: rowData ? rowData.port : '25',
        timeout: rowData ? rowData.timeout : '60',
        user: rowData ? rowData.user : '',
        email: rowData ? rowData.email : '',
        password: rowData ? rowData.password : '',
        tls_domain: rowData ? rowData.tls_domain : '',
        loading: false,
    });
    const [addError, setAddError] = useState({
        name: '',
        host: '',
        port: '',
        timeout: '',
        user: '',
        email: '',
        password: '',
        tls_domain: '',
    });

    let onSubmit = () => {
        setAddData({
            ...addData,
            loading: true
        })
        if (rowData && rowData.id) {

            mailEditSmtpConfig({
                id: rowData.id,
                name: addData.name,
                host: addData.host,
                port: addData.port,
                timeout: addData.timeout,
                user: addData.user,
                email: addData.email,
                password: addData.password,
                tls_domain: addData.tls_domain,
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
                        host: '',
                        port: '',
                        timeout: '',
                        user: '',
                        password: '',
                        tls_domain: '',
                    })
                    setAddData({
                        ...addData,
                        loading: false
                    })
                    onFinish(rowData.id);
                }
            })
        } else {
            mailAddSmtpConfig({
                name: addData.name,
                host: addData.host,
                port: addData.port,
                timeout: addData.timeout,
                user: addData.user,
                email: addData.email,
                password: addData.password,
                tls_domain: addData.tls_domain,
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
                        host: '',
                        port: '',
                        timeout: '',
                        user: '',
                        password: '',
                        tls_domain: '',
                    })
                    setAddData({
                        name: '',
                        host: '',
                        port: '',
                        timeout: '60',
                        user: '',
                        password: '',
                        tls_domain: '',
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
                邮件发送服务器配置
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
                            label="smtp host"
                            type="text"
                            name="host"
                            size="small"
                            onChange={(e) => {
                                setAddData({
                                    ...addData,
                                    host: e.target.value
                                })
                            }}
                            value={addData.host}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            disabled={addData.loading}
                            error={!!addError.host}
                            helperText={addError.host}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label="smtp 端口"
                            type="text"
                            name="port"
                            size="small"
                            onChange={(e) => {
                                setAddData({
                                    ...addData,
                                    port: e.target.value
                                })
                            }}
                            value={addData.port}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            disabled={addData.loading}
                            error={!!addError.port}
                            helperText={addError.port}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label="连接超时[秒]"
                            type="text"
                            name="timeout"
                            size="small"
                            onChange={(e) => {
                                setAddData({
                                    ...addData,
                                    timeout: e.target.value
                                })
                            }}
                            value={addData.timeout}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            disabled={addData.loading}
                            error={!!addError.timeout}
                            helperText={addError.timeout}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label="smtp 用户名[不使用留空]"
                            type="text"
                            name="user"
                            size="small"
                            onChange={(e) => {
                                setAddData({
                                    ...addData,
                                    user: e.target.value
                                })
                            }}
                            value={addData.user}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}

                            disabled={addData.loading}
                            error={!!addError.user}
                            helperText={addError.user}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label="smtp 密码[不使用留空]"
                            type="text"
                            name="password"
                            size="small"
                            onChange={(e) => {
                                setAddData({
                                    ...addData,
                                    password: e.target.value
                                })
                            }}
                            value={addData.password}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}

                            disabled={addData.loading}
                            error={!!addError.password}
                            helperText={addError.password}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label="邮箱地址"
                            type="text"
                            name="email"
                            size="small"
                            onChange={(e) => {
                                setAddData({
                                    ...addData,
                                    email: e.target.value
                                })
                            }}
                            value={addData.email}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            disabled={addData.loading}
                            error={!!addError.email}
                            helperText={addError.email}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label="TLS 验证域名[不使用留空]"
                            type="text"
                            name="tls_domain"
                            size="small"
                            onChange={(e) => {
                                setAddData({
                                    ...addData,
                                    tls_domain: e.target.value
                                })
                            }}
                            value={addData.tls_domain}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            disabled={addData.loading}
                            error={!!addError.tls_domain}
                            helperText={addError.tls_domain}
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


export default function SystemAppMailSettingSmtpPage(props) {
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

            style: { width: 150 },
            label: '主机信息',
            render: (row) => {
                return row.host + ":" + row.port + (row.tls_domain ? "[Tls]" : "")
            }
        },
        {
            style: { width: 180 },
            label: '发送账号',
            render: (row) => {
                let name = row.user ? row.user : row.email;
                return <Box>
                    <Box>{name + (row.password ? (":" + row.hide_password) : "")}</Box>
                    <Box>{row.email ? `邮箱:${row.email}` : ""}</Box>
                </Box>
            }
        },
        {
            style: { width: 80 },
            label: '超时',
            align: "right",
            render: (row) => {
                return row.timeout
            }
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
                    return mailDelSmtpConfig({ id: row.id }).then((data) => {
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
        return mailListSmtpConfig({
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
            showBox = <SystemAppMailSettingSmtpBox
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
            showBox = <SystemAppMailSettingSmtpBox
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
        <Box sx={{ m: 2 }}>
            <Paper
                sx={{ p: 2, display: 'flex', alignItems: 'center', mb: 2 }}
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
                <TableContainer component={Paper} >
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



//关联发送


export function AppMailTplConfigSmtpShowDetail(row) {
    return <Box>
        <Box>{`Smtp端口:${row.setting_name}`}</Box>
        <Box>  {(row.config_data?.from_email && row.config_data?.from_email.length > 0) ? `发送邮箱:${row.config_data?.from_email}` : ``}</Box>
        <Box>  {(row.config_data?.reply_email && row.config_data?.reply_email.length > 0) ? `回复邮箱:${row.config_data?.reply_email}` : ``}</Box>
        <Box>{`主题模板:${row.config_data?.subject_tpl_id} 内容模板:${row.config_data?.body_tpl_id}`}</Box>
    </Box>;
}

export function AppMailTplConfigSmtpAddBox(props) {
    const { onAdd, appId, userId } = props;
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
        reply_email: '',
        subject_tpl_id: '',
        body_tpl_id: '',
        loading: false,
    });
    const [addError, setAddError] = useState({
        smtp_config_id: '',
        name: '',
        tpl_id: '',
        from_email: '',
        subject_tpl_id: '',
        body_tpl_id: '',
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
            reply_email: configData.reply_email,
            subject_tpl_id: configData.subject_tpl_id,
            body_tpl_id: configData.body_tpl_id,
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
                    reply_email: '',
                    subject_tpl_id: '',
                    body_tpl_id: '',
                    loading: false,
                })
                onAdd(data.id)
            }
        })
    };


    return (
        <Grid item xs={10}>
            <Form method="post" onSubmit={(e) => {
                e.preventDefault();
                doAdd()
            }}>



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

                <TextField
                    variant="outlined"
                    label={`回复邮箱`}
                    type="text"
                    size="small"
                    onChange={(e) => {
                        setConfigData({
                            ...configData,
                            reply_email: e.target.value
                        })
                    }}
                    value={configData.reply_email}
                    sx={{
                        width: 1,
                        paddingBottom: 2
                    }}
                    error={!!addError.reply_email}
                    helperText={addError.reply_email}
                />

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

                <LoadingButton sx={{
                    width: 1,
                    mb: 4
                }} variant="contained" type="submit">添加</LoadingButton>
            </Form>
        </Grid>
    )
}