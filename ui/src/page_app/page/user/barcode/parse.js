

import DeleteIcon from '@mui/icons-material/Delete';
import SearchIcon from '@mui/icons-material/Search';
import { Alert, Button, FormControl, IconButton, Paper, Stack, } from '@mui/material';
import Box from '@mui/material/Box';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { ConfirmButton } from '../../../../common/ui/dialog';
import { ClearTextField } from '../../../../library/input';
import { LoadingButton } from '../../../../library/loading';
import { SimpleTablePage } from '../../../../library/table_page';
import { showTime } from '../../../../common/utils/utils';
import { AppSelect } from '../../common/sender/lib_app_select';
import { barcodeParseDel, barcodeParseList, createBarcodeType } from '../../../../common/rest/barcode';
import { useSearchChange } from '../../../../common/utils/hook';
import { UserSessionContext } from '../../../../common/context/session';
import { ItemTooltip } from '../../../../library/tips';
import LogoDevIcon from '@mui/icons-material/LogoDev';
import HelpOutlineOutlinedIcon from '@mui/icons-material/HelpOutlineOutlined';


export default function UserAppBarCodeParsePage(props) {
    const { userData } = useContext(UserSessionContext)
    const [searchParam, setSearchParam] = useSearchChange({
        app_id: '',
        barcode_type: '',
        page: 0,
        page_size: 25,
    });
    const [filterData, setfilterData] = useState({
        barcode_type: '',
        app_id: '',
    })

    useEffect(() => {
        setfilterData({
            ...filterData,

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
        return barcodeParseList({
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
    }, [filterData.app_id, filterData.barcode_type])

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
            label: 'ID',
            align: "right",
            style: { width: 80 }
        },
        {
            field: 'app_id',
            style: { width: 100 },
            label: '应用ID',
        },
        {
            style: { width: 100 },
            label: '文件HASH',
            render: (row) => {
                return <ItemTooltip
                    placement="top"
                    title={row.hash}>
                    {row.hash.substr(0, 6)}
                </ItemTooltip>
            }
        },
        {

            label: '解析结果',
            render: (row) => {
                if (row.status) {
                    let f = createBarcodeType.find((e) => { return e.key == row.barcode_type });
                    return <Box>成功,数据:{f ? "[" + f.val + "]" : ""}:{row.text}</Box>
                } else {
                    return <Box>失败,原因:{row.error}</Box>
                }
            }
        },
        {
            style: { width: 180 },
            label: '解析时间',
            render: (row) => {
                return showTime(row.create_time, "未知")
            }
        },
        {
            style: { width: 100 },
            label: '操作',
            render: (row) => {
                let delAction = () => {
                    return barcodeParseDel({ id: row.id }).then((data) => {
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
                    <ConfirmButton
                        message={`确定删除 [${row.id}] 吗?`}
                        onAction={delAction}
                        renderButton={(props) => {
                            return <IconButton  {...props} size='small' ><DeleteIcon fontSize="small" /></IconButton>
                        }} />
                </Fragment>
            }
        }
    ];

    return <Fragment>
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
            <Button
                variant="outlined"
                size="medium"
                startIcon={<LogoDevIcon />}
                endIcon={<HelpOutlineOutlinedIcon fontSize='small' />}
                sx={{ mr: 1, p: "7px 15px", minWidth: 110 }}
                onClick={() => {
                    window.open("https://github.com/shanliu/lsys/tree/main/http/rest/rest_barcode.md", "_blank")
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
