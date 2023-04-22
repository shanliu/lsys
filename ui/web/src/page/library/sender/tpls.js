
import AddCircleOutlineIcon from '@mui/icons-material/AddCircleOutline';
import SearchIcon from '@mui/icons-material/Search';
import DeleteIcon from '@mui/icons-material/Delete';
import EditIcon from '@mui/icons-material/Edit';
import { Alert, Button, Dialog, DialogActions, DialogContent, DialogContentText, Divider, Drawer, FormControl, Grid, IconButton, InputLabel, MenuItem, Paper, Select, Table, TableContainer, TextField, Typography } from '@mui/material';
import Box from '@mui/material/Box';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { Form } from 'react-router-dom';
import { UserSessionContext } from '../../../context/session';
import { ToastContext } from '../../../context/toast';
import { ConfirmButton } from '../../../library/dialog';
import { ClearTextField } from '../../../library/input';
import { LoadingButton } from '../../../library/loading';
import { BaseTableBody, BaseTableHead, BaseTablePage } from '../../../library/table_page';
import { SenderType, smsAddAliConfig, smsDelAliConfig, smsEditAliConfig, smsListAliConfig, tplsAddConfig, tplsDelConfig, tplsEditConfig, tplsListConfig } from '../../../rest/sender_setting';
import { useSearchChange } from '../../../utils/hook';
import { showTime } from '../../../utils/utils';


function AddBox(props) {
    const {
        tplType,
        userId,
        rowData,
        onFinish
    } = props;

    const { toast } = useContext(ToastContext);

    let [addData, setAddData] = useState({

        tpl_data: rowData ? rowData.tpl_data : '',
        tpl_id: rowData ? rowData.tpl_id : '',
        loading: false,
    });
    const [addError, setAddError] = useState({

        tpl_data: '',
        tpl_id: '',
    });

    let onSubmit = () => {
        setAddData({
            ...addData,
            loading: true
        })
        if (rowData && rowData.id) {
            tplsEditConfig({
                id: rowData.id,

                tpl_data: addData.tpl_data,
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
                        tpl_data: '',
                        tpl_id: '',
                    })
                    setAddData({
                        ...addData,
                        loading: false
                    })
                    onFinish(rowData.id);
                }
            })
        } else {
            tplsAddConfig({
                user_id: userId,
                sender_type: tplType,

                tpl_data: addData.tpl_data,
                tpl_id: addData.tpl_id
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
                        tpl_data: '',
                        tpl_id: '',
                    })
                    setAddData({
                        name: '',
                        tpl_data: '',
                        tpl_id: '',
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
                模板配置
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
                            label="模板ID"
                            type="text"
                            name="name"
                            size="small"
                            onChange={(e) => {
                                setAddData({
                                    ...addData,
                                    tpl_id: e.target.value
                                })
                            }}
                            value={addData.tpl_id}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            disabled={addData.loading}
                            error={!!addError.tpl_id}
                            helperText={addError.tpl_id}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <TextField
                            multiline
                            rows={8}
                            variant="outlined"
                            label="模板内容"
                            type="text"
                            name="name"
                            size="small"
                            onChange={(e) => {
                                setAddData({
                                    ...addData,
                                    tpl_data: e.target.value
                                })
                            }}
                            value={addData.tpl_data}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            disabled={addData.loading}
                            error={!!addError.tpl_data}
                            helperText={addError.tpl_data}
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


export default function SenderTplsPage(props) {
    const {
        userId, tplType
    } = props;
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
            field: 'tpl_id',
            style: { width: 220 },
            label: '模板ID',
        },
        {
            style: { width: 100 },
            label: '模板内容',
            render: (row) => {
                const [showBox, setShowBox] = useState({
                    open: false
                });
                return <Fragment>
                    <Dialog
                        open={showBox.open}
                        onClose={() => { setShowBox({ ...showBox, open: false }) }}
                    >
                        <DialogContent sx={{
                            minWidth: 350
                        }}>
                            <DialogContentText>
                                {row.tpl_data}
                            </DialogContentText>
                        </DialogContent>
                        <DialogActions>

                            <Button onClick={() => { setShowBox({ ...showBox, open: false }) }} >
                                关闭
                            </Button>
                        </DialogActions>
                    </Dialog>
                    <Button onClick={() => {
                        setShowBox({ ...showBox, open: true })
                    }
                    }>查看</Button>
                </Fragment>
            }
        },
        {

            style: { width: 180 },
            label: '更新用户ID',
            render: (row) => {
                return showTime(row.last_user_id, "未知")
            }
        },
        {
            style: { width: 180 },
            label: '更新时间',
            render: (row) => {
                return showTime(row.last_change_time, "未知")
            }
        },
        {
            label: '操作',
            align: "center",
            render: (row) => {
                let delAction = () => {
                    return tplsDelConfig({ id: row.id }).then((data) => {
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
                        message={`确定删除模板 [${row.tpl_id}] 吗?`}
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
        page: "",
        page_size: "",
        tpl_id: "",
    });
    const [filterData, setfilterData] = useState({
        id: searchParam.get("id"),
        page: searchParam.get("page"),
        page_size: searchParam.get("page_size"),
        tpl_id: searchParam.get("tpl_id"),
    })
    const loadConfigData = () => {
        setLoadData({
            ...loadData,
            loading: true
        })
        return tplsListConfig({
            user_id: userId,
            id: searchParam.get("id"),
            sender_type: tplType,
            page: searchParam.get("page") || 0,
            page_size: searchParam.get("page_size") || 10,
            tpl_id: searchParam.get("tpl_id")
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
            showBox = <AddBox
                tplType={tplType}
                userId={userId}
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
            showBox = <AddBox
                tplType={tplType}
                userId={userId}
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
        <Paper
            sx={{ p: 2, display: 'flex', alignItems: 'center', marginBottom: 1, marginTop: 1 }}
        >
            <FormControl sx={{ minWidth: 80, mr: 1 }} size="small"  >
                <ClearTextField
                    sx={{ mr: 1 }}
                    variant="outlined"
                    label={`模板ID`}
                    type="munber"
                    name="code"
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
            <FormControl sx={{ minWidth: 80, mr: 1 }} size="small"  >
                <ClearTextField
                    sx={{ mr: 1 }}
                    variant="outlined"
                    label={`ID`}
                    type="munber"
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
                sx={{ mr: 1, p: "7px 15px", minWidth: 85 }}
                disabled={loadData.loading}
            >
                过滤
            </LoadingButton>
            <Button
                variant="outlined"
                size="medium"
                startIcon={<AddCircleOutlineIcon />}
                sx={{ mr: 1, p: "7px 15px" }}
                onClick={() => {
                    setChangeBox({ data: {}, show: 1 })
                }}>
                新增配置
            </Button>
        </Paper>

        {(loadData.status || loadData.loading)
            ? <Box sx={{ height: 1, width: '100%' }}>
                <BaseTablePage
                    rows={loadData.data ?? []}
                    columns={columns}
                    count={loadData.total}
                    page={searchParam.get("page") || 0}
                    onPageChange={(e, newPage) => {
                        setSearchParam({
                            page: newPage
                        }, loadConfigData)
                    }}
                    rowsPerPage={searchParam.get("page_size") || 10}
                    onRowsPerPageChange={(e) => {
                        setSearchParam({
                            page_size: e.target.value,
                            page: 0
                        }, loadConfigData)
                    }}
                    loading={loadData.loading}
                />
            </Box> : <Alert severity="error">{loadData.message}</Alert>}
    </Fragment>
}

