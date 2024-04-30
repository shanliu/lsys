
import { default as AddCircleOutlineIcon } from '@mui/icons-material/AddCircleOutline';
import DeleteIcon from '@mui/icons-material/Delete';
import SearchIcon from '@mui/icons-material/Search';
import { Alert, Button, Dialog, DialogActions, DialogContent, DialogTitle, Divider, Drawer, FormControl, Grid, IconButton, InputLabel, MenuItem, Paper, Select, Stack, TextField, Typography } from '@mui/material';
import Box from '@mui/material/Box';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { Form } from 'react-router-dom';
import { ToastContext } from '../../../../common/context/toast';
import { ConfirmButton } from '../../../../common/ui/dialog';
import { ClearTextField } from '../../../../library/input';
import { LoadingButton } from '../../../../library/loading';
import { SimpleTablePage } from '../../../../library/table_page';
import { showTime } from '../../../../common/utils/utils';
import { AppSelect } from '../../common/sender/lib_app_select';
import { barcodeCreateAdd, barcodeCreateDel, barcodeCreateEdit, barcodeCreateList, createBarcodeType, createImageFormat, createStatus } from '../../../../common/rest/barcode';
import { useSearchChange } from '../../../../common/utils/hook';
import { UserSessionContext } from '../../../../common/context/session';
import EditIcon from '@mui/icons-material/Edit';
import { MuiColorInput } from 'mui-color-input';
import { ItemTooltip } from '../../../../library/tips';
import QrCodeIcon from '@mui/icons-material/QrCode';
import HelpOutlineOutlinedIcon from '@mui/icons-material/HelpOutlineOutlined';
import LogoDevIcon from '@mui/icons-material/LogoDev';
function CreateBox(props) {
    const {
        appId,
        appName,
        rowData,
        onFinish
    } = props;

    const { toast } = useContext(ToastContext);

    let [addData, setAddData] = useState({
        status: rowData ? rowData.status : '',
        image_height: rowData ? rowData.image_height : '',
        image_width: rowData ? rowData.image_width : '',
        margin: rowData ? rowData.margin : '',
        image_format: rowData ? rowData.image_format : '',
        barcode_type: rowData ? rowData.barcode_type : '',
        image_background: rowData ? ('#' + rowData.image_background) : '#FFFFFF',
        image_color: rowData ? ('#' + rowData.image_color) : '#000000',
        loading: false,
    });
    const [addError, setAddError] = useState({
        image_height: '',
        image_width: '',
        margin: '',
        image_format: '',
        barcode_type: '',
        image_background: '',
        image_color: '',
    });

    let onSubmit = () => {
        setAddData({
            ...addData,
            loading: true
        })
        if (rowData && rowData.id) {
            barcodeCreateEdit({
                id: rowData.id,
                barcode_type: addData.barcode_type,
                status: addData.status,
                image_format: addData.image_format,
                image_width: addData.image_width,
                image_height: addData.image_height,
                margin: addData.margin,
                image_color: addData.image_color,
                image_background: addData.image_background
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
                        image_height: '',
                        image_width: '',
                        margin: '',
                        image_format: '',
                        barcode_type: '',
                        image_background: '',
                        image_color: '',
                    })
                    setAddData({
                        name: '',
                        tpl_data: '',
                        tpl_id: '',
                        loading: false
                    })
                    onFinish(rowData.id);
                }
            })
        } else {
            barcodeCreateAdd({
                app_id: appId,
                barcode_type: addData.barcode_type,
                status: addData.status,
                image_format: addData.image_format,
                image_width: addData.image_width,
                image_height: addData.image_height,
                margin: addData.margin,
                image_color: addData.image_color,
                image_background: addData.image_background
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
                        image_height: '',
                        image_width: '',
                        margin: '',
                        image_format: '',
                        barcode_type: '',
                        image_background: '',
                        image_color: '',
                    })
                    setAddData({
                        ...addData,
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
                应用{appName}-条码{rowData ? "编辑" : "添加"}
            </Typography>
            <Divider variant="middle" />
            <Form method="post" onSubmit={(e) => {
                e.preventDefault();
                onSubmit()
            }}>
                <Grid
                    container
                    justifyContent="center"
                    alignItems="center" sx={{
                        mt: 2
                    }}
                >
                    <Grid item xs={10} sx={{
                        marginBottom: 2
                    }}>
                        <FormControl size="small" fullWidth>
                            <InputLabel >是否公开</InputLabel>
                            <Select
                                label="是否公开"
                                fullWidth
                                size='small'
                                value={addData.status}
                                onChange={
                                    (e) => {
                                        setAddData({
                                            ...addData,
                                            status: e.target.value
                                        })
                                    }
                                }>
                                {createStatus.map((item) => {
                                    return <MenuItem key={`s-${item.key}`} value={item.key}>{item.val}</MenuItem>
                                })}
                            </Select>
                        </FormControl>
                    </Grid>
                    <Grid item xs={10} sx={{
                        marginBottom: 2
                    }}>
                        <FormControl fullWidth size="small" >
                            <InputLabel  >条码类型</InputLabel>
                            <Select
                                fullWidth
                                size='small'
                                value={addData.barcode_type}
                                onChange={
                                    (e) => {
                                        setAddData({
                                            ...addData,
                                            barcode_type: e.target.value
                                        })
                                    }
                                }
                                label="条码类型">
                                {createBarcodeType.map((item) => {
                                    return <MenuItem key={`s-${item.key}`} value={item.key}>{item.val}</MenuItem>
                                })}
                            </Select>
                        </FormControl>
                    </Grid>
                    <Grid item xs={10} sx={{
                        marginBottom: 2
                    }}>
                        <FormControl fullWidth size="small">
                            <InputLabel >输出格式</InputLabel>
                            <Select
                                fullWidth
                                size='small'
                                value={addData.image_format}
                                onChange={
                                    (e) => {
                                        setAddData({
                                            ...addData,
                                            image_format: e.target.value
                                        })
                                    }
                                }
                                label="输出格式">
                                {createImageFormat.map((item) => {
                                    return <MenuItem key={`s-${item.key}`} value={item.key}>{item.val}</MenuItem>
                                })}
                            </Select>
                        </FormControl>
                    </Grid>
                    <Grid item xs={10} sx={{
                        marginBottom: 2
                    }}>
                        <TextField
                            variant="outlined"
                            label="图片宽度"
                            type="number"
                            size="small"
                            onChange={(e) => {
                                setAddData({
                                    ...addData,
                                    image_width: e.target.value
                                })
                            }}
                            value={addData.image_width}
                            sx={{
                                width: 1
                            }}
                            required
                            disabled={addData.loading}
                            error={!!addError.image_width}
                            helperText={addError.image_width}
                        />
                    </Grid>
                    <Grid item xs={10} sx={{
                        marginBottom: 2
                    }}>
                        <TextField
                            variant="outlined"
                            label="图片宽度"
                            type="number"
                            size="small"
                            onChange={(e) => {
                                setAddData({
                                    ...addData,
                                    image_height: e.target.value
                                })
                            }}
                            value={addData.image_height}
                            sx={{
                                width: 1
                            }}
                            required
                            disabled={addData.loading}
                            error={!!addError.image_height}
                            helperText={addError.image_height}
                        />
                    </Grid>
                    <Grid item xs={10} sx={{
                        marginBottom: 2
                    }}>
                        <TextField
                            variant="outlined"
                            label="图片边距"
                            type="number"
                            size="small"
                            onChange={(e) => {
                                setAddData({
                                    ...addData,
                                    margin: e.target.value
                                })
                            }}
                            value={addData.margin}
                            sx={{
                                width: 1
                            }}
                            required
                            disabled={addData.loading}
                            error={!!addError.margin}
                            helperText={addError.margin}
                        />
                    </Grid>
                    <Grid item xs={10} sx={{
                        marginBottom: 2
                    }}>
                        <MuiColorInput
                            variant="outlined"
                            label="条码颜色"
                            format="hex"
                            size="small"
                            onChange={(value) => {
                                setAddData({
                                    ...addData,
                                    image_color: value
                                })
                            }}
                            value={addData.image_color}
                            sx={{
                                width: 1
                            }}
                            required
                            disabled={addData.loading}
                            error={!!addError.image_color}
                            helperText={addError.image_color}
                        />
                    </Grid>
                    <Grid item xs={10} sx={{
                        marginBottom: 2
                    }}>
                        <MuiColorInput
                            variant="outlined"
                            label="背景颜色"
                            format="hex"
                            size="small"
                            onChange={(value) => {
                                setAddData({
                                    ...addData,
                                    image_background: value
                                })
                            }}
                            value={addData.image_background}
                            sx={{
                                width: 1
                            }}
                            required
                            disabled={addData.loading}
                            error={!!addError.image_background}
                            helperText={addError.image_background}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <LoadingButton sx={{
                            width: 1,
                        }} variant="contained" type="submit" loading={addData.loading} disabled={addData.loading} >{rowData ? "修改" : "添加"}</LoadingButton>
                    </Grid>
                </Grid>
            </Form ></Fragment >)
}


function CreateShowBox(props) {
    const { id, urlPrefix, renderButton, ...other } = props
    const [showBox, setShowBox] = useState({
        open: false,
        data: ''
    });
    return <Fragment>
        <Dialog
            open={showBox.open}
            onClose={() => { setShowBox({ ...showBox, open: false }) }}
        >
            <DialogTitle>预览二维码,code_id:{id}</DialogTitle>
            <DialogContent style={{
                width: 500
            }} >
                <Stack sx={{
                    mt: 1
                }}>
                    <TextField
                        onChange={(e) => {
                            setShowBox({ ...showBox, data: e.target.value })
                        }}
                        value={setShowBox.data}
                        size="small"
                        label={`二维码内容`}
                        type='text'
                        fullWidth
                        multiline
                        minRows={1}
                    />

                    {(showBox.data && showBox.data.length > 0) ?
                        <Fragment>
                            <Grid
                                sx={{
                                    mt: 2,
                                    mb: 2
                                }}
                                container
                                direction="row"
                                justifyContent="center"
                                alignItems="center"
                                fullWidth
                            >  <img style={{ maxWidth: 400, border: "1px solid #eee", padding: 1, margin: 4 }} src={urlPrefix + showBox.data} />

                            </Grid>

                            <Typography style={{ textAlign: "center" }} variant="caption" display="block" gutterBottom>
                                {urlPrefix + showBox.data}
                            </Typography>
                        </Fragment> : null}
                </Stack>
            </DialogContent>
            <DialogActions>
                <Button onClick={() => { setShowBox({ ...showBox, open: false }) }} >
                    关闭
                </Button>
            </DialogActions>
        </Dialog>
        {renderButton({
            ...other,
            onClick: () => {
                setShowBox({ ...showBox, open: true })
            }
        })}
    </Fragment>
}

export default function UserAppBarCodeCreatePage(props) {
    const { userData } = useContext(UserSessionContext)
    const [searchParam, setSearchParam] = useSearchChange({
        id: '',
        app_id: '',
        barcode_type: '',
        page: 0,
        page_size: 25,
    });
    const [filterData, setfilterData] = useState({
        id: "",
        app_id: "",
        app_name: '',
        barcode_type: ''
    })
    useEffect(() => {
        setfilterData({
            ...filterData,
            id: searchParam.get("id"),
            app_id: searchParam.get("app_id"),
            barcode_type: searchParam.get("barcode_type")
        })
    }, [searchParam])
    const loadAppData = () => {
        setLoadData({
            ...loadData,
            loading: true
        })
        window.scrollTo({ top: 0, left: 0, behavior: 'smooth' });
        return barcodeCreateList({
            id: searchParam.get("id"),
            app_id: searchParam.get("app_id"),
            barcode_type: searchParam.get("barcode_type"),
            page: searchParam.get("page"),
            page_size: searchParam.get("page_size"),
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
        loadAppData()
    }, [filterData.app_id, filterData.barcode_type, filterData.id])

    let [loadData, setLoadData] = useState({
        status: false,
        message: null,
        loading: true,
        data: [],
        total: 0,
    });
    let columns = [
        {
            field: 'id',
            label: 'code_id',
            align: "right",
            style: { width: 80 }
        },
        {
            field: 'app_id',
            style: { width: 120 },
            label: '应用ID',
        },
        {
            style: { width: 120 },
            label: '状态',
            render: (row) => {
                let f = createStatus.find((e) => { return e.key == row.status });
                if (!f) {
                    return "未知状态";
                } else {
                    return f.val;
                }
            }
        },
        {
            style: { width: 100 },
            label: '类型',
            render: (row) => {
                let f = createBarcodeType.find((e) => { return e.key == row.barcode_type });
                if (!f) {
                    return "未知类型";
                } else {
                    return f.val;
                }
            }
        },
        {
            style: { width: 100 },
            label: '格式',
            render: (row) => {
                let f = createImageFormat.find((e) => { return e.key == row.image_format });
                if (!f) {
                    return "未知";
                } else {
                    return f.val;
                }
            }
        },
        {
            style: { width: 140 },
            label: '宽高(px)',
            render: (row) => {
                return row.image_width + "x" + row.image_height + ":" + row.margin
            }
        },
        {
            style: { width: 120 },
            label: '颜色',
            render: (row) => {
                return <Stack direction="row" spacing={1}>
                    <ItemTooltip
                        placement="top"
                        title="条码颜色">
                        <Stack
                            direction="row" justifyContent="center"
                            alignItems="center"
                        >

                            <Box sx={{ backgroundColor: "#" + row.image_color, width: "12px", height: "12px", border: "1px solid #ccc", mr: "2px" }}></Box>


                            <Box>#{row.image_color}</Box>


                        </Stack>
                    </ItemTooltip>
                    <ItemTooltip
                        placement="top"
                        title="背景颜色">
                        <Stack direction="row" justifyContent="center"
                            alignItems="center"
                        >
                            <Box sx={{ backgroundColor: "#" + row.image_background, width: "12px", height: "12px", border: "1px solid #ccc", mr: "2px" }}></Box>
                            <Box>#{row.image_background}</Box>
                        </Stack>
                    </ItemTooltip>
                </Stack >
            }
        },
        {
            style: { width: 180 },
            label: '设置时间',
            render: (row) => {
                return showTime(row.change_time, "未知")
            }
        },
        {
            label: '操作',
            render: (row) => {
                let delAction = () => {
                    return barcodeCreateDel({ id: row.id }).then((data) => {
                        if (!data.status) return data;
                        let rows = loadData.data.filter((item) => {
                            if (item.id == row.id) return;
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
                        setChangeBox({ show: 'edit', rowData: row })
                    }}>
                        <EditIcon fontSize="small" />
                    </IconButton>
                    <ConfirmButton
                        message={`确定删除 [${row.id}] 吗?`}
                        onAction={delAction}
                        renderButton={(props) => {
                            return <IconButton  {...props} size='small' ><DeleteIcon fontSize="small" /></IconButton>
                        }} />
                    {row.status == 2 ? <CreateShowBox
                        id={row.id}
                        urlPrefix={row.url}
                        renderButton={(props) => {
                            return <IconButton  {...props} size='small' ><QrCodeIcon fontSize="small" /></IconButton>
                        }}
                    /> : <IconButton  {...props} size='small' onClick={() => {
                        window.open("https://github.com/shanliu/lsys/tree/main/http/rest/rest_barcode.md", "_blank")
                    }} ><LogoDevIcon size='small' fontSize="接口示例" /></IconButton>
                    }

                </Fragment>
            }
        }
    ];

    const [changeBoxState, setChangeBox] = useState({ show: 0, rowData: null });

    let showBox
    switch (changeBoxState.show) {
        case 'add':
            showBox = <CreateBox
                rowData={null}
                appId={filterData.app_id}
                appName={filterData.app_name}
                onFinish={(id) => {
                    setSearchParam({
                        id: id,
                        app_id: filterData.app_id,
                        page: 0
                    }, loadAppData)
                    setChangeBox({ show: '', rowData: null })
                }} />;
            break;
        case 'edit':
            showBox = <CreateBox
                rowData={changeBoxState.rowData}
                appId={filterData.app_id}
                appName={filterData.app_name}
                onFinish={(id) => {
                    setSearchParam({
                        id: id,
                        app_id: filterData.app_id,
                        page: 0
                    }, loadAppData)
                    setChangeBox({ show: '', rowData: null })
                }} />;
            break;
    };

    return <Fragment>
        <Drawer
            sx={{ zIndex: (theme) => theme.zIndex.drawer + 3 }}
            anchor={"right"}
            open={changeBoxState.show != ''}
            onClose={() => {
                setChangeBox({ show: '', rowData: null })
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
            <AppSelect
                sx={{
                    width: 200,
                    marginRight: 1
                }}
                urlParam={{
                    check_barcode: true
                }}
                accCheck={(item) => item.is_barcode}
                userId={parseInt(userData.user_data.user_id)}
                appId={searchParam.get("app_id") ?? ''}
                onLoad={(data) => {
                    setfilterData({
                        ...filterData,
                        app_id: data.id,
                        app_name: data.name
                    })
                }}
                onChange={(e) => {
                    setSearchParam({
                        app_id: e.target.value,
                        page: 0
                    })
                }}
            />
            <FormControl sx={{ minWidth: 120, mr: 1 }} size="small"  >
                <ClearTextField
                    sx={{ mr: 1 }}
                    variant="outlined"
                    label={`ID`}
                    type="text"
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
            <FormControl sx={{ minWidth: 120, mr: 1 }} size="small"  >
                <ClearTextField
                    sx={{ mr: 1 }}
                    variant="outlined"
                    label={`类型`}
                    type="text"
                    value={filterData.barcode_type}
                    size="small"
                    disabled={loadData.loading}
                    onChange={(event, nval) => {
                        setfilterData({
                            ...filterData,
                            barcode_type: nval
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
            {filterData.app_id > 0 ? <Button
                variant="outlined"
                size="medium"
                startIcon={<AddCircleOutlineIcon />}
                sx={{ mr: 1, p: "7px 15px", minWidth: 110 }}
                onClick={() => {
                    setChangeBox({ show: 'add', rowData: null })
                }}>
                添加
            </Button> : null}
            <Button
                variant="outlined"
                size="medium"
                startIcon={<LogoDevIcon />}
                endIcon={<HelpOutlineOutlinedIcon fontSize='small' />}
                sx={{ mr: 1, p: "7px 15px", minWidth: 110 }}
                onClick={() => {
                    window.open("https://github.com/shanliu/lsys/blob/main/http/rest/rest_barcode.md#%E4%BA%8C%E7%BB%B4%E7%A0%81%E5%88%9B%E5%BB%BA", "_blank")
                }}>
                请求示例
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