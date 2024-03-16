
import AddCircleOutlineIcon from '@mui/icons-material/AddCircleOutline';
import HighlightOffIcon from '@mui/icons-material/HighlightOff';
import ControlPointIcon from '@mui/icons-material/ControlPoint';
import AllOutIcon from '@mui/icons-material/AllOut';
import EditIcon from '@mui/icons-material/Edit';
import KeyIcon from '@mui/icons-material/Key';
import SmsIcon from '@mui/icons-material/Sms';
import AppsIcon from '@mui/icons-material/Apps';
import MailIcon from '@mui/icons-material/Mail';
import PersonIcon from '@mui/icons-material/Person';
import SearchIcon from '@mui/icons-material/Search';
import PinIcon from '@mui/icons-material/Pin';
import LogoDevIcon from '@mui/icons-material/LogoDev';
import { Alert, Button, CircularProgress, Dialog, DialogActions, DialogContent, DialogContentText, DialogTitle, Divider, Drawer, FormControl, Grid, IconButton, InputLabel, Link, MenuItem, Paper, Select, Stack, Switch, TextField, Typography } from '@mui/material';
import Box from '@mui/material/Box';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { Form, useNavigate } from 'react-router-dom';
import { UserSessionContext } from '../../../common/context/session';
import { ToastContext } from '../../../common/context/toast';
import { ConfirmButton } from '../../../common/ui/dialog';
import { ClearTextField, LoadSelect } from '../../../library/input';
import { LoadingButton, Progress } from '../../../library/loading';
import { BaseTableNoRows, SimpleTablePage } from '../../../library/table_page';
import { ItemTooltip } from '../../../library/tips';
import { appAdd, appEdit, appList, delParentApp, listParentApp, listSubApp, listSubUser, resetSecretApp, setParentApp, setSubUser, userParentApp, viewSecretApp } from '../../../common/rest/app';
import { useSearchChange } from '../../../common/utils/hook';
import { showTime } from '../../../common/utils/utils';
import { PageNav } from './menu';
import { UserSearchInput } from '../common/user';
const filterStatus = {
    status: [
        { key: 1, val: '审核中' },
        { key: 2, val: '已审核' },
        { key: -1, val: '已禁用' },
    ],
};




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

                return row.name;

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
                return showTime(row.change_time, "未知")
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
                    setAppNowData(row);
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
                    button_add.push(<IconButton title="关联应用用户管理" key={`${row.id}-sub-user`} onClick={() => {
                        setAppNowData(row);
                        setChangeBox(3)
                    }} size='small'>
                        <PersonIcon fontSize='small' />
                    </IconButton>);
                    button_add.push(<IconButton title="可用关联应用" key={`${row.id}-sub-app`} onClick={() => {
                        setAppNowData(row);
                        setChangeBox(4)
                    }} size='small'>
                        <AppsIcon fontSize='small' />
                    </IconButton>);
                    button_add.push(<IconButton title="关联应用授权" key={`${row.id}-parent-app`} onClick={() => {
                        setAppNowData(row);
                        setChangeBox(5)
                    }} size='small'>
                        <PinIcon fontSize='small' />
                    </IconButton>);
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
        page_size: 25,
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
        window.scrollTo({ top: 0, left: 0, behavior: 'smooth' });
        return appList({
            user_id: parseInt(userData.user_data.user_id),
            status: searchParam.get("status") ?? '',
            client_id: searchParam.get("client_id") ?? '',
            page: searchParam.get("page") || 0,
            page_size: searchParam.get("page_size") || 25,
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


    const [appNowData, setAppNowData] = useState({});
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
                loadData={appNowData}
                onFinish={(client_id) => {
                    setSearchParam({
                        status: "",
                        client_id: client_id,
                        page: 0
                    }, loadAppData)
                    setAppNowData({})
                    setChangeBox(0)
                }} />;
            break;
        case 3:
            showBox = <SubUserListBox
                loadData={appNowData}
            />;
            break;
        case 4:
            showBox = <SubAppListBox
                loadData={appNowData}
            />;
            break;
        case 5:
            showBox = <ParentAppListBox
                loadData={appNowData}
            />;
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
                sx={{ width: [1, 2].includes(changeBoxState) ? 450 : ([5].includes(changeBoxState) ? 850 : 700) }}
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
                    window.open("https://github.com/shanliu/lsys/tree/main/sdk/go", "_blank")
                }}>
                示例代码
            </Button>
            <Button
                variant="outlined"
                size="medium"
                startIcon={<LogoDevIcon />}
                sx={{ mr: 1, p: "7px 15px", minWidth: 120 }}
                onClick={() => {
                    window.open("http://www.lsys.cc:8080", "_blank")
                }}>
                在线示例
            </Button>
        </Paper>

        {(loadData.status || loadData.loading)
            ? <Box sx={{ height: 1, width: '100%' }}>
                <SimpleTablePage
                    rows={loadData.data}
                    columns={columns}
                    count={loadData.total}
                    page={searchParam.get("page") || 0}
                    onPageChange={(e, newPage) => {
                        setSearchParam({
                            page: newPage
                        }, loadAppData)
                    }}
                    rowsPerPage={searchParam.get("page_size") || 25}
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




export function ParentAppListBox(props) {

    const { loadData } = props;
    const { toast } = useContext(ToastContext);

    const [appParam, setAppParam] = useState({
        page: 0,
        page_size: 25
    });
    const [appData, setAppData] = useState({
        loading: false,
        rows: [],
        rows_total: 0,
        error: null,
    })

    const loadAppData = () => {
        setAppData({
            ...appData,
            loading: true
        })
        listParentApp({
            appid: loadData.id,
            page: appParam.page,
            page_size: appParam.page_size
        }).then((data) => {
            if (!data.status) {
                toast(data.message);
                setAppData({
                    ...appData,
                    error: data.message,
                    loading: false
                })
                return;
            }
            setAppData({
                ...appData,
                rows: data.data ?? [],
                rows_total: data.total || 0,
                loading: false,
            })
        })
    }

    useEffect(() => {
        loadAppData();
    }, [appParam])

    const columns = [
        {
            label: "应用id",
            style: { width: 120 },
            align: "right",
            field: "app_id",
        },
        {
            label: "应用名",
            style: { width: 140 },
            align: "left",
            field: "app_name",
        },
        {
            label: "应用用户ID",
            style: { width: 120 },
            align: "center",
            field: "user_id",
        },
        {
            label: "授权时间",
            style: { width: 180 },
            align: "center",
            render: (row) => {
                return showTime(row.change_time, "无")
            }
        },
        {
            label: "授权信息",
            align: "center",
            render: (row) => {
                const [AddBox, setAddBox] = useState({
                    open: false,
                    secret: '',
                    loading: false,
                });

                switch (row.sub_app_status) {
                    case 1: return <Stack direction="row" justifyContent="center"
                        alignItems="center" spacing={1} >
                        <Box>已授权:[{row.sub_app_client_secret}]</Box>
                        <ConfirmButton
                            message={`确定删除此应用访问吗?`}
                            onAction={() => {
                                return delParentApp({
                                    appid: loadData.id,
                                    parent_appid: row.app_id
                                }).then((data) => {
                                    if (!data.status) return;
                                    let rows = appData.rows.map((t) => {
                                        if (t.app_id == row.app_id) {
                                            return {
                                                ...t,
                                                sub_app_status: -1
                                            }
                                        }
                                        return t;
                                    })
                                    setAppData({
                                        ...appData,
                                        rows: rows,
                                    })
                                    return data;
                                })
                            }}
                            renderButton={(props) => {
                                return <IconButton>
                                    <HighlightOffIcon title="删除" fontSize='small' {...props} />
                                </IconButton>
                            }} />
                    </Stack>;
                    case 2: return '已被禁用';
                    default: return <Fragment>
                        <Dialog
                            open={AddBox.open}
                            onClose={() => { setAddBox({ ...AddBox, open: false }) }}
                        >
                            <DialogContent sx={{
                                minWidth: 350
                            }}>
                                <Typography variant="subtitle1" gutterBottom>
                                    设置用于被应用[{row.app_name}]获取的秘钥
                                </Typography>

                                <Form sx={{ mt: 2 }}>
                                    <TextField
                                        sx={{
                                            width: 1,
                                        }}
                                        label="client secret"
                                        variant="outlined"
                                        size="small"
                                        value={AddBox.secret}
                                        onChange={(e) => {
                                            let value = (e.target.value + '').replace(/[^0-9a-z]+/, '');
                                            setAddBox({
                                                ...AddBox,
                                                secret: value
                                            })
                                        }}
                                        disabled={AddBox.loading}
                                        required
                                    />
                                </Form>
                            </DialogContent>
                            <DialogActions>
                                <LoadingButton disabled={AddBox.loading}
                                    loading={AddBox.loading}
                                    onClick={() => {
                                        setAddBox({ ...AddBox, loading: true })
                                        return setParentApp({
                                            appid: loadData.id,
                                            parent_appid: row.app_id,
                                            sub_secret: AddBox.secret
                                        }).then((data) => {
                                            if (!data.status) {
                                                toast(data.message);
                                                setAddBox({ ...AddBox, loading: false })
                                                return;
                                            }
                                            let rows = appData.rows.map((t) => {
                                                if (t.app_id == row.app_id) {
                                                    return {
                                                        ...t,
                                                        sub_app_status: 1,
                                                        sub_app_client_secret: AddBox.secret
                                                    }
                                                }
                                                return t;
                                            })
                                            setAddBox({ ...AddBox, loading: false, open: false, secret: '' })
                                            setAppData({
                                                ...appData,
                                                rows: rows,
                                            })
                                        })
                                    }} >
                                    保存
                                </LoadingButton>
                            </DialogActions>
                        </Dialog>
                        <Stack direction="row" spacing={1} justifyContent="center"
                            alignItems="center"  >
                            <Box>未设置授权</Box>
                            <IconButton onClick={() => {
                                setAddBox({ ...AddBox, open: true })
                            }}>
                                <ControlPointIcon title="设置" fontSize='small' />
                            </IconButton>
                        </Stack>
                    </Fragment>
                        ;
                }
            }
        }
    ];

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
                应用 [{loadData.name}] 可关联以下应用
            </Typography>
            <Divider variant="middle" />
            <Box sx={{
                m: 2
            }}>
                <SimpleTablePage
                    rows={appData.rows ?? []}
                    columns={columns}

                    count={appData.rows_total ?? 0}
                    page={appParam.page}
                    rowsPerPage={appParam.page_size}
                    onPageChange={(e, newPage) => {
                        setAppParam({
                            ...appParam,
                            page: newPage
                        })
                    }}
                    onRowsPerPageChange={(e) => {
                        setAppParam({
                            ...appParam,
                            page_size: e.target.value,
                            page: 0
                        })
                    }}
                    loading={appData.loading}
                />
            </Box>
        </Fragment>)
}



export function SubAppListBox(props) {
    const { loadData } = props;
    const { toast } = useContext(ToastContext);

    const [subAppParam, setSubAppParam] = useState({
        app_id: 0,
        page: 0,
        page_size: 25
    });
    const [subAppData, setSubAppData] = useState({
        loading: false,
        rows: [],
        rows_total: 0,
        error: null,
    })

    const loadSubAppData = () => {
        setSubAppData({
            ...subAppData,
            loading: true
        })
        listSubApp({
            appid: loadData.id,
            page: subAppParam.page,
            page_size: subAppParam.page_size
        }).then((data) => {
            if (!data.status) {
                toast(data.message);
                setSubAppData({
                    ...subAppData,
                    error: data.message,
                    loading: false
                })
                return;
            }
            setSubAppData({
                ...subAppData,
                rows: data.data ?? [],
                rows_total: data.total || 0,
                loading: false,

            })
        })
    }

    useEffect(() => {
        loadSubAppData();
    }, [subAppParam])

    const columns = [
        {
            label: "应用名",
            style: { width: 120 },
            align: "left",
            field: "sub_app_name",
        },
        {
            label: "应用client_id",
            style: { width: 120 },
            align: "left",
            field: "sub_app_client_id",
        },
        {
            label: "应用client_secret",
            style: { width: 150 },
            align: "center",
            field: "sub_app_client_secret",
        },
        {
            label: "应用用户ID",
            style: { width: 120 },
            align: "center",
            field: "user_id"
        },
        {
            label: "当前状态",

            align: "center",
            render: (row) => {
                switch (row.status) {
                    case 1: return '正常';
                    case 2: return '用户被禁用';
                    default: return '未知';
                }
            }
        }
    ];

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
                应用[{loadData.name}]可访问以下应用
            </Typography>
            <Divider variant="middle" />
            <Box sx={{
                m: 2
            }}>
                <SimpleTablePage
                    rows={subAppData.rows ?? []}
                    columns={columns}
                    count={subAppData.rows_total ?? 0}
                    page={subAppParam.page}
                    rowsPerPage={subAppParam.page_size}
                    onPageChange={(e, newPage) => {
                        setSubAppParam({
                            ...subAppParam,
                            page: newPage
                        })
                    }}
                    onRowsPerPageChange={(e) => {
                        setSubAppParam({
                            ...subAppParam,
                            page_size: e.target.value,
                            page: 0
                        })
                    }}
                    loading={subAppData.loading}
                />
            </Box>
        </Fragment>)
}


const AppUserStatus = {
    status: [
        { key: 1, val: '正常' },
        { key: 0, val: '禁用' }
    ],
};



export function SubUserListBox(props) {
    const { loadData } = props;
    const { toast } = useContext(ToastContext);

    const [userDataInput, setUserDataInput] = useState({
        show: false,
        op_user_id: 0,
        input_user_id: 0,
        add_loading: false,
        add_error: '',
    });
    const [userDataParam, setUserDataParam] = useState({
        op_user_id: 0,
        page: 0,
        page_size: 25
    });
    const [userData, setUserData] = useState({
        loading: false,
        rows: [],
        rows_total: 0,
        error: null,
    })


    const loadUserData = () => {
        setUserData({
            ...userData,
            loading: true
        })
        listSubUser({
            appid: loadData.id,
            user_id: userDataParam.op_user_id,
            page: userDataParam.page,
            page_size: userDataParam.page_size
        }).then((data) => {
            if (!data.status) {
                setUserData({
                    ...userData,
                    error: data.message,
                    loading: false
                })
                return;
            }
            setUserData({
                ...userData,
                rows: data.data ?? [],
                rows_total: data.total || 0,
                loading: false,

            })
            if (parseInt(data.total) == 0) {
                setUserDataInput({
                    ...userDataInput,
                    input_user_id: userDataParam.op_user_id,
                    show: true,
                })
            }
        })
    }

    useEffect(() => {
        loadUserData();
    }, [userDataParam])

    const columns = [
        {
            label: "用户ID",
            style: { width: 100 },
            align: "center",
            field: "user_id",
        },
        {
            label: "授权时间",
            style: { width: 170 },
            align: "left",
            render: (row) => {
                return showTime(row.change_time, "未知")
            }
        },
        {
            label: "当前状态",
            style: { width: 120 },
            align: "center",
            render: (row) => {
                let status = AppUserStatus.status.find((e) => {
                    return row.status == e.key;
                });
                if (status) {
                    return status.val
                } else {
                    return "未知"
                }
            }
        },
        {
            label: "操作",
            align: "center",
            render: (row) => {
                const [userStatusData, setUserStatusData] = useState({
                    load: false,
                    set_status: row.status,
                    now_status: row.status,
                });
                return <Fragment>
                    <Switch size="small"

                        checked={userStatusData.set_status == 1}
                        onChange={
                            (e) => {
                                setUserStatusData({
                                    ...userStatusData,
                                    load: true,
                                    set_status: e.target.checked ? 1 : 2
                                })
                                return setSubUser({
                                    appid: loadData.id,
                                    user_id: row.user_id,
                                    used: !!e.target.checked
                                }).then((res) => {
                                    if (!res.status) {
                                        toast(res.message)
                                        setUserStatusData({
                                            ...userStatusData,
                                            load: false,
                                            set_status: userStatusData.now_status
                                        })
                                    } else {
                                        let rows = userData.rows.map((t) => {
                                            if (t.user_id == row.user_id) {
                                                return {
                                                    ...t,
                                                    status: e.target.checked ? 1 : 0
                                                };
                                            } else { return t; }
                                        });
                                        setUserData({
                                            ...userData,
                                            rows: rows
                                        })
                                    }
                                });
                            }
                        } />
                </Fragment >
                    ;
            }
        },
    ];
    let LoadError = function (props) {
        const { error, ...other } = props;
        return <Box {...other}>
            <Alert severity="error">{error}</Alert>
        </Box>
    }



    return (
        <Fragment>
            <Typography
                align="center"
                variant="subtitle1"
                noWrap
                sx={{
                    mt: 4,
                    mb: 2,
                    fontWeight: 100,
                    alignItems: "center",
                    letterSpacing: '.3rem',
                    color: 'inherit',
                    textDecoration: 'none',
                }}
            >
                应用[{loadData.name}]可被以下用户关联应用
            </Typography>
            <Divider variant="middle" />
            <Grid
                sx={{
                    mt: 3,
                }}
                container
                justifyContent="center"
                alignItems="center"
            >
                <Grid item xs={11}>
                    <Form method="post" onSubmit={(e) => {
                        e.preventDefault();
                    }} >
                        <Grid
                            container item
                            justifyContent="center"
                            alignItems="center"
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}

                        >
                            <Grid item xs={3.6}>
                                <UserSearchInput
                                    onSelect={(nval) => {
                                        let value = (nval + '').replace(/[^0-9]+/, '');
                                        setUserDataInput({
                                            ...userDataInput,
                                            op_user_id: value
                                        })
                                    }}
                                    sx={{
                                        width: 1,
                                    }}
                                    variant="outlined"
                                    label={`选择用户`}
                                    value={userDataInput.op_user_id > 0 ? userDataInput.op_user_id : ''}
                                    type="text"
                                    name="code"

                                    size="small"
                                    disabled={userDataInput.add_loading || userData.loading}
                                    enableUser={true}
                                    required
                                />
                            </Grid>
                            <Grid item xs={2.8} sx={{ pl: 1 }}>
                                <LoadingButton
                                    loading={userData.loading}
                                    disabled={userDataInput.add_loading || userData.loading}
                                    variant="outlined"
                                    size="medium"
                                    startIcon={<SearchIcon />}
                                    sx={{ pd: 1, pt: 1, width: 1 }}
                                    onClick={() => {
                                        setUserDataParam({
                                            ...userDataParam,
                                            op_user_id: userDataInput.op_user_id
                                        })
                                    }}
                                >
                                    查找
                                </LoadingButton>
                            </Grid>
                            <Grid item xs={2.8} sx={{ pl: 1 }}>
                                <LoadingButton
                                    loading={userDataInput.add_loading}
                                    disabled={userDataInput.add_loading}
                                    variant="outlined"
                                    size="medium"
                                    startIcon={<AddCircleOutlineIcon />}
                                    sx={{ pd: 1, pt: 1, width: 1 }}
                                    onClick={() => {
                                        setUserData({
                                            ...userData,
                                            rows: [],
                                            rows_total: 0
                                        });
                                        setUserDataInput({
                                            ...userDataInput,
                                            input_user_id: 0,
                                            op_user_id: 0,
                                            show: true
                                        })
                                    }}
                                >
                                    新增
                                </LoadingButton>
                            </Grid>
                            <Grid item xs={2.8} sx={{ pl: 1 }}>
                                <LoadingButton
                                    loading={userData.loading}
                                    disabled={userDataInput.add_loading || userData.loading}
                                    variant="outlined"
                                    size="medium"
                                    startIcon={<AllOutIcon />}
                                    sx={{ pd: 1, pt: 1, width: 1 }}
                                    onClick={() => {
                                        setUserDataInput({
                                            ...userDataInput,
                                            op_user_id: ''
                                        })
                                        setUserDataParam({
                                            ...userDataParam,
                                            op_user_id: 0
                                        })
                                    }}
                                >
                                    全部
                                </LoadingButton>
                            </Grid>

                        </Grid>

                    </Form>
                </Grid>
                <Grid item xs={11} sx={{ mb: 3 }}>

                    {userData.error ?
                        <LoadError error={userData.error} /> :
                        (userDataInput.show && userData.rows.length == 0) ?
                            <Fragment>
                                <Paper sx={{ p: 2 }}>
                                    {userDataParam.op_user_id ? <BaseTableNoRows msg={`当前应用未关联用户ID ${userDataParam.op_user_id} ,可以尝试添加`} /> : null}

                                    {userDataInput.add_error ? <LoadError sx={{ mb: 1 }} error={userDataInput.add_error} /> : null}

                                    <Form>

                                        <TextField
                                            sx={{
                                                width: 1,
                                            }}
                                            label="用户ID"
                                            variant="outlined"
                                            name="name"
                                            size="small"
                                            value={userDataInput.input_user_id > 0 ? userDataInput.input_user_id : ''}
                                            onChange={(e) => {
                                                let value = (e.target.value + '').replace(/[^0-9]+/, '');
                                                setUserDataInput({
                                                    ...userDataInput,
                                                    input_user_id: value
                                                })
                                            }}
                                            disabled={userDataInput.add_loading || userData.loading}
                                            required
                                        />

                                        <LoadingButton
                                            loading={userDataInput.add_loading}
                                            disabled={userDataInput.add_loading}
                                            variant="outlined"
                                            size="medium"
                                            startIcon={<AddCircleOutlineIcon />}
                                            sx={{ width: 1, mt: 2 }}
                                            onClick={() => {
                                                setUserDataInput({
                                                    ...userDataInput,
                                                    add_loading: true
                                                })
                                                setSubUser({
                                                    appid: loadData.id,
                                                    user_id: userDataInput.input_user_id,
                                                    used: true
                                                }).then((data) => {
                                                    if (!data.status) {
                                                        setUserDataInput({
                                                            ...userDataInput,
                                                            add_loading: false,
                                                            add_error: data.message
                                                        })
                                                    } else {
                                                        setUserDataInput({
                                                            ...userDataInput,
                                                            add_loading: false,
                                                            op_user_id: userDataInput.input_user_id
                                                        })
                                                        setUserDataParam({
                                                            ...userDataParam,
                                                            op_user_id: userDataInput.input_user_id
                                                        });
                                                    }
                                                })
                                            }}
                                        >
                                            授权用户ID添加子应用
                                        </LoadingButton>
                                    </Form>

                                </Paper>
                            </Fragment> :
                            <SimpleTablePage
                                rows={userData.rows ?? []}
                                columns={columns}
                                count={userData.rows_total ?? 0}
                                page={userDataParam.page}
                                rowsPerPage={userDataParam.page_size}
                                onPageChange={(e, newPage) => {
                                    setUserDataParam({
                                        ...userDataParam,
                                        page: newPage
                                    })
                                }}
                                onRowsPerPageChange={(e) => {
                                    setUserDataParam({
                                        ...userDataParam,
                                        page_size: e.target.value,
                                        page: 0
                                    })
                                }}
                                loading={userData.loading}
                            />
                    }
                </Grid>
            </Grid >
        </Fragment >)
}



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
    const { onFinish, loadData } = props;
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
                onFinish(appData.client_id)
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
                            disabled={true}
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

