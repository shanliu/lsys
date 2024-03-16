
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { Alert, Box, Divider, Drawer, Grid, IconButton, Link, Table, TextField, Typography } from '@mui/material';
import { senderGetMessageNotify, senderSetMessageNotify } from '../../../../common/rest/sender_setting';
import EditIcon from '@mui/icons-material/Edit';
import { LoadingButton } from '../../../../library/loading';
import { BaseTableBody, BaseTableHead } from '../../../../library/table_page';
import { Form, useNavigate, useSearchParams } from 'react-router-dom';
import { showTime } from '../../../../common/utils/utils';
import { Link as RouterLink } from 'react-router-dom';
import { ToastContext } from '../../../../common/context/toast';
export default function UserAppSmsNotifyPage(props) {
    const [pageData, setPageData] = useState({
        rows: [],
        loading: false,
        err: ''
    });


    const getData = () => {
        return senderGetMessageNotify("smser").then((data) => {

            if (!data.status) {
                setPageData({
                    ...pageData,
                    err: data.message
                })
            } else {
                setPageData({
                    ...pageData,
                    rows: data.data ?? [],
                })
            }
            return data;
        })
    };


    const [changeBoxState, setChangeBox] = useState({ show: 0, row: {} });


    const [searchParam, _] = useSearchParams();
    const checkOpenBox = () => {
        let app_id = searchParam.get("app_id") ?? 0;
        if (app_id > 0) {
            pageData.rows.map((t) => {
                if (t.app_id == app_id) {
                    setChangeBox({
                        show: 1,
                        row: t
                    })
                }
            })
        }
    }

    useEffect(() => {
        checkOpenBox();
    }, [pageData.rows])

    useEffect(() => {
        getData()
    }, [])


    useEffect(() => {
        checkOpenBox();
    }, [searchParam])


    let columns = [
        {
            field: 'app_id',
            style: { width: 120 },
            label: '应用ID',
        },
        {
            field: 'app_name',
            style: { width: 240 },
            label: '应用名',
        },
        {

            label: '回调地址',
            render: (row) => {
                if (!row.call_url) {
                    return "未设置"
                } else {
                    return row.call_url;
                }
            }
        },
        {
            style: { width: 180 },
            label: '最后修改时间',
            render: (row) => {
                return showTime(row.change_time, "")
            }
        },
        {
            label: '操作',
            render: (row) => {
                return <Link component={RouterLink} to={`/user/sms/notify?app_id=${row.app_id}`}>
                    <IconButton size='small' ><EditIcon fontSize="small" /></IconButton>
                </Link >

            }
        }
    ];

    let showBox
    switch (changeBoxState.show) {
        case 1:
            showBox = <AddBox
                row={changeBoxState.row}
            />;
            break;
    };
    const navigate = useNavigate();

    return <Fragment>
        <Drawer
            sx={{ zIndex: (theme) => theme.zIndex.drawer + 3 }}
            anchor={"right"}
            open={changeBoxState.show != 0}
            onClose={() => {
                setChangeBox({ show: 0 })
                getData()
                navigate("/user/sms/notify")
            }}
        >
            <Box
                sx={{ width: 450 }}
                role="presentation"
            >
                {showBox}
            </Box>
        </Drawer>

        {pageData.loading ? <Progress /> : (!pageData.err ? <Box sx={{ height: 1, width: '100%' }}>
            <Table>
                <BaseTableHead
                    columns={columns}
                />
                <BaseTableBody
                    columns={columns}
                    loading={pageData.loading}
                    rows={pageData.rows}
                />
            </Table>
        </Box> : <Alert severity="error">{pageData.err}</Alert>)}
    </Fragment>
}




function AddBox(props) {
    const { row } = props;
    const { toast } = useContext(ToastContext);
    const [configData, setConfigData] = useState({
        url: row ? row.call_url : '',
        loading: false,
    });
    const doAdd = function () {
        setConfigData({
            ...configData,
            loading: true
        })
        senderSetMessageNotify("smser", {
            url: configData.url,
            app_id: row.app_id,
        }).then((data) => {
            if (!data.status) {
                toast(data.message)
                setConfigData({
                    ...configData,
                    loading: false,
                })
            } else {
                toast("已保存")
                setConfigData({
                    ...configData,
                    loading: false,
                })

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
                配置{row.app_name}回调地址
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
                            label={"输入回调地址"}
                            type="text"
                            name="name"
                            size="small"
                            onChange={(e) => {
                                setConfigData({
                                    ...configData,
                                    url: e.target.value ?? '',
                                })
                            }}
                            value={configData.url}
                            sx={{
                                width: 1,
                                paddingBottom: 2
                            }}
                            required
                            disabled={configData.loading}
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
