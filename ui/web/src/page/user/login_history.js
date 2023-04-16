
import SearchIcon from '@mui/icons-material/Search';
import { Alert, FormControl, InputLabel, MenuItem, Paper, Popover, Select, Typography } from '@mui/material';
import Box from '@mui/material/Box';
import React, { useEffect, useState } from 'react';
import { ClearTextField } from '../../library/input';
import { LoadingButton } from '../../library/loading';
import { BaseTablePage } from '../../library/table_page';
import { loginHistroy } from '../../rest/user';
import { useSearchChange } from '../../utils/hook';
import { showTime } from '../../utils/utils';
import { PageNav } from './menu';
const filterStatus = {
    is_login: [
        { key: 0, val: '登陆失败' },
        { key: 1, val: '登陆成功' },
    ],
    login_type: [
        { key: "name Login", val: '用户名登陆' },
        { key: "Email Login", val: '邮箱登陆' },
    ]
};

function IpCityShow(props) {
    const { text, city } = props;
    const [anchorEl, setAnchorEl] = React.useState(null);
    const handlePopoverOpen = (event) => {
        setAnchorEl(event.currentTarget);
    };
    const handlePopoverClose = () => {
        setAnchorEl(null);
    };
    const open = Boolean(anchorEl);
    return (
        <div>
            <Typography
                aria-owns={open ? 'mouse-over-popover' : undefined}
                aria-haspopup="true"
                onMouseEnter={handlePopoverOpen}
                onMouseLeave={handlePopoverClose}
            >
                {text}
            </Typography>
            <Popover
                id="mouse-over-popover"
                sx={{
                    pointerEvents: 'none',
                }}
                open={open}
                anchorEl={anchorEl}
                anchorOrigin={{
                    vertical: 'bottom',
                    horizontal: 'left',
                }}
                transformOrigin={{
                    vertical: 'top',
                    horizontal: 'left',
                }}
                onClose={handlePopoverClose}
                disableRestoreFocus
            >
                <Typography sx={{ p: 1 }}>登陆城市:{city.replaceAll("-", " ")}</Typography>
            </Popover>
        </div>
    );
}

export default function UserLoginHistroyPage(props) {
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
            field: 'login_account',
            style: { width: 220 },
            label: '登陆账号',
        },
        {
            field: 'login_type',
            style: { width: 120 },
            label: '登陆类型',
            render: (row) => {
                let item = filterStatus.login_type.find((e) => {
                    return e.key == row.login_type
                });
                if (!item) return row.login_type;
                return item.val;
            }
        },
        {
            field: 'add_time',

            style: { width: 180 },
            label: '登陆时间',
            render: (row) => {
                return showTime(row.add_time, "未知")
            }
        },
        {
            field: 'login_ip',

            style: { width: 180 },
            label: '登陆IP(城市)',
            render: (row) => {
                if (row.login_city.length > 0) {
                    return <IpCityShow text={row.login_ip} city={row.login_city} />;
                }
                return row.login_ip;
            }
        },
        {
            field: 'is_login',
            style: { width: 100 },
            label: '登陆状态',
            render: (row) => {
                let item = filterStatus.is_login.find((e) => {
                    return e.key == row.is_login
                });
                if (!item) return row.is_login;
                return item.val;
            }
        },
        {
            field: 'login_msg',
            style: { width: 240 },
            label: '登陆消息',
            render: (row) => {
                return row.is_login ? "完成登陆" : row.login_msg
            }
        }
    ];
    const [searchParam, setSearchParam] = useSearchChange({
        login_type: "",
        login_account: "",
        is_login: '',
        page: 0,
        page_size: 10,
    });
    const [filterData, setfilterData] = useState({
        is_login: 0,
        login_type: searchParam.get("login_type"),
        login_account: searchParam.get("login_account")
    })
    const loadHistoryData = () => {
        setLoadData({
            ...loadData,
            loading: true
        })
        return loginHistroy({
            login_type: searchParam.get("login_type"),
            login_account: searchParam.get("login_account"),
            is_login: searchParam.get("is_login"),
            page: searchParam.get("page") || 0,
            page_size: searchParam.get("page_size") || 10
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
            is_login: searchParam.get("is_login"),
            login_type: searchParam.get("login_type"),
        })
        loadHistoryData()
    }, [searchParam])
    const execFilterData = () => {
        setSearchParam({

            ...filterData,
            page: 0
        }, loadHistoryData)
    }
    return <div>
        <PageNav />
        <Paper
            component="form"
            sx={{ p: 2, display: 'flex', alignItems: 'center', marginBottom: 1, marginTop: 1 }}
        >
            <FormControl sx={{ minWidth: 110, mr: 1 }} size="small"  >
                <InputLabel id="select-type">登录状态</InputLabel>
                <Select
                    labelId="select-type"
                    id="select-type"
                    label="登录状态"
                    disabled={loadData.loading}
                    onChange={(event) => {
                        setfilterData({
                            ...filterData,
                            is_login: event.target.value
                        })
                    }}
                    value={filterData.is_login ?? ''}
                >
                    <MenuItem value="">
                        全部
                    </MenuItem>
                    {
                        filterStatus.is_login.map((status) => {
                            return <MenuItem key={`status_${status.key}`} value={status.key}>{status.val}</MenuItem>
                        })
                    }
                </Select>
            </FormControl>
            <FormControl sx={{ minWidth: 110, mr: 1 }} size="small"  >
                <InputLabel id="select-small">登陆类型</InputLabel>
                <Select
                    labelId="select-small"
                    id="select-small"
                    label="登陆类型"
                    onChange={(event) => {
                        setfilterData({
                            ...filterData,
                            login_type: event.target.value
                        })
                    }}
                    disabled={loadData.loading}
                    value={filterData.login_type ?? ''}
                >
                    <MenuItem value="">
                        全部
                    </MenuItem>
                    {
                        filterStatus.login_type.map((status) => {
                            return <MenuItem key={`status_${status.key}`} value={status.key}>{status.val}</MenuItem>
                        })
                    }
                </Select>
            </FormControl>
            <FormControl sx={{ minWidth: 120, mr: 1 }} size="small"  >
                <ClearTextField
                    sx={{ mr: 1 }}
                    variant="outlined"
                    label={`用户账号`}
                    type="text"
                    name="code"
                    value={filterData.login_account}
                    size="small"
                    disabled={loadData.loading}
                    onChange={(event, nval) => {
                        setfilterData({
                            ...filterData,
                            login_account: nval
                        })
                    }}
                />
            </FormControl>
            <LoadingButton
                onClick={execFilterData}
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
                <BaseTablePage
                    rows={loadData.data}
                    columns={columns}
                    count={loadData.total}
                    page={searchParam.get("page") || 0}
                    onPageChange={(e, newPage) => {
                        setSearchParam({

                            page: newPage
                        }, loadHistoryData)
                    }}
                    rowsPerPage={searchParam.get("page_size") || 10}
                    onRowsPerPageChange={(e) => {
                        setSearchParam({

                            page_size: e.target.value,
                            page: 0
                        }, loadHistoryData)
                    }}
                    loading={loadData.loading}
                />
            </Box> : <Alert severity="error">{loadData.message}</Alert>}
    </div>
}


