

import { Alert, Box, Button, Dialog, DialogActions, DialogContent, DialogContentText, Divider, Grid, IconButton, Typography } from '@mui/material';
import React, { Fragment, useEffect, useState } from 'react';
import { SimpleTablePage } from '../../../../library/table_page';
import { MessageLogStatus, MessageLogType, senderCancelAppMessage, senderListAppMessageLog, senderSeeAppMessage } from '../../../../common/rest/sender_setting';
import { showTime } from '../../../../common/utils/utils';
import { ConfirmButton } from '../../../../common/ui/dialog';
import CancelIcon from '@mui/icons-material/Cancel';
import ArticleIcon from '@mui/icons-material/Article';
import { Progress } from '../../../../library/loading';
export function MessageSeeBody(props) {
    const {
        row,
        msgType
    } = props;
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
                    <Box>消息ID:{row.snid.toString()}</Box>
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
        <IconButton title="查看内容" onClick={() => {
            setKeyBox({
                ...keyBox,
                open: true,
                loading: true,
            })
            return senderSeeAppMessage(msgType, { message_id: row.id }).then((data) => {
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


export function MessageCancelButton(props) {
    const {
        row,
        message,
        msgType,
        onDelete
    } = props;
    let delAction = () => {
        return senderCancelAppMessage(msgType, { message_id: row.id }).then((data) => {
            if (!data.status) return data;
            onDelete(row)
            return data;
        })
    };
    return row.status == 1 ? <ConfirmButton
        message={message}
        onAction={() => { return delAction(row) }}
        renderButton={(props) => {
            return <IconButton title="取消发送"  {...props} size='small' ><CancelIcon fontSize="small" /></IconButton>
        }} /> : null
}


export function MessageLogBox(props) {
    const { msgType, msgData } = props;
    const [historyData, setHistoryData] = useState({
        loading: false,
        rows: [],
        rows_total: 0,
        error: null,
    })
    const [historyDataParam, setHistoryDataParam] = useState({
        page: 0,
        page_size: 25
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
        senderListAppMessageLog(msgType, param).then((data) => {
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
            style: { width: 100 },
            render: (row) => {
                let val = MessageLogType.find((e) => {
                    return row.log_type == e.key
                });
                if (!val) return "未知";
                return val.val;
            }
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
            style: { width: 160 },
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
                查看 {msgData.snid.toString()} 日志
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
                        <SimpleTablePage
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

