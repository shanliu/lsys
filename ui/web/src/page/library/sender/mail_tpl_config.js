

import { default as AddCircleOutlineIcon } from '@mui/icons-material/AddCircleOutline';
import DeleteIcon from '@mui/icons-material/Delete';
import SearchIcon from '@mui/icons-material/Search';
import { Alert, Button, Divider, Drawer, FormControl, Grid, IconButton, InputLabel, MenuItem, Paper, Select, Typography } from '@mui/material';
import Box from '@mui/material/Box';
import React, { Fragment, useEffect, useState } from 'react';
import { ConfirmButton } from '../../../library/dialog';
import { ClearTextField } from '../../../library/input';
import { LoadingButton } from '../../../library/loading';
import { SimpleTablePage } from '../../../library/table_page';
import { mailDelAppTplConfig, mailListTplConfig } from '../../../rest/sender_setting';
import { showTime } from '../../../utils/utils';
import { AppMailTplConfigSmtpAddBox, AppMailTplConfigSmtpShowDetail } from './mail/smtp';


export const MailSenderType = [
    {
        key: 'smtp-config',
        val: 'SMTP方式发送',
        showDetail: AppMailTplConfigSmtpShowDetail,
        addBox: AppMailTplConfigSmtpAddBox,
    }
];


function AddBox(props) {
    const { onAdd, appId, appName, userId } = props;
    const [addType, setAddType] = useState({
        sender_type: MailSenderType[0].key,
    });
    let addBox;
    let find = MailSenderType.find((item) => {
        return item.key == addType.sender_type
    });
    if (find) {
        addBox = find.addBox({
            onAdd: onAdd,
            appId: appId,
            userId: userId,
        });
    }
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
                添加{appName}模板关联
            </Typography>
            <Divider variant="middle" />
            <Grid
                sx={{
                    mt: 5,
                }}
                container
                justifyContent="center"
                alignItems="center"
            >
                <Grid item xs={10}>
                    <FormControl fullWidth sx={{
                        width: 1,
                        paddingBottom: 2
                    }}>
                        <InputLabel size="small" >选择邮件端口</InputLabel>
                        <Select
                            fullWidth
                            size='small'
                            value={addType.sender_type}
                            onChange={
                                (e) => {
                                    setAddType({
                                        ...addType,
                                        sender_type: e.target.value
                                    })
                                }
                            }
                            label="选择邮件端口">
                            {MailSenderType.map((item) => {
                                return <MenuItem key={`s-${item.key}`} value={item.key}>{item.val}</MenuItem>
                            })}
                        </Select>
                    </FormControl>
                </Grid>
                {addBox}
            </Grid>
        </Fragment >)
}



export default function AppMailTplConfig(props) {
    const {
        userId,
        appId,
        mapId,
        appName,
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
    let columns = [
        {
            label: 'ID',
            align: "right",
            style: { width: 80 },
            render: (row) => {
                return row.id
            }
        },
        {
            field: "app_id",
            style: { width: 120 },
            label: '应用ID',
            render: (row) => {
                return row.app_id
            }
        },
        {
            style: { width: 120 },
            label: '名称',
            render: (row) => {
                return row.name
            }
        },
        {

            style: { width: 120 },
            label: '模板名',
            render: (row) => {
                return row.tpl_id
            }
        },
        {
            style: { width: 360 },
            label: '端口详细',
            render: (row) => {
                let show = MailSenderType.find((item) => item.key == row.setting_key);
                if (show) return show.showDetail(row);
                return row.setting_name;
            }
        },
        {
            style: { width: 120 },
            label: '用户ID',
            align: "center",
            render: (row) => {
                return row.change_user_id
            }
        },
        {
            style: { width: 180 },
            label: '更新时间',
            render: (row) => {
                return showTime(row.change_time, "未知")
            }
        },
        {

            label: '操作',
            render: (row) => {
                let delAction = () => {
                    return mailDelAppTplConfig({ id: row.id }).then((data) => {
                        if (!data.status) return data;
                        let rows = loadData.data.filter((item) => {
                            if (item.config.id == row.id) return;
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
                        message={`确定删除关系 [${row.id}] 吗?`}
                        onAction={delAction}
                        renderButton={(props) => {
                            return <IconButton  {...props} size='small' ><DeleteIcon fontSize="small" /></IconButton>
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
            id: mapId,
        }, ...props.children ? { app_id: appId } : {}
    })
    const loadAppData = () => {
        setLoadData({
            ...loadData,
            loading: true
        })
        window.scrollTo({ top: 0, left: 0, behavior: 'smooth' });
        return mailListTplConfig({
            id: mapId,
            user_id: parseInt(userId),
            app_id: (props.children && !appId) ? -1 : appId,
            page: page || 0,
            page_size: pageSize || 25
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
                id: mapId,
            }, ...props.children ? { app_id: appId } : {}
        })
        loadAppData()
    }, [
        props.userId,
        props.appId,
        props.mapId,
        props.appName,
        props.page,
        props.pageSize,
    ])
    const [changeBoxState, setChangeBox] = useState(0);
    let showBox
    switch (changeBoxState) {
        case 1:
            showBox = <AddBox
                userId={parseInt(userId)}
                appId={(props.children && !appId) ? -1 : appId}
                appName={appName}
                onAdd={(id) => {
                    onSearchChange({
                        id: id,
                        app_id: appId,
                        page: 0
                    }, loadAppData)
                    setChangeBox(0)
                }} />;
            break;
    };

    return <Fragment>
        <Drawer
            sx={{ zIndex: (theme) => theme.zIndex.drawer + 3 }}
            anchor={"right"}
            open={changeBoxState != 0}
            onClose={() => {
                setChangeBox(0)
            }}
        >
            <Box
                sx={{ width: 450 }}
                role="presentation"
            >
                {showBox}
            </Box>
        </Drawer>
        <Box sx={{ m: 2 }}>
            <Paper
                sx={{ p: 2, display: 'flex', alignItems: 'center', marginBottom: 1 }}
            >
                {props.children}
                <FormControl sx={{ minWidth: 120, mr: 1 }} size="small"  >
                    <ClearTextField
                        sx={{ mr: 1 }}
                        variant="outlined"
                        label={`配置ID`}
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
                <LoadingButton
                    onClick={() => {
                        onSearchChange({
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
                {appName ? <Button
                    variant="outlined"
                    size="medium"
                    startIcon={<AddCircleOutlineIcon />}
                    sx={{ mr: 1, p: "7px 15px" }}
                    onClick={() => {
                        setChangeBox(1)
                    }}>
                    添加
                </Button> : null}
            </Paper>

            {(loadData.status || loadData.loading)
                ? <Box sx={{ height: 1, width: '100%' }}>
                    <SimpleTablePage
                        rows={loadData.data}
                        columns={columns}
                        count={loadData.total}
                        page={page || 0}
                        onPageChange={(e, newPage) => {
                            onSearchChange({
                                page: newPage
                            }, loadAppData)
                        }}
                        rowsPerPage={pageSize || 25}
                        onRowsPerPageChange={(e) => {
                            onSearchChange({
                                page_size: e.target.value,
                                page: 0
                            }, loadAppData)
                        }}
                        loading={loadData.loading}
                    />
                </Box> : <Alert severity="error">{loadData.message}</Alert>}
        </Box>
    </Fragment>
}


