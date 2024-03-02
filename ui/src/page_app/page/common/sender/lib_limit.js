
import { default as AddCircleOutlineIcon } from '@mui/icons-material/AddCircleOutline';
import DeleteIcon from '@mui/icons-material/Delete';
import SearchIcon from '@mui/icons-material/Search';
import { Alert, Button, Divider, Drawer, FormControl, Grid, IconButton, InputLabel, MenuItem, Paper, Select, TextField, Typography } from '@mui/material';
import Box from '@mui/material/Box';
import { Stack } from '@mui/system';
import React, { Fragment, useContext, useEffect, useState } from 'react';
import { Form } from 'react-router-dom';
import { ToastContext } from '../../../../common/context/toast';
import { ConfirmButton } from '../../../../common/ui/dialog';
import { ClearTextField, SliderInput } from '../../../../library/input';
import { LoadingButton } from '../../../../library/loading';
import { SimpleTablePage } from '../../../../library/table_page';
import { senderAddConfig, senderDelConfig, senderListConfig } from '../../../../common/rest/sender_setting';
import { showTime } from '../../../../common/utils/utils';

function RuleBox(props) {
    const { loading, priority, configType, configData, onChange, limitMapData } = props;
    const [ruleData, setRuleData] = useState({
        config_data: configData,
        config_type: configType,
        priority: priority
    });
    useEffect(() => {
        onChange(ruleData)
    }, [ruleData])
    return <Stack>
        <SliderInput
            fullWidth
            sx={{
                width: 1,
                mb: 2,
                mt: 2,
                padding: "0 16px",
                textAlign: "center"
            }}
            label="优先级"
            loading={loading}
            value={ruleData.priority}
            onChange={(e) => {
                setRuleData({
                    ...ruleData,
                    priority: e.target.value
                })
            }}
        />
        <FormControl sx={{
            mb: 2,
        }}  >
            <InputLabel size="small" id="sender-limit-select-label">类型</InputLabel>
            <Select
                disabled={loading}
                size="small"
                labelId="sender-limit-select-label"
                id="sender-limit-select"
                label="类型"
                onChange={(e) => {
                    setRuleData({
                        ...ruleData,
                        config_type: e.target.value,
                        config_data: {}
                    })
                }}
                value={ruleData.config_type}
            >
                {limitMapData.map((item) => { return <MenuItem key={`sender-limit-type-${item.key}`} value={item.key}>{item.val}</MenuItem> })}
            </Select>
        </FormControl>

        {ruleData.config_type == 2 ? <Fragment>
            <TextField
                variant="outlined"
                label={`间隔时间:秒`}
                type="number"
                size="small"
                onChange={(e) => {
                    let val = parseInt(e.target.value);
                    if (val <= 0) return
                    setRuleData({
                        ...ruleData,
                        config_data: {
                            range_time: val,
                            max_send: ruleData.config_data.max_send ?? ''
                        }
                    })
                }}
                value={ruleData.config_data.range_time ?? ''}
                sx={{
                    width: 1,
                    paddingBottom: 2
                }}
                required
            />
            <TextField
                variant="outlined"
                label={`间隔可发送次数`}
                type="number"
                size="small"
                onChange={(e) => {
                    let val = parseInt(e.target.value);
                    if (val <= 0) return
                    setRuleData({
                        ...ruleData,
                        config_data: {
                            range_time: ruleData.config_data.range_time ?? '',
                            max_send: val
                        }
                    })
                }}
                value={ruleData.config_data.max_send ?? ''}
                sx={{
                    width: 1,
                    paddingBottom: 2
                }}
                required
            />
        </Fragment> : null}
        {ruleData.config_type == 10 ? <Fragment>
            <TextField
                variant="outlined"
                label={`需屏蔽的手机号`}
                type="text"
                size="small"
                onChange={(e) => {
                    setRuleData({
                        ...ruleData,
                        config_data: e.target.value
                    })
                }}
                value={typeof ruleData.config_data == 'string' ? ruleData.config_data : ''}
                sx={{
                    width: 1,
                    paddingBottom: 2
                }}
                required
            />
        </Fragment> : null}
        {ruleData.config_type == 4 ? <Fragment>
            <TextField
                variant="outlined"
                label={`忽略限制的模板`}
                type="text"
                size="small"
                onChange={(e) => {
                    setRuleData({
                        ...ruleData,
                        config_data: e.target.value
                    })
                }}
                sx={{
                    width: 1,
                    paddingBottom: 2
                }}
                value={typeof ruleData.config_data == 'string' ? ruleData.config_data : ''}
                required
            />
        </Fragment> : null}
        {ruleData.config_type == 3 ? <Fragment>
            <TextField
                variant="outlined"
                label={`批量发送最大数量`}
                type="number"
                size="small"
                onChange={(e) => {
                    let val = parseInt(e.target.value);
                    if (val <= 0) return
                    setRuleData({
                        ...ruleData,
                        config_data: val
                    })
                }}
                value={typeof ruleData.config_data == 'object' ? '' : ruleData.config_data}
                sx={{
                    width: 1,
                    paddingBottom: 2
                }}
                required
            />
        </Fragment> : null}
        {ruleData.config_type == 20 ? <Fragment>
            <TextField
                variant="outlined"
                label={`屏蔽指定邮箱`}
                type="text"
                size="small"
                onChange={(e) => {
                    setRuleData({
                        ...ruleData,
                        config_data: e.target.value
                    })
                }}
                value={typeof ruleData.config_data == 'string' ? ruleData.config_data : ''}
                sx={{
                    width: 1,
                    paddingBottom: 2
                }}
                required
            />
        </Fragment> : null}
        {ruleData.config_type == 21 ? <Fragment>
            <TextField
                variant="outlined"
                label={`屏蔽指定邮箱域名`}
                type="text"
                size="small"
                onChange={(e) => {
                    setRuleData({
                        ...ruleData,
                        config_data: e.target.value
                    })
                }}
                value={typeof ruleData.config_data == 'string' ? ruleData.config_data : ''}
                sx={{
                    width: 1,
                    paddingBottom: 2
                }}
                required
            />
        </Fragment> : null}
    </Stack>
}
function AddBox(props) {
    const { onAdd, userId, appId, appName, limitType, limitMapData } = props;
    const { toast } = useContext(ToastContext);
    const [configData, setConfigData] = useState({
        data: {},
        loading: false,
    });
    const doAdd = function () {
        setConfigData({
            ...configData,
            loading: true
        })
        senderAddConfig(limitType, {
            user_id: userId,
            app_id: appId,
            ...configData.data
        }).then((data) => {
            if (!data.status) {
                toast(data.message)
                setConfigData({
                    ...configData,
                    loading: false,
                })
            } else {
                setConfigData({
                    ...configData,
                    data: {},
                    loading: false,
                })
                onAdd(data.id)
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
                {appName} 新增限额配置
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
                        <RuleBox
                            limitMapData={limitMapData}
                            onChange={(data) => {
                                setConfigData({
                                    ...configData,
                                    data: data
                                })
                            }}
                            loading={configData.loading}
                            priority={50}
                            configType={1}
                            configData={{}}
                        />
                    </Grid>
                    <Grid item xs={10}>
                        <LoadingButton sx={{
                            width: 1,
                        }} variant="contained" type="submit">添加</LoadingButton>
                    </Grid>
                </Grid>
            </Form >
        </Fragment>)
}


export function SenderLimit(props) {
    const {
        limitType,
        limitMapData,
        userId,
        appId,
        appName,
        limitId,
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
            field: 'id',
            label: 'ID',
            align: "right",
            style: { width: 80 }
        },
        {
            field: 'app_id',
            style: { width: 120 },
            label: '应用ID',
        },
        {
            style: { width: 180 },
            label: '类型',
            render: (row) => {
                let f = limitMapData.find((e) => { return e.key == row.config_type });
                if (!f) {
                    return "未知类型";
                } else {
                    return f.val;
                }
            }
        },
        {
            style: { width: 260 },
            label: '相关数据',
            render: (row) => {
                let f = limitMapData.find((e) => { return e.key == row.config_type });
                if (!f) {
                    return "未知类型";
                } else {
                    return f.show(row.config_data);
                }
            }
        },
        {
            field: 'priority',
            style: { width: 80 },
            label: '优先级'
        },
        {
            style: { width: 180 },
            label: '添加时间',
            render: (row) => {
                return showTime(row.add_time, "未知")
            }
        },
        {
            label: '操作',
            render: (row) => {
                let delAction = () => {
                    return senderDelConfig(limitType, { config_id: row.id }).then((data) => {
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
                        message={`确定删除限制配置 [${row.id}] 吗?`}
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
            id: limitId,
        }, ...props.children ? { app_id: appId } : {}
    })
    const loadAppData = () => {
        setLoadData({
            ...loadData,
            loading: true
        })
        window.scrollTo({ top: 0, left: 0, behavior: 'smooth' });
        return senderListConfig(limitType, {
            user_id: parseInt(userId),
            id: limitId,
            app_id: (props.children && !appId) ? -1 : appId,
            page: page,
            page_size: pageSize
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
                id: limitId,
            }, ...props.children ? { app_id: appId } : {}
        })
        loadAppData()
    }, [
        props.limitType,
        props.userId,
        props.appId,
        props.appName,
        props.limitId,
        props.page,
        props.pageSize,
    ])

    const [changeBoxState, setChangeBox] = useState({ show: 0 });

    let showBox
    switch (changeBoxState.show) {
        case 1:
            showBox = <AddBox
                limitMapData={limitMapData}
                limitType={limitType}
                userId={parseInt(userId)}
                appId={(props.children && !appId) ? -1 : appId}
                appName={appName}
                onAdd={(id) => {
                    onSearchChange({
                        id: id,
                        app_id: appId,
                        page: 0
                    }, loadAppData)
                    setChangeBox({ show: 0 })
                }} />;
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
                sx={{ width: 450 }}
                role="presentation"
            >
                {showBox}
            </Box>
        </Drawer>

        <Paper
            sx={{ p: 2, display: 'flex', alignItems: 'center', marginBottom: 1, marginTop: 1 }}
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
                sx={{ mr: 1, p: "7px 15px" }}
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
                    setChangeBox({ show: 1 })
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
    </Fragment>
}


