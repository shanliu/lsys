
import { default as AddCircleOutlineIcon } from '@mui/icons-material/AddCircleOutline';
import EditIcon from '@mui/icons-material/Edit';
import KeyIcon from '@mui/icons-material/Key';
import SmsIcon from '@mui/icons-material/Sms';
import MailIcon from '@mui/icons-material/Mail';
import SearchIcon from '@mui/icons-material/Search';
import LogoDevIcon from '@mui/icons-material/LogoDev';
import { Alert, Button, Dialog, DialogActions, DialogContent, DialogContentText, Divider, Drawer, FormControl, Grid, IconButton, InputLabel, MenuItem, Paper, Select, TextField, Typography } from '@mui/material';
import Box from '@mui/material/Box';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { Form, useNavigate } from 'react-router-dom';
import { UserSessionContext } from '../../context/session';
import { ToastContext } from '../../context/toast';
import { ConfirmButton } from '../../library/dialog';
import { ClearTextField } from '../../library/input';
import { LoadingButton, Progress } from '../../library/loading';
import { BaseTablePage } from '../../library/table_page';
import { ItemTooltip } from '../../library/tips';
import { appAdd, appEdit, appList, resetSecretApp, viewSecretApp } from '../../rest/app';
import { useSearchChange } from '../../utils/hook';
import { showTime } from '../../utils/utils';
import { PageNav } from './menu';
const filterStatus = {
    status: [
        { key: 1, val: '审核中' },
        { key: 2, val: '已审核' },
        { key: -1, val: '已禁用' },
    ],
};

export function AddBox(props) {
    const { onAdd, userId } = props;
    const { toast } = useContext(ToastContext);
    const [appData, setAppData] = useState({
        name: '',
        client_id: '',
        domain: '',
        loading: false,
    });
    const [addError, setAddError] = useState({
        name: '',
        client_id: '',
        domain: '',
    });
    const doAdd = function () {
        setAppData({
            ...appData,
            loading: true
        })
        appAdd({
            user_id: userId,
            name: appData.name,
            client_id: appData.client_id,
            domain: appData.domain
        }).then((data) => {
            if (!data.status) {
                toast(data.message)
                setAddError({
                    ...addError,
                    ...data.field
                })
                setAppData({
                    ...appData,
                    loading: false,
                })
            } else {
                setAppData({
                    ...appData,
                    name: '',
                    client_id: '',
                    domain: '',
                    loading: false,
                })
                onAdd(appData.client_id)
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
                申请APP
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
                        <TextField
                            variant="outlined"
                            label={`名称`}
                            type="text"
                            size="small"
                            onChange={(e) => {
                                setAppData({
                                    ...appData,
                                    name: e.target.value
                                })
                            }}
                            value={appData.name}
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
                            label={`AppID`}
                            type="text"
                            size="small"
                            onChange={(e) => {
                                setAppData({
                                    ...appData,
                                    client_id: e.target.value
                                })
                            }}
                            value={appData.client_id}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            error={!!addError.client_id}
                            helperText={addError.client_id}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label={`回调域名`}
                            type="text"
                            size="small"
                            onChange={(e) => {
                                setAppData({
                                    ...appData,
                                    domain: e.target.value
                                })
                            }}
                            value={appData.domain}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            error={!!addError.domain}
                            helperText={addError.domain}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <LoadingButton sx={{
                            width: 1,
                        }} variant="contained" type="submit">提交申请</LoadingButton>
                    </Grid>
                </Grid>
            </Form >
        </Fragment>)
}


export function EditBox(props) {
    const { onEdit, loadData } = props;
    const { toast } = useContext(ToastContext);
    const [appData, setAppData] = useState({
        name: loadData.name,
        client_id: loadData.client_id,
        domain: loadData.callback_domain || '',
        loading: false,
    });
    const [addError, setAddError] = useState({
        name: '',
        client_id: '',
        domain: '',
    });
    const doEdit = function () {
        setAppData({
            ...appData,
            loading: true
        })
        appEdit({ appid: loadData.id, domain: appData.domain, name: appData.name, client_id: appData.client_id }).then((data) => {
            if (!data.status) {
                toast(data.message)
                setAddError({
                    ...addError,
                    ...data.field
                })
                setAppData({
                    ...appData,
                    loading: false,
                })
            } else {
                onEdit(appData.client_id)
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
                编辑APP
            </Typography>
            <Divider variant="middle" />
            <Form method="post" onSubmit={(e) => {
                e.preventDefault();
                doEdit()
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
                            label={`名称`}
                            type="text"
                            size="small"
                            onChange={(e) => {
                                setAppData({
                                    ...appData,
                                    name: e.target.value
                                })
                            }}
                            value={appData.name}
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
                            label={`AppID`}
                            type="text"
                            size="small"
                            onChange={(e) => {
                                setAppData({
                                    ...appData,
                                    client_id: e.target.value
                                })
                            }}
                            value={appData.client_id}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            error={!!addError.client_id}
                            helperText={addError.client_id}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label={`回调域名`}
                            type="text"
                            size="small"
                            onChange={(e) => {
                                setAppData({
                                    ...appData,
                                    domain: e.target.value
                                })
                            }}
                            value={appData.domain}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            error={!!addError.domain}
                            helperText={addError.domain}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <LoadingButton sx={{
                            width: 1,
                        }} variant="contained" type="submit">保存</LoadingButton>
                    </Grid>
                </Grid>
            </Form >
        </Fragment>)
}





export default function UserAppIndexPage(props) {
    const { toast } = useContext(ToastContext);
    const { userData } = useContext(UserSessionContext)
    let [loadData, setLoadData] = useState({
        status: false,
        message: null,
        loading: true,
        data: [],
        total: 0,
    });
    const navigate = useNavigate();
    const columns = [
        {
            field: 'id',
            label: 'ID',
            align: "right",
            style: { width: 80 }
        },
        {

            style: { width: 120 },
            label: '名称',
            render: (row) => {
                if (row.is_view_app) {
                    return <Fragment>
                        <ItemTooltip title={'此应用已被授权访问任意app信息'} placement="top"><span style={{ color: '#9d5432' }} >{row.name}</span></ItemTooltip>
                    </Fragment>
                } else {
                    return row.name;
                }
            }
        },
        {
            field: 'client_id',
            style: { width: 120 },
            label: 'AppID',
        },
        {
            style: { width: 280 },
            label: '回调域名',
            render: (row) => {
                if (!row.callback_domain || row.callback_domain == '') {
                    return "未配置";
                } else {
                    return row.callback_domain;
                }
            }
        },
        {
            style: { width: 90 },
            label: '审核状态',
            render: (row) => {
                if (row.status == 2) {
                    return <Fragment>
                        <ItemTooltip title={'审核时间:' + showTime(row.confirm_time, "未知")} placement="top"><span>已审核</span></ItemTooltip>
                    </Fragment>

                }
                return filterStatus.status.find((e) => { return e.key == row.status })?.val ?? '未知';
            }
        },
        {
            style: { width: 180 },
            label: '申请时间',
            render: (row) => {
                return showTime(row.add_time, "未知")
            }
        },
        {

            label: '管理',
            align: "center",
            render: (row) => {
                const [keyBox, setKeyBox] = useState({
                    open: false,
                    loading: false,
                    secret: '',
                    oauth_secret: ''
                });
                let button_add = [];
                button_add.push(<IconButton title="编辑 App 信息" key={`${row.id}-edit`} onClick={() => {
                    setAppEditData(row);
                    setChangeBox(2)
                }} size='small'>
                    <EditIcon fontSize='small' />
                </IconButton>);
                button_add.push(<IconButton title="查看 App Secret" key={`${row.id}-key`} onClick={() => {
                    setKeyBox({
                        ...keyBox,
                        open: true,
                        loading: true,
                    })
                    return viewSecretApp({ appid: row.id }).then((data) => {
                        if (!data.status) {
                            toast(data.message)
                            setKeyBox({
                                ...keyBox,
                                loading: false,
                                open: false,
                                secret: '',
                                oauth_secret: ''
                            })
                            return;
                        }
                        setKeyBox({
                            open: true,
                            loading: false,
                            secret: data.secret,
                            oauth_secret: data.oauth_secret,
                        })

                    })
                }} size='small'>
                    <KeyIcon fontSize='small' />
                </IconButton>);
                let resetBut;
                if (row.status == 2) {
                    if (row.is_sms) {
                        button_add.push(<IconButton title="应用功能-短信发送" key={`${row.id}-sms`} onClick={() => {
                            navigate("/user/sms/message?app_id=" + row.id);
                        }} size='small'>
                            <SmsIcon fontSize='small' />
                        </IconButton>);
                    }
                    if (row.is_mail) {
                        button_add.push(<IconButton title="应用功能-邮件发送" key={`${row.id}-mail`} onClick={() => {
                            navigate("/user/mail/message?app_id=" + row.id);
                        }} size='small'>
                            <MailIcon fontSize='small' />
                        </IconButton>);
                    }

                    let resetAction = () => {
                        return resetSecretApp({ appid: row.id }).then((data) => {
                            if (!data.status) {
                                return data;
                            }
                            setKeyBox({
                                ...keyBox,
                                open: true,
                                loading: false,
                                secret: data.secret,
                                oauth_secret: data.oauth_secret,
                            })
                            return data;
                        })
                    };
                    resetBut = <ConfirmButton
                        key={`${row.id}-reset`}
                        sx={{ marginLeft: 1 }}
                        message={`确定要重置应用 [${row.name}] 的 AppSecret 吗?`}
                        onAction={resetAction}
                        renderButton={(props) => {
                            return <Button {...props} >
                                重置
                            </Button>
                        }} />;
                }

                return <Fragment>
                    <Dialog
                        open={keyBox.open}
                        onClose={() => { setKeyBox({ ...keyBox, open: false, loading: false, data: '' }) }}
                    >
                        {keyBox.loading ? <DialogContent sx={{
                            minWidth: 350
                        }}>
                            <Progress />

                        </DialogContent> : <DialogContent sx={{
                            minWidth: 380
                        }}>
                            <DialogContentText>
                                应用 {row.name}
                                <br />
                                <span style={{
                                    display: "block",
                                    marginTop: "14px",
                                    background: " #eee",
                                    borderRadius: "3px",
                                    padding: "0 15px",
                                    lineHeight: "40px",
                                }}>App Secret : {keyBox.secret}</span>
                                <br />
                                <span style={{
                                    display: "block",
                                    background: " #eee",
                                    borderRadius: "3px",
                                    padding: "0 15px",
                                    lineHeight: "40px",
                                }}>OAuth Secret : {keyBox.oauth_secret}</span>
                            </DialogContentText>
                        </DialogContent>}
                        <DialogActions>
                            {resetBut}
                            <Button onClick={() => { setKeyBox({ ...keyBox, open: false, loading: false, data: '' }) }} >
                                关闭
                            </Button>
                        </DialogActions>
                    </Dialog>
                    {button_add.map((e) => e)}
                </Fragment>
            }
        },
    ];
    const [searchParam, setSearchParam] = useSearchChange({
        status: "",
        client_id: "",
        page: 0,
        page_size: 10,
    });
    const [filterData, setfilterData] = useState({
        status: searchParam.get("status"),
        client_id: searchParam.get("client_id")
    })
    const loadAppData = () => {
        setLoadData({
            ...loadData,
            loading: true
        })
        return appList({
            user_id: parseInt(userData.user_data.user_id),
            status: searchParam.get("status") ?? '',
            client_id: searchParam.get("client_id") ?? '',
            page: searchParam.get("page") || 0,
            page_size: searchParam.get("page_size") || 10,
            check_sms: true,
            check_mail: true,
            check_view_app: true,
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
            status: searchParam.get("status"),
            client_id: searchParam.get("client_id"),
        })
        loadAppData()
    }, [searchParam])


    const [appEditData, setAppEditData] = useState({});
    const [changeBoxState, setChangeBox] = useState(0);

    let showBox
    switch (changeBoxState) {
        case 1:
            showBox = <AddBox
                userId={parseInt(userData.user_data.user_id)}
                onAdd={(client_id) => {
                    setSearchParam({
                        status: "",
                        client_id: client_id,
                        page: 0
                    }, loadAppData)
                    setChangeBox(0)
                }} />;
            break;
        case 2:
            showBox = <EditBox
                loadData={appEditData}
                onEdit={(client_id) => {
                    setSearchParam({
                        status: "",
                        client_id: client_id,
                        page: 0
                    }, loadAppData)
                    setAppEditData({})
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
        <PageNav />
        <Paper
            sx={{ p: 2, display: 'flex', alignItems: 'center', marginBottom: 1, marginTop: 1 }}
        >
            <FormControl sx={{ minWidth: 110, mr: 1 }} size="small"  >
                <InputLabel id="select-type">审核状态</InputLabel>
                <Select
                    labelId="select-type"
                    id="select-type"
                    label="审核状态"
                    disabled={loadData.loading}
                    onChange={(event) => {
                        setfilterData({
                            ...filterData,
                            status: event.target.value
                        })
                    }}
                    value={filterData.status ?? ''}
                >
                    <MenuItem value="">
                        全部
                    </MenuItem>
                    {
                        filterStatus.status.map((status) => {
                            return <MenuItem key={`status_${status.key}`} value={status.key}>{status.val}</MenuItem>
                        })
                    }
                </Select>
            </FormControl>
            <FormControl sx={{ minWidth: 120, mr: 1 }} size="small"  >
                <ClearTextField
                    sx={{ mr: 1 }}
                    variant="outlined"
                    label={`APPID`}
                    type="text"
                    name="code"
                    value={filterData.client_id}
                    size="small"
                    disabled={loadData.loading}
                    onChange={(event, nval) => {
                        setfilterData({
                            ...filterData,
                            client_id: nval
                        })
                    }}
                />
            </FormControl>
            <LoadingButton
                onClick={() => {
                    setSearchParam({
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
            <Button
                variant="outlined"
                size="medium"
                startIcon={<AddCircleOutlineIcon />}
                sx={{ mr: 1, p: "7px 15px", minWidth: 130 }}
                onClick={() => {
                    setChangeBox(1)
                }}>
                申请新应用
            </Button>
            <Button
                variant="outlined"
                size="medium"
                startIcon={<LogoDevIcon />}
                sx={{ mr: 1, p: "7px 15px", minWidth: 120 }}
                onClick={() => {
                    https://github.com/shanliu/lsys/
                    window.open("https://github.com/shanliu/lsys/tree/main/sdk/go", "_blank")
                }}>
                接入示例
            </Button>
        </Paper>

        {(loadData.status || loadData.loading)
            ? <Box sx={{ height: 1, width: '100%' }}>
                <BaseTablePage
                    rows={loadData.data}
                    columns={columns}
                    count={loadData.total}
                    page={searchParam.get("page") || 0}
                    onPageChange={(e, newPage) => {
                        setSearchParam({
                            page: newPage
                        }, loadAppData)
                    }}
                    rowsPerPage={searchParam.get("page_size") || 10}
                    onRowsPerPageChange={(e) => {
                        setSearchParam({
                            page_size: e.target.value,
                            page: 0
                        }, loadAppData)
                    }}
                    loading={loadData.loading}
                />
            </Box> : <Alert severity="error">{loadData.message}</Alert>}
    </Fragment>
}


