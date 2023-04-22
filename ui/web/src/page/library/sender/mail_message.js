import ManageSearchIcon from '@mui/icons-material/ManageSearch';
import SearchIcon from '@mui/icons-material/Search';
import { Alert, Drawer, FormControl, IconButton, InputLabel, MenuItem, Paper, Select } from '@mui/material';
import Box from '@mui/material/Box';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { ToastContext } from '../../../context/toast';
import { ClearTextField } from '../../../library/input';
import { LoadingButton } from '../../../library/loading';
import { BaseTablePage } from '../../../library/table_page';
import { MessageStatus, senderListAppMessage } from '../../../rest/sender_setting';
import { showTime } from '../../../utils/utils';
import { MessageDeleteButton, MessageLogBox, MessageSeeBody } from './lib_message';
import { ItemTooltip } from '../../../library/tips';

export function AppMailMessage(props) {
    const {
        userId,
        appId,
        tplId,
        to_mail,
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
            field: "to_mail",
            style: { width: 120 },
            label: '发送邮箱'
        },
        {
            field: "reply_to",
            style: { width: 120 },
            label: '回复邮箱'
        },
        {
            field: "tpl_id",
            style: { width: 100 },
            label: '模板'
        },
        {
            style: { width: 100 },
            label: '数据',
            render: (row) => {
                return <MessageSeeBody row={row} msgType="mailer" />
            }
        },
        {

            style: { width: 120 },
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
            style: { width: 300 },
            label: '发送详细',
            render: (row) => {
                let num_txt = "还未开始";
                if (row.try_num > 0) {
                    if (row.status == 1) {
                        num_txt = "已失败" + row.try_num + "次";
                    } else {
                        num_txt = "发送" + row.try_num + "次";
                    }
                }
                let stime = "发送于:" + showTime(row.send_time, "未知");
                if (row.status == 3) {
                    stime = "失败于:" + showTime(row.send_time, "未知")
                } else if (row.status == 1) {
                    stime = "预计于:" + showTime(row.send_time, "未知")
                }
                return <ItemTooltip title={num_txt} placement="top">
                    <span> {stime}</span>
                </ItemTooltip>
            }
        },
        {
            style: { width: 120 },
            label: '添加时间',
            render: (row) => {
                return showTime(row.add_time, "未知")
            }
        },
        {

            style: { width: 160 },
            label: '操作',
            align: "center",
            render: (row) => {
                return <Fragment>
                    <IconButton
                        title="发送历史"
                        size='small'
                        onClick={() => {
                            setChangeBox({ show: 1, data: row })
                        }}
                    ><ManageSearchIcon fontSize="small" /></IconButton>
                    <MessageDeleteButton
                        row={row}
                        message={`确定取消发送邮件 [${row.id}] 吗?`}
                        msgType="mailer"
                        onDelete={(delRow) => {
                            let rows = loadData.data.map((item) => {
                                if (item.id == delRow.id) {
                                    item.status = 4;
                                }
                                return item;
                            })
                            setLoadData({
                                ...loadData,
                                data: rows
                            })
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
            status: status,
            tpl_id: tplId,
            to_mail: to_mail,
        }, ...props.children ? { app_id: appId } : {}
    })
    const loadMsgData = () => {
        setLoadData({
            ...loadData,
            loading: true
        })
        return senderListAppMessage("mailer", {
            user_id: parseInt(userId),
            app_id: (props.children && !appId) ? -1 : appId,
            tpl_id: tplId,
            to_mail: to_mail,
            status: status,
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
            }, ...props.children ? { app_id: appId } : {}
        })
        loadMsgData()
    }, [props])
    const [changeBoxState, setChangeBox] = useState({ show: 0, data: null });

    let showBox
    switch (changeBoxState.show) {
        case 1:
            showBox = <MessageLogBox
                msgData={changeBoxState.data}
                msgType="mailer"
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
                    value={filterData.to_mail}
                    size="small"
                    disabled={loadData.loading}
                    onChange={(event, nval) => {
                        setfilterData({
                            ...filterData,
                            to_mail: nval
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
                sx={{ mr: 1, p: "7px 15px", minWidth: 85 }}
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


