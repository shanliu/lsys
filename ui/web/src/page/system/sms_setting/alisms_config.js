
import AddCircleOutlineIcon from '@mui/icons-material/AddCircleOutline';
import SearchIcon from '@mui/icons-material/Search';
import DeleteIcon from '@mui/icons-material/Delete';
import EditIcon from '@mui/icons-material/Edit';
import { Alert, Button, Divider, Drawer, FormControl, Grid, IconButton, Paper, Table, TableContainer, TextField, Typography } from '@mui/material';
import Box from '@mui/material/Box';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { Form } from 'react-router-dom';
import { UserSessionContext } from '../../../context/session';
import { ToastContext } from '../../../context/toast';
import { ConfirmButton } from '../../../library/dialog';
import { ClearTextField } from '../../../library/input';
import { LoadingButton } from '../../../library/loading';
import { BaseTableBody, BaseTableHead } from '../../../library/table_page';
import { addAliConfig, delAliConfig, editAliConfig, listAliConfig } from '../../../rest/sms_setting';
import { useSearchChange } from '../../../utils/hook';
import { showTime } from '../../../utils/utils';




function AddBox(props) {
    const {
        rowData,
       onFinish
    } = props;

    const { toast } = useContext(ToastContext);
    
    let [addData, setAddData] = useState({
        name: rowData?rowData.name:'',
        access_id: rowData?rowData.access_id:'',
        access_secret: rowData?rowData.access_secret:'',
        loading: false,
    });
    const [addError, setAddError] = useState({
        name: '',
        access_id: '',
        access_secret: '',
    });

    let onSubmit=()=>{
        setAddData({
            ...addData,
            loading: true
        })
        if(rowData&&rowData.id){
            editAliConfig({
                id:rowData.id,
                name: addData.name,
                access_id: addData.access_id,
                access_secret: addData.access_secret
            }) .then((data) => {
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
                        access_id: '',
                        access_secret: '',
                    })
                    setAddData({
                        ...addData,
                        loading: false
                    })
                    onFinish(rowData.id);
                }
            })
        }else{
            addAliConfig({
                name: addData.name,
                access_id: addData.access_id,
                access_secret: addData.access_secret
            }) .then((data) => {
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
                        access_id: '',
                        access_secret: '',
                    })
                    setAddData({
                        name: '',
                        access_id: '',
                        access_secret: '',
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
               阿里短信配置
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
                            onChange={(e)=>{
                                setAddData({
                                    ...addData,
                                    name:e.target.value
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
                            label="阿里云 access id"
                            type="text"
                            name="name"
                            size="small"
                            onChange={(e)=>{
                                setAddData({
                                    ...addData,
                                    access_id:e.target.value
                                })
                            }}
                            value={addData.access_id}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            disabled={addData.loading}
                            error={!!addError.access_id}
                            helperText={addError.access_id}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <TextField
                            variant="outlined"
                            label="阿里云 access secret"
                            type="text"
                            name="name"
                            size="small"
                            onChange={(e)=>{
                                setAddData({
                                    ...addData,
                                    access_secret:e.target.value
                                })
                            }}
                            value={addData.access_secret}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            disabled={addData.loading}
                            error={!!addError.access_secret}
                            helperText={addError.access_secret}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <LoadingButton sx={{
                            width: 1,
                        }} variant="contained" type="submit" loading={addData.loading} disabled={addData.loading} >{rowData?"修改":"添加"}</LoadingButton>
                    </Grid>
                </Grid>
            </Form ></Fragment>)
}


export default function SystemSmsSettingAlismsPage(props) {
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
            field: 'access_id',
            style: { width: 220 },
            label: 'access id',
        },
        {
            field: 'access_secret',
            style: { width: 240 },
            label: 'access secret',
        },
        {
            field: 'add_time',
            style: { width: 180 },
            label: '添加时间',
            render: (row) => {
                return showTime(row.add_time, "未知")
            }
        },
        {
            label: '操作',
            align: "center",
            render: (row) => {
                let delAction = () => {
                    return delAliConfig({ id: row.id }).then((data) => {
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
                    <IconButton size='small' onClick={()=>{
                        setChangeBox({data:row,show:2})
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
        return listAliConfig({
            id: searchParam.get("id"),
            full_data:true
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
            ...filterData,
            id: searchParam.get("id")
        })
        loadConfigData()
    }, [searchParam])


      //添加跟更新
    const [changeBoxState, setChangeBox] = useState({
        show:0,
        data:{}
    });
    let showBox
    switch (changeBoxState.show) {
        case 1:
            showBox = <AddBox
                onFinish={(id) => {
                    setChangeBox({data:{},show:0})
                    setSearchParam({
                        ...filterData,
                        id:id
                    }, loadConfigData)
                }}
            />;
            break
        case 2:
            showBox = <AddBox
            rowData={changeBoxState.data}
                onFinish={(id) => {
                    setChangeBox({data:{},show:0})
                    setSearchParam({
                        ...filterData,
                        id:id
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
                setChangeBox({data:{},show:0})
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
                sx={{ mr: 1, p: "7px 15px" }}
                loading={loadData.loading}
            >
                过滤
            </LoadingButton>
            <Button
                variant="outlined"
                size="medium"
                startIcon={<AddCircleOutlineIcon/>}
                sx={{ mr: 1, p: "7px 15px" }}
                onClick={() => {
                    setChangeBox({data:{},show:1})
                }}>
                新增配置
            </Button>
        </Paper>

        {(loadData.status || loadData.loading)
            ? <Box sx={{ height: 1, width: '100%' }}>
                <TableContainer component={Paper}>
                    <Table>
                        <BaseTableHead
                            columns={columns}
                        />
                        <BaseTableBody
                            columns={columns}
                            loading={loadData.loading}
                            rows={loadData.data??[]}
                        />
                    </Table>
                </TableContainer>
            </Box> : <Alert severity="error">{loadData.message}</Alert>}
    </Fragment>
}


