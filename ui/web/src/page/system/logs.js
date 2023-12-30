
import SearchIcon from '@mui/icons-material/Search';
import { Alert, Button, Dialog, DialogActions, DialogContent, DialogContentText, DialogTitle, FormControl, IconButton, InputLabel, MenuItem, Paper, Select, Stack } from '@mui/material';
import Box from '@mui/material/Box';
import React, { Fragment, useEffect, useState } from 'react';
import { LoadingButton } from '../../library/loading';
import { SimplePaginationTablePage } from '../../library/table_page';
import { logsMap, userLogs } from '../../rest/user';
import { useSearchChange } from '../../utils/hook';
import { showTime } from '../../utils/utils';
import { UserSearchInput } from '../library/user';
import CodeEditor from '@uiw/react-textarea-code-editor';
import MoreHorizIcon from '@mui/icons-material/MoreHoriz';
import { ShowCode } from '../library/show_code';
export default function SystemLogsPage(props) {
    const [searchParam, setSearchParam] = useSearchChange({
        user_id: "",
        add_user_id: "",
        log_type: "",
        start_pos: '',
        end_pos: '',
        page_size: 25,
    });
    let [loadData, setLoadData] = useState({
        status: false,
        message: null,
        loading: true,
        data: [],
        startPos: '',
        nextPos: '',
        isFirst: false,
        isEnd: true,
    });
    const columns = [
        {
            field: 'id',
            label: 'ID',
            align: "right",
            style: { width: 90 }
        },
        {
            style: { width: 160 },
            label: '操作类型',
            render: (row) => {
                let val = logsMap.logType.find((e) => {
                    return row.log_type == e.key
                });
                if (!val) return row.log_type;
                return val.val;
            }
        },
        {
            style: { width: 110 },
            label: '资源用户ID',
            align: "center",
            render: (row) => {
                return row.user_id ? row.user_id : "系统";
            }

        },
        {
            style: { width: 110 },
            label: '操作用户ID',
            align: "center",
            render: (row) => {
                return row.add_user_id ? row.add_user_id : "系统";

            }

        },
        {
            label: '操作信息',
            render: (row) => {
                return <Stack direction={"row"}><Box sx={{ lineHeight: "32px" }}>{row.message}</Box> <ShowCode
                    language="json"
                    title={`操作相关数据${row.id}`}
                    dataCallback={() => {
                        let data = JSON.parse(row.log_data);
                        if (data) {
                            data = JSON.stringify(data, null, 2);
                        }
                        if (!data) {
                            data = row.log_data
                        }
                        return data
                    }}
                    sx={{
                        minWidth: 350
                    }} > <IconButton ><MoreHorizIcon fontSize='small' /></IconButton>
                </ShowCode>
                </Stack>

            }
        },
        {
            field: "user_ip",
            style: { width: 120 },
            label: '操作IP'
        },
        {
            style: { width: 180 },
            label: '操作时间',
            render: (row) => {
                return showTime(row.add_time, "未知")
            }
        }
    ];

    useEffect(() => {
        let startPos = searchParam.get("start_pos") ?? '';
        setLoadData({
            ...loadData,
            isFirst: (!startPos || startPos == '') ? true : false,
        })
    }, [])
    const [filterData, setfilterData] = useState({
        user_id: searchParam.get("user_id"),
        add_user_id: searchParam.get("add_user_id"),
        log_type: searchParam.get("log_type"),
    })
    const loadLogsData = () => {
        setLoadData({
            ...loadData,
            loading: true
        })
        window.scrollTo({ top: 0, left: 0, behavior: 'smooth' });
        let param = {
            more: true,
            user_id: searchParam.get("user_id"),
            add_user_id: searchParam.get("add_user_id"),
            log_type: searchParam.get("log_type"),
            start_pos: searchParam.get("start_pos") ?? '',
            end_pos: searchParam.get("end_pos") ?? '',
            page_size: searchParam.get("page_size") || 25,
        }
        return userLogs(param).then((data) => {
            let setData = data.status && data.data && data.data.length > 0 ? data.data : [];
            if (param.end_pos && param.end_pos != '') {
                setLoadData({
                    ...loadData,
                    status: data.status ?? false,
                    message: data.message ?? '',
                    data: setData,
                    loading: false,
                    startPos: setData.length > 0 ? setData[0].id : '',
                    nextPos: param.end_pos,
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
                    isFirst: !param.start_pos || param.start_pos == '',
                    isEnd: !data.status || !data.next || data.next == '',
                })
            }
        })
    }
    useEffect(() => {
        setfilterData({
            ...filterData,
            user_id: searchParam.get("user_id"),
            add_user_id: searchParam.get("add_user_id"),
            log_type: searchParam.get("log_type"),
        })
        loadLogsData()
    }, [searchParam])

    return <Fragment>
        <Paper
            sx={{ p: 2, display: 'flex', alignItems: 'center', marginBottom: 1, marginTop: 1 }}
        >
            <FormControl sx={{ minWidth: 110, mr: 1 }} size="small"  >
                <InputLabel id="select-type">操作类型</InputLabel>
                <Select
                    labelId="select-type"
                    id="select-type"
                    label="操作类型"
                    disabled={loadData.loading}
                    onChange={(event) => {
                        setfilterData({
                            ...filterData,
                            log_type: event.target.value
                        })
                    }}
                    value={filterData.log_type ?? ''}
                >
                    <MenuItem value="">全部</MenuItem>
                    {
                        logsMap.logType.map((status) => {
                            return <MenuItem key={`status_${status.key}`} value={status.key}>{status.val}</MenuItem>
                        })
                    }
                </Select>
            </FormControl>
            <FormControl sx={{ minWidth: 170, mr: 1 }} size="small"  >
                <UserSearchInput
                    onSelect={(nval) => {
                        let value = (nval + '').replace(/[^0-9]+/, '');
                        setfilterData({
                            ...filterData,
                            user_id: value
                        })
                    }}
                    sx={{ width: 1 }}
                    variant="outlined"
                    label={`选择资源用户`}
                    value={filterData.user_id > 0 ? filterData.user_id : ''}
                    type="text"
                    size="small"
                    disabled={loadData.loading}
                    enableUser={true}
                />
            </FormControl>
            <FormControl sx={{ minWidth: 170, mr: 1 }} size="small"  >
                <UserSearchInput
                    onSelect={(nval) => {
                        let value = (nval + '').replace(/[^0-9]+/, '');
                        setfilterData({
                            ...filterData,
                            add_user_id: value
                        })
                    }}
                    sx={{ width: 1 }}
                    variant="outlined"
                    label={`选择操作用户`}
                    value={filterData.add_user_id > 0 ? filterData.add_user_id : ''}
                    type="text"
                    size="small"
                    disabled={loadData.loading}
                    enableUser={true}
                />
            </FormControl>
            <LoadingButton
                onClick={() => {
                    setSearchParam({
                        ...filterData,
                        start_pos: '',
                        end_pos: ''
                    }, loadLogsData)
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
                            setSearchParam({
                                start_pos: loadData.nextPos,
                                end_pos: '',
                            }, loadLogsData)
                        } else {
                            setSearchParam({
                                start_pos: '',
                                end_pos: loadData.startPos,
                            }, loadLogsData)
                        }
                    }}
                    rowsPerPage={searchParam.get("page_size") || 25}
                    onRowsPerPageChange={(e) => {
                        setSearchParam({
                            page_size: e.target.value,
                        }, loadLogsData)
                    }}
                    loading={loadData.loading}
                />

            </Box> : <Alert severity="error">{loadData.message}</Alert>}
    </Fragment>
}


