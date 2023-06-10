import ManageSearchIcon from '@mui/icons-material/ManageSearch';
import SearchIcon from '@mui/icons-material/Search';
import { Alert, Drawer, FormControl, IconButton, InputLabel, MenuItem, Paper, Select } from '@mui/material';
import Box from '@mui/material/Box';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { ToastContext } from '../../../context/toast';
import { ClearTextField } from '../../../library/input';
import { LoadingButton } from '../../../library/loading';
import { SimplePaginationTablePage } from '../../../library/table_page';
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
        startPos,
        endPos,
        pageSize,
        onSearchChange,
    } = props;
    let [loadData, setLoadData] = useState({
        status: false,
        message: null,
        loading: true,
        data: [],
        startPos: '',
        nextPos: '',
        isFirst: (!startPos || startPos == '') ? true : false,
        isEnd: true,
    });

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
            style: { width: 120 },
            label: '发送邮箱',
            render: (row) => {
                if (row.reply_to && row.reply_to.length > 0) {
                    return row.to_mail + `<br>回复:${row.reply_to}`
                }
                return row.to_mail
            }
        },
        {
            field: "tpl_id",
            style: { width: 100 },
            label: '模板'
        },
        {
            style: { width: 60 },
            label: '数据',
            render: (row) => {
                return <MessageSeeBody row={row} msgType="mailer" />
            }
        },
        {

            style: { width: 100 },
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
            style: { width: 280 },
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
            style: { width: 130 },
            label: '添加时间',
            render: (row) => {
                return showTime(row.add_time, "未知")
            }
        },
        {

            style: { width: 160 },
            label: '发送日志',
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
        window.scrollTo({ top: 0, left: 0, behavior: 'smooth' });
        return senderListAppMessage("mailer", {
            user_id: parseInt(userId),
            app_id: (props.children && !appId) ? -1 : appId,
            tpl_id: tplId,
            to_mail: to_mail,
            status: status,
            start_pos: startPos || '',
            end_pos: endPos || '',
            page_size: pageSize || 25
        }).then((data) => {
            let setData = data.status && data.data && data.data.length > 0 ? data.data : [];
            if (endPos && endPos != '') {
                setLoadData({
                    ...loadData,
                    status: data.status ?? false,
                    message: data.message ?? '',
                    data: setData,
                    loading: false,
                    startPos: setData.length > 0 ? setData[0].id : '',
                    nextPos: endPos,
                    isFirst: !data.status || !data.next || data.next == '',
                    isEnd: false,
                })
            } else {
                setLoadData({
                    ...loadData,
                    status: data.status ?? false,
                    message: data.message ?? '',
                    data: setData,
                    loading: false,
                    startPos: setData.length > 0 ? setData[0].id : '',
                    nextPos: data.status && data.next ? data.next : '',
                    isFirst: !startPos || startPos == '',
                    isEnd: !data.status || !data.next || data.next == '',
                })
            }

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
                    label={`邮箱`}
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
                        start_pos: '',
                        end_pos: '',
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
                <SimplePaginationTablePage
                    rows={loadData.data ?? []}
                    columns={columns}
                    isFirst={loadData.isFirst}
                    isEnd={loadData.isEnd}
                    onPageChange={(e, next) => {
                        if (next) {
                            onSearchChange({
                                start_pos: loadData.nextPos,
                                end_pos: '',
                            }, loadMsgData)
                        } else {
                            onSearchChange({
                                start_pos: '',
                                end_pos: loadData.startPos,
                            }, loadMsgData)
                        }
                    }}
                    rowsPerPage={pageSize || 25}
                    onRowsPerPageChange={(e) => {
                        onSearchChange({
                            page_size: e.target.value,
                        }, loadMsgData)
                    }}
                    loading={loadData.loading}
                />
            </Box> : <Alert severity="error">{loadData.message}</Alert>}
    </Fragment>
}


