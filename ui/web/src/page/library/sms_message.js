

import CancelIcon from '@mui/icons-material/Cancel';
import ManageSearchIcon from '@mui/icons-material/ManageSearch';
import SearchIcon from '@mui/icons-material/Search';
import { Alert, Button, Dialog, DialogActions, DialogContent, DialogContentText, DialogTitle, Divider, Drawer, FormControl, Grid, IconButton, InputLabel, MenuItem, Paper, Select, Typography } from '@mui/material';
import Box from '@mui/material/Box';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { ToastContext } from '../../context/toast';
import { ConfirmButton } from '../../library/dialog';
import { ClearTextField } from '../../library/input';
import { LoadingButton, Progress } from '../../library/loading';
import { BaseTablePage } from '../../library/table_page';
import { cancelAppMessage, listAppMessage, listAppMessageHistory, MessageLogStatus, MessageLogType, MessageStatus, viewAppMessage } from '../../rest/sms_setting';
import { showTime } from '../../utils/utils';
import ArticleIcon from '@mui/icons-material/Article';

function HistoryBox(props) {
    const { msgData } = props;
    const [historyData, setHistoryData] = useState({
        loading: false,
        rows: [],
        rows_total: 0,
        error: null,
    })
    const [historyDataParam, setHistoryDataParam] = useState({
        page: 0,
        page_size: 10
    });
    const loadhistoryData = () => {
        setHistoryData({
            ...historyData,
            loading: true
        })
        let param = {
            message_id: msgData.id,
            page: historyDataParam.page,
            page_size: historyDataParam.page_size
        };
        listAppMessageHistory(param).then((data) => {
            if (!data.status) {
                setHistoryData({
                    ...historyData,
                    error: data.message,
                    loading: false
                })
                return;
            }
            setHistoryData({
                ...historyData,
                rows: data.data ?? [],
                rows_total: data.total || 0,
                loading: false,

            })
        });
    }

    useEffect(() => {
        loadhistoryData();
    }, [historyDataParam])


    const columns = [
        {
            field: 'id',
            label: 'ID',
            align: "right",
            style: { width: 80 }
        },
        {
            label: "类型",
            style: { width: 70 },
            render: (row) => {
                let val = MessageLogType.find((e) => {
                    return row.send_type == e.key
                });
                if (!val) return "未知";
                return val.val;
            }
        },
        {
            field: "send_type",
            label: "发送方",
            style: { width: 80 }
        },
        {
            label: "状态",
            align: "center",
            style: { width: 70 },
            render: (row) => {
                let val = MessageLogStatus.find((e) => {
                    return row.status == e.key
                });
                if (!val) return "未知";
                return val.val;
            }
        },
        {
            label: "时间",
            style: { width: 180 },
            render: (row) => {
                return showTime(row.create_time, "未知")
            }
        },
        {
            field: "message",
            label: "消息"
        }
    ];
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
                消息 {msgData.id.toString()} 日志
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
                <Grid item xs={11} sx={{ mb: 3 }}>
                    {historyData.error ?
                        <Alert severity="error">{historyData.error}</Alert>
                        :
                        <BaseTablePage
                            rows={historyData.rows ?? []}
                            columns={columns}
                            count={historyData.rows_total ?? 0}
                            page={historyDataParam.page}
                            rowsPerPage={historyDataParam.page_size}
                            onPageChange={(e, newPage) => {
                                setHistoryDataParam({
                                    ...historyDataParam,
                                    page: newPage
                                })
                            }}
                            onRowsPerPageChange={(e) => {
                                setHistoryDataParam({
                                    ...historyDataParam,
                                    page_size: e.target.value,
                                    page: 0
                                })
                            }}
                            loading={historyData.loading}
                        />
                    }
                </Grid>
            </Grid >
        </Fragment >)
}



export default function AppSmsMessage(props) {
    const {
        userId,
        appId,
        tplId,
        mobile,
        status,
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
    const { toast } = useContext(ToastContext);
    let columns = [
        {
            label: 'ID',
            align: "right",
            style: { width: 80 },
            render: (row) => {
                return row.id.toString()
            }
        },
        {
            field: "app_id",
            style: { width: 80 },
            label: 'AppID'
        },
        {
            field: "mobile",
            style: { width: 160 },
            label: '接收号码'
        },
        {
            field: "tpl_id",
            style: { width: 120 },
            label: '模板'
        },
        {
            style: { width: 120 },
            label: '数据',
            render: (row) => {
                const [keyBox, setKeyBox] = useState({
                    open: false,
                    loading: false,
                    data: ''
                });
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
                                <Box>消息ID:{row.id.toString()}</Box>
                                <Box sx={{
                                    mt: 1,
                                    fontSize: "0.8rem"
                                }}>消息内容:{keyBox.body}</Box>
                            </DialogContentText>
                        </DialogContent>}
                        <DialogActions>
                            <Button onClick={() => { setKeyBox({ ...keyBox, open: false, loading: false, data: '' }) }} >
                                关闭
                            </Button>
                        </DialogActions>
                    </Dialog>
                    <IconButton title="查看短信内容" onClick={() => {
                        setKeyBox({
                            ...keyBox,
                            open: true,
                            loading: true,
                        })
                        return viewAppMessage({ message_id: row.id }).then((data) => {
                            if (!data.status) {
                                toast(data.message)
                                setKeyBox({
                                    ...keyBox,
                                    loading: false,
                                    open: false,
                                    body: ''
                                })
                                return;
                            }
                            setKeyBox({
                                open: true,
                                loading: false,
                                body: data.body
                            })
                        })
                    }} size='small'>
                        <ArticleIcon fontSize='small' />
                    </IconButton>
                </Fragment>
            }
        },
        {

            style: { width: 150 },
            label: '状态',
            render: (row) => {
                let f = MessageStatus.find((e) => { return e.key == row.status });
                if (!f) {
                    return "未知类型";
                } else {
                    return f.val;
                }
            }
        },
        {
            align: "center",
            style: { width: 200 },
            label: '已发送次数',
            render: (row) => {
                if (row.try_num == 0) {
                    return "还未开始";
                } else {
                    if (row.status == 1) {
                        return "已失败" + row.try_num + "次";
                    } else {

                        return "总发送" + row.try_num + "次";
                    }
                }
            }
        },
        {
            style: { width: 160 },
            label: '预计发送时间',
            render: (row) => {
                return showTime(row.expected_time, "未知")
            }
        },
        {
            style: { width: 160 },
            label: '添加时间',
            render: (row) => {
                return showTime(row.add_time, "未知")
            }
        },
        {
            style: { width: 160 },
            label: '实际发送时间',
            render: (row) => {
                return showTime(row.send_time, "未发送")
            }
        },
        {

            style: { width: 160 },
            label: '操作',
            align: "center",
            render: (row) => {
                let delAction = () => {
                    return cancelAppMessage({ message_id: row.id }).then((data) => {
                        if (!data.status) return data;
                        let rows = loadData.data.map((item) => {
                            if (item.id == row.id) {
                                item.status = 4;
                            }
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
                    <IconButton
                        title="发送历史"
                        size='small'
                        onClick={() => {
                            setChangeBox({ show: 1, data: row })
                        }}
                    ><ManageSearchIcon fontSize="small" /></IconButton>
                    {row.status == 1 ? <ConfirmButton
                        message={`确定取消发送短信 [${row.id}] 吗?`}
                        onAction={delAction}
                        renderButton={(props) => {
                            return <IconButton title="取消发送"  {...props} size='small' ><CancelIcon fontSize="small" /></IconButton>
                        }} /> : null}
                </Fragment>
            }
        }
    ];
    if (!props.children) {
        columns = columns.filter((e) => { return e.field != 'app_id' })
    }

    const [filterData, setfilterData] = useState({
        ...{
            status: status,
            tpl_id: tplId,
            mobile: mobile,
        }, ...props.children ? { app_id: appId } : {}
    })
    const loadMsgData = () => {
        setLoadData({
            ...loadData,
            loading: true
        })
        return listAppMessage({
            user_id: parseInt(userId),
            app_id: (props.children && !appId) ? -1 : appId,
            tpl_id: tplId,
            mobile: mobile,
            status: status,
            page: page || 0,
            page_size: pageSize || 10
        }).then((data) => {
            setLoadData({
                ...loadData,
                ...data,
                loading: false
            })
        })
    }
    useEffect(() => {
        setfilterData({
            ...{
                ...filterData,
            }, ...props.children ? { app_id: appId } : {}
        })
        loadMsgData()
    }, [props])
    const [changeBoxState, setChangeBox] = useState({ show: 0, data: null });

    let showBox
    switch (changeBoxState.show) {
        case 1:
            showBox = <HistoryBox
                msgData={changeBoxState.data}
            />;
            break;
    };

    return <Fragment>
        <Drawer
            sx={{ zIndex: (theme) => theme.zIndex.drawer + 3 }}
            anchor={"right"}
            open={changeBoxState.show != 0}
            onClose={() => {
                setChangeBox({ show: 0 })
            }}
        >
            <Box
                sx={{ width: 680 }}
                role="presentation"
            >
                {showBox}
            </Box>
        </Drawer>

        <Paper
            sx={{ p: 2, display: 'flex', alignItems: 'center', marginBottom: 1, marginTop: 1 }}
        >
            {props.children}

            <FormControl sx={{
                minWidth: 130,
                mr: 1
            }}>
                <InputLabel size="small" id="res-select-label">发送状态</InputLabel>
                <Select
                    fullWidth
                    size='small'
                    value={filterData.status}
                    onChange={
                        (e) => {
                            setfilterData({
                                ...filterData,
                                status: e.target.value
                            })
                        }
                    }
                    labelId="config-select-small"
                    id="config-select-small"
                    label="发送状态"><MenuItem value=''>全部</MenuItem>
                    {MessageStatus.map((item) => {
                        return <MenuItem key={`status-${item.key}`} value={item.key}>{item.val}</MenuItem>
                    })}
                </Select>
            </FormControl>
            <FormControl sx={{ minWidth: 80 }} size="small"  >
                <ClearTextField
                    sx={{ mr: 1 }}
                    variant="outlined"
                    label={`模板`}
                    type="text"
                    value={filterData.tpl_id}
                    size="small"
                    disabled={loadData.loading}
                    onChange={(event, nval) => {
                        setfilterData({
                            ...filterData,
                            tpl_id: nval
                        })
                    }}
                />
            </FormControl>
            <FormControl sx={{ minWidth: 80 }} size="small"  >
                <ClearTextField
                    sx={{ mr: 1 }}
                    variant="outlined"
                    label={`手机号`}
                    type="text"
                    value={filterData.mobile}
                    size="small"
                    disabled={loadData.loading}
                    onChange={(event, nval) => {
                        setfilterData({
                            ...filterData,
                            mobile: nval
                        })
                    }}
                />
            </FormControl>

            <LoadingButton
                onClick={() => {
                    onSearchChange({
                        ...filterData,
                        page: 0
                    }, loadMsgData)
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
                        }, loadMsgData)
                    }}
                    rowsPerPage={pageSize || 10}
                    onRowsPerPageChange={(e) => {
                        onSearchChange({
                            page_size: e.target.value,
                            page: 0
                        }, loadMsgData)
                    }}
                    loading={loadData.loading}
                />
            </Box> : <Alert severity="error">{loadData.message}</Alert>}
    </Fragment>
}


